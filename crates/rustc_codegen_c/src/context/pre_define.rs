use rustc_codegen_ssa::traits::PreDefineMethods;
use rustc_hir::def_id::DefId;
use rustc_middle::mir::mono::{Linkage, Visibility};
use rustc_middle::ty::layout::FnAbiOf;
use rustc_middle::ty::{self, Instance, Ty};
use rustc_target::abi::call::{ArgAbi, PassMode};

use crate::context::CodegenCx;
use crate::module::CType;

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

        let args = fn_abi.args.iter().map(|arg| type_from_abi(arg)).collect();
        let ret = type_from_abi(&fn_abi.ret);
        self.function_abis.borrow_mut().insert(symbol_name.to_string(), (args, ret));

        self.functions.borrow_mut().insert(symbol_name.to_string(), format!("{:?}", instance));
    }
}

fn type_from_abi(abi: &ArgAbi<'_, Ty>) -> CType {
    match &abi.mode {
        PassMode::Ignore => CType::Builtin("void".to_string()),
        PassMode::Direct(attributes) => {
            // TODO: other types
            CType::Builtin("int".to_string())
        }
        _ => todo!(),
    }
}
