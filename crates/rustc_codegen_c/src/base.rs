use rustc_codegen_ssa::ModuleCodegen;
use rustc_middle::ty::TyCtxt;

pub fn compile_codegen_unit(
    _tcx: TyCtxt<'_>,
    _cgu_name: rustc_span::Symbol,
) -> (ModuleCodegen<()>, u64) {
    todo!()
}
