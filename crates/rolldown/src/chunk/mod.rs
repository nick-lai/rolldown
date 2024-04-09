// cSpell:disable
mod de_conflict;
pub mod render_chunk;
mod render_chunk_exports;
mod render_chunk_imports;

use index_vec::IndexVec;
use rolldown_common::{ChunkId, FileNameTemplate};

pub type ChunksVec = IndexVec<ChunkId, Chunk>;

use rolldown_common::{
  ChunkKind, ExternalModuleId, NamedImport, NormalModuleId, RenderedChunk, Specifier, SymbolRef,
};
use rolldown_rstr::Rstr;
use rolldown_sourcemap::{ConcatSource, RawSource, SourceMap, SourceMapSource};
use rolldown_utils::BitSet;
use rustc_hash::FxHashMap;
use sugar_path::SugarPath;

use crate::types::module_render_output::ModuleRenderOutput;
use crate::utils::render_normal_module::render_normal_module;
use crate::SharedOptions;
use crate::{
  error::BatchedResult,
  {chunk_graph::ChunkGraph, stages::link_stage::LinkStageOutput},
};

#[derive(Debug, Clone)]
pub struct CrossChunkImportItem {
  pub export_alias: Option<Specifier>,
  pub import_ref: SymbolRef,
}

#[derive(Debug, Default)]
pub struct Chunk {
  pub kind: ChunkKind,
  pub modules: Vec<NormalModuleId>,
  pub name: Option<String>,
  pub file_name: Option<String>,
  pub canonical_names: FxHashMap<SymbolRef, Rstr>,
  pub bits: BitSet,
  pub imports_from_other_chunks: FxHashMap<ChunkId, Vec<CrossChunkImportItem>>,
  pub imports_from_external_modules: FxHashMap<ExternalModuleId, Vec<NamedImport>>,
  // meaningless if the chunk is an entrypoint
  pub exports_to_other_chunks: FxHashMap<SymbolRef, Rstr>,
}

pub struct ChunkRenderReturn {
  pub code: String,
  pub map: Option<SourceMap>,
  pub rendered_chunk: RenderedChunk,
}

impl Chunk {
  pub fn new(
    name: Option<String>,
    bits: BitSet,
    modules: Vec<NormalModuleId>,
    kind: ChunkKind,
  ) -> Self {
    Self { modules, name, bits, kind, ..Self::default() }
  }

  pub fn file_name_template<'a>(
    &mut self,
    output_options: &'a SharedOptions,
  ) -> &'a FileNameTemplate {
    if matches!(self.kind, ChunkKind::EntryPoint { is_user_defined, .. } if is_user_defined) {
      &output_options.entry_file_names
    } else {
      &output_options.chunk_file_names
    }
  }

  #[allow(clippy::unnecessary_wraps, clippy::cast_possible_truncation)]
  pub async fn render(
    &self,
    options: &SharedOptions,
    graph: &LinkStageOutput,
    chunk_graph: &ChunkGraph,
  ) -> BatchedResult<ChunkRenderReturn> {
    use rayon::prelude::*;
    let mut rendered_modules = FxHashMap::default();
    let mut concat_source = ConcatSource::default();

    concat_source
      .add_source(Box::new(RawSource::new(self.render_imports_for_esm(graph, chunk_graph))));

    self
      .modules
      .par_iter()
      .copied()
      .map(|id| &graph.module_table.normal_modules[id])
      .filter_map(|m| {
        // Here file path is generated by chunk file name template, it maybe including path segments.
        // So here need to read it's parent directory as file_dir.
        let file_path = options.cwd.as_path().join(&options.dir).join(
          self.file_name.as_ref().expect("chunk file name should be generated before rendering"),
        );
        let file_dir = file_path.parent().expect("chunk file name should have a parent");
        render_normal_module(
          m,
          &graph.ast_table[m.id],
          // TODO(underfin): refactor the relative path
          m.resource_id.expect_file().relative_path(file_dir).to_slash_lossy().as_ref(),
          options,
          file_dir.to_string_lossy().as_ref(),
        )
      })
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|module_render_output| {
        let ModuleRenderOutput {
          module_path,
          module_pretty_path,
          rendered_module,
          rendered_content,
          sourcemap,
          lines_count,
        } = module_render_output;
        concat_source.add_source(Box::new(RawSource::new(format!("// {module_pretty_path}",))));
        if let Some(sourcemap) = sourcemap {
          concat_source.add_source(Box::new(SourceMapSource::new(
            rendered_content,
            sourcemap,
            lines_count,
          )));
        } else {
          concat_source.add_source(Box::new(RawSource::new(rendered_content)));
        }
        rendered_modules.insert(module_path.to_string(), rendered_module);
      });
    let rendered_chunk = self.get_rendered_chunk_info(graph, options, rendered_modules);

    // TODO avoid rendered_chunk clone
    // add banner
    if let Some(banner_txt) = options.banner.call(rendered_chunk.clone()).await? {
      concat_source.add_prepend_source(Box::new(RawSource::new(banner_txt)));
    }

    if let Some(exports) = self.render_exports(graph, options) {
      concat_source.add_source(Box::new(RawSource::new(exports)));
    }

    // add footer
    if let Some(footer_txt) = options.footer.call(rendered_chunk.clone()).await? {
      concat_source.add_source(Box::new(RawSource::new(footer_txt)));
    }

    let (content, map) = concat_source.content_and_sourcemap();

    Ok(ChunkRenderReturn { code: content, map, rendered_chunk })
  }
}
