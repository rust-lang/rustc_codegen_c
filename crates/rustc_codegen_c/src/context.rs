#![allow(unused_variables)] // TODO

use std::cell::RefCell;

use rustc_abi::{HasDataLayout, TargetDataLayout};
use rustc_codegen_ssa::traits::BackendTypes;
use rustc_hash::FxHashMap;
use rustc_middle::ty::layout::{
    FnAbiError, FnAbiOfHelpers, FnAbiRequest, HasParamEnv, HasTyCtxt, LayoutError, LayoutOfHelpers,
    TyAndLayout,
};
use rustc_middle::ty::{ParamEnv, Ty, TyCtxt};
use rustc_target::abi::call::FnAbi;
use rustc_target::spec::{HasTargetSpec, Target};

use crate::module::{CDecl, CParamVar, CStmt, CType, Module};

mod asm;
mod base_type;
mod r#const;
mod debug_info;
mod layout_type;
mod misc;
mod pre_define;
mod r#static;
mod type_membership;

pub struct CodegenCx<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub function_abis: RefCell<FxHashMap<String, (Vec<CType>, CType)>>,
    pub functions: RefCell<FxHashMap<String, String>>,
}

impl<'tcx> CodegenCx<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            function_abis: RefCell::new(FxHashMap::default()),
            functions: RefCell::new(FxHashMap::default()),
        }
    }

    pub fn finish(self) -> Module {
        let mut decls = vec![];

        let function_abis = self.function_abis.borrow();
        for (name, (args, ret)) in function_abis.iter() {
            decls.push(CDecl::Function {
                name: name.to_string(),
                ty: ret.clone(),
                params: args.iter().cloned().map(|ty| CParamVar { ty, name: None }).collect(),
                body: None,
            });
        }

        for (name, instance) in self.functions.into_inner() {
            let (args, ret) = &function_abis[&name];
            decls.push(CDecl::Function {
                name,
                ty: ret.clone(),
                params: args.iter().cloned().map(|ty| CParamVar { ty, name: None }).collect(),
                body: Some(CStmt::Compound(vec![CStmt::Decl(Box::new(CDecl::Raw(format!(
                    "// {}",
                    instance
                ))))])),
            });
        }

        Module { includes: vec![], decls }
    }
}

impl<'tcx> BackendTypes for CodegenCx<'tcx> {
    type Value = ();
    type Function = &'tcx str;
    type BasicBlock = ();
    type Type = ();
    type Funclet = ();
    type DIScope = ();
    type DILocation = ();
    type DIVariable = ();
}

impl<'tcx> HasTargetSpec for CodegenCx<'tcx> {
    fn target_spec(&self) -> &Target {
        todo!()
    }
}

impl<'tcx> HasParamEnv<'tcx> for CodegenCx<'tcx> {
    fn param_env(&self) -> ParamEnv<'tcx> {
        ParamEnv::reveal_all()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

impl HasDataLayout for CodegenCx<'_> {
    fn data_layout(&self) -> &TargetDataLayout {
        todo!()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for CodegenCx<'tcx> {
    type LayoutOfResult = TyAndLayout<'tcx>;

    fn handle_layout_err(&self, err: LayoutError<'tcx>, span: rustc_span::Span, ty: Ty<'tcx>) -> ! {
        todo!()
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'tcx> {
    type FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>;

    fn handle_fn_abi_err(
        &self,
        err: FnAbiError<'tcx>,
        span: rustc_span::Span,
        fn_abi_request: FnAbiRequest<'tcx>,
    ) -> ! {
        todo!()
    }
}
