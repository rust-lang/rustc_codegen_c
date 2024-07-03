use rustc_codegen_ssa::traits::StaticBuilderMethods;
use rustc_hir::def_id::DefId;

use crate::builder::Builder;

impl<'tcx> StaticBuilderMethods for Builder<'_, 'tcx> {
    fn get_static(&mut self, def_id: DefId) -> Self::Value {
        todo!()
    }
}
