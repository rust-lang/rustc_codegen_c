use rustc_codegen_ssa::traits::CoverageInfoBuilderMethods;

use crate::builder::Builder;

impl<'tcx> CoverageInfoBuilderMethods<'tcx> for Builder<'_, 'tcx> {
    fn add_coverage(
        &mut self,
        instance: rustc_middle::ty::Instance<'tcx>,
        kind: &rustc_middle::mir::coverage::CoverageKind,
    ) {
        todo!()
    }
}
