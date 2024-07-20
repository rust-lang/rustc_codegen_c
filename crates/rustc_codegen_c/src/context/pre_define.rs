use rustc_codegen_ssa::traits::{LayoutTypeMethods, PreDefineMethods};
use rustc_hir::def_id::DefId;
use rustc_middle::mir::mono::{Linkage, Visibility};
use rustc_middle::ty::layout::FnAbiOf;
use rustc_middle::ty::{self, Instance};

use crate::context::{CFunctionBuilder, CodegenCx};

impl<'tcx> PreDefineMethods<'tcx> for CodegenCx<'tcx> {
    fn predefine_static(
        &self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        todo!()
    }

    fn predefine_fn(
        &self,
        instance: Instance<'tcx>,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        let fn_abi = self.fn_abi_of_instance(instance, ty::List::empty());

        let args = fn_abi.args.iter().map(|arg| self.immediate_backend_type(arg.layout)).collect();
        let ret = self.immediate_backend_type(fn_abi.ret.layout);

        let function = CFunctionBuilder::new(symbol_name.to_string(), ret, args);
        let function_id = self.functions.borrow_mut().insert(function);
        self.function_instances.borrow_mut().insert(instance, function_id);
    }
}
