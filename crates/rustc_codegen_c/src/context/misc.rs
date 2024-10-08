use std::cell::RefCell;

use rustc_codegen_ssa::traits::MiscMethods;
use rustc_hash::FxHashMap;
use rustc_middle::mir::mono::CodegenUnit;
use rustc_middle::ty::{Instance, PolyExistentialTraitRef, Ty};

use crate::context::CodegenCx;

impl<'tcx, 'mx> MiscMethods<'tcx> for CodegenCx<'tcx, 'mx> {
    fn vtables(
        &self,
    ) -> &RefCell<FxHashMap<(Ty<'tcx>, Option<PolyExistentialTraitRef<'tcx>>), Self::Value>> {
        todo!()
    }

    fn get_fn(&self, instance: Instance<'tcx>) -> Self::Function {
        *self.function_instances.borrow().get(&instance).unwrap()
    }

    fn get_fn_addr(&self, instance: Instance<'tcx>) -> Self::Value {
        todo!()
    }

    fn eh_personality(&self) -> Self::Value {
        todo!()
    }

    fn sess(&self) -> &rustc_session::Session {
        self.tcx.sess
    }

    fn codegen_unit(&self) -> &'tcx CodegenUnit<'tcx> {
        todo!()
    }

    fn set_frame_pointer_type(&self, llfn: Self::Function) {
        todo!()
    }

    fn apply_target_cpu_attr(&self, llfn: Self::Function) {
        todo!()
    }

    fn declare_c_main(&self, fn_type: Self::Type) -> Option<Self::Function> {
        todo!()
    }
}
