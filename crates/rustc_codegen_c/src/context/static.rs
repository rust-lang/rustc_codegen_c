use rustc_abi::Align;
use rustc_codegen_ssa::traits::StaticMethods;
use rustc_hir::def_id::DefId;

use crate::context::CodegenCx;

impl<'tcx> StaticMethods for CodegenCx<'tcx> {
    fn static_addr_of(&self, cv: Self::Value, align: Align, kind: Option<&str>) -> Self::Value {
        todo!()
    }

    fn codegen_static(&self, def_id: DefId) {
        todo!()
    }

    fn add_used_global(&self, global: Self::Value) {
        todo!()
    }

    fn add_compiler_used_global(&self, global: Self::Value) {
        todo!()
    }
}
