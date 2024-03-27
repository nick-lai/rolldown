use napi::Env;
use napi_derive::napi;
use rolldown::{Bundler as NativeBundler, BundlerBuilder};

use crate::{
  options::{BindingInputOptions, BindingOutputOptions},
  types::binding_outputs::BindingOutputs,
  utils::{normalize_binding_options::normalize_binding_options, try_init_custom_trace_subscriber},
};

#[napi]
pub struct Bundler {
  inner: NativeBundler,
}

#[napi]
impl Bundler {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    input_options: BindingInputOptions,
    output_options: BindingOutputOptions,
  ) -> napi::Result<Self> {
    #[cfg(not(target_family = "wasm"))]
    {
      try_init_custom_trace_subscriber(env);
    }
    let ret = normalize_binding_options(input_options, output_options)?;

    Ok(Self {
      inner: BundlerBuilder::default()
        .with_input_options(ret.input_options)
        .with_output_options(ret.output_options)
        .with_plugins(ret.plugins)
        .build(),
    })
  }

  #[napi]
  pub async unsafe fn write(&mut self) -> napi::Result<BindingOutputs> {
    let maybe_outputs = self.inner.write().await;

    let outputs = match maybe_outputs {
      Ok(outputs) => outputs,
      Err(err) => {
        // TODO: better handing errors
        eprintln!("{err:?}");
        return Err(napi::Error::from_reason("Build failed"));
      }
    };

    Ok(outputs.assets.into())
  }

  #[napi]
  pub async unsafe fn generate(&mut self) -> napi::Result<BindingOutputs> {
    let maybe_outputs = self.inner.generate().await;

    let outputs = match maybe_outputs {
      Ok(outputs) => outputs,
      Err(err) => {
        // TODO: better handing errors
        eprintln!("{err:?}");
        return Err(napi::Error::from_reason("Build failed"));
      }
    };

    Ok(outputs.assets.into())
  }

  #[napi]
  pub async unsafe fn scan(&mut self) -> napi::Result<()> {
    let result = self.inner.scan().await;

    if let Err(err) = result {
      // TODO: better handing errors
      eprintln!("{err:?}");
      return Err(napi::Error::from_reason("Build failed"));
    }

    Ok(())
  }
}
