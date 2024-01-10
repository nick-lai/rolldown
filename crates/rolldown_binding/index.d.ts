/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface PluginOptions {
  name: string
  buildStart?: () => Promise<void>
  resolveId?: (
    specifier: string,
    importer?: string,
    options?: HookResolveIdArgsOptions,
  ) => Promise<undefined | ResolveIdResult>
  filterId?: () => string[] | FilterIdResult
  load?: (id: string) => Promise<undefined | SourceResult>
  transform?: (id: string, code: string) => Promise<undefined | SourceResult>
  buildEnd?: (error: string) => Promise<void>
}
export interface HookResolveIdArgsOptions {
  isEntry: boolean
  kind: string
}
export interface ResolveIdResult {
  id: string
  external?: boolean
}
export interface SourceResult {
  code: string
}
export interface FilterIdResult {
  include: Array<string>
  exclude: Array<string>
}
export interface InputItem {
  name?: string
  import: string
}
export interface ResolveOptions {
  alias?: Record<string, Array<string>>
  aliasFields?: Array<Array<string>>
  conditionNames?: Array<string>
  exportsFields?: Array<Array<string>>
  extensions?: Array<string>
  mainFields?: Array<string>
  mainFiles?: Array<string>
  modules?: Array<string>
  symlinks?: boolean
}
export interface InputOptions {
  external?: (source: string, importer?: string, isResolved: boolean) => boolean
  input: Array<InputItem>
  plugins: Array<PluginOptions>
  resolve?: ResolveOptions
  cwd: string
}
export interface OutputOptions {
  entryFileNames?: string
  chunkFileNames?: string
  dir?: string
  exports?: 'default' | 'named' | 'none' | 'auto'
  format?: 'esm' | 'cjs'
}
export interface RenderedModule {
  code?: string
  removedExports: Array<string>
  renderedExports: Array<string>
  originalLength: number
  renderedLength: number
}
export interface OutputChunk {
  code: string
  fileName: string
  isEntry: boolean
  isDynamicEntry: boolean
  facadeModuleId?: string
  modules: Record<string, RenderedModule>
  exports: Array<string>
}
export interface OutputAsset {
  fileName: string
  source: string
}
export interface Outputs {
  chunks: Array<OutputChunk>
  assets: Array<OutputAsset>
}
export class Bundler {
  constructor(inputOpts: InputOptions)
  write(opts: OutputOptions): Promise<Outputs>
  generate(opts: OutputOptions): Promise<Outputs>
  build(): Promise<void>
  scan(): Promise<void>
}
