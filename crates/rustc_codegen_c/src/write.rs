use rustc_codegen_ssa::back::write::{CodegenContext, ModuleConfig};
use rustc_codegen_ssa::{CompiledModule, ModuleCodegen};
use rustc_errors::{DiagCtxtHandle, FatalError};

pub(crate) unsafe fn codegen(
    _cgcx: &CodegenContext<crate::CCodegen>,
    _dcx: DiagCtxtHandle<'_>,
    _module: ModuleCodegen<()>,
    _config: &ModuleConfig,
) -> Result<CompiledModule, FatalError> {
    todo!()
}

pub(crate) fn link(
    _cgcx: &CodegenContext<crate::CCodegen>,
    _dcx: DiagCtxtHandle<'_>,
    mut _modules: Vec<ModuleCodegen<()>>,
) -> Result<ModuleCodegen<()>, FatalError> {
    unimplemented!();
}
