#![allow(unused_variables)] // TODO

use std::cell::RefCell;

use rustc_abi::{HasDataLayout, TargetDataLayout};
use rustc_codegen_ssa::traits::BackendTypes;
use rustc_hash::FxHashMap;
use rustc_middle::ty::layout::{
    FnAbiError, FnAbiOfHelpers, FnAbiRequest, HasParamEnv, HasTyCtxt, LayoutError, LayoutOfHelpers,
    TyAndLayout,
};
use rustc_middle::ty::{Instance, ParamEnv, Ty, TyCtxt};
use rustc_target::abi::call::FnAbi;
use rustc_target::spec::{HasTargetSpec, Target};

use crate::module::{CDecl, CFunction, CStmt, CType, CValue, ModuleContext};
use crate::utils::slab::{Id, Slab};

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
    pub mcx: ModuleContext,
    pub function_instances: RefCell<FxHashMap<Instance<'tcx>, Id<CFunctionBuilder>>>,
    // TODO: better inner mutablity for slab
    pub functions: RefCell<Slab<CFunctionBuilder>>,
}

impl<'tcx> CodegenCx<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let mcx = ModuleContext::new();
        Self {
            tcx,
            mcx,
            function_instances: RefCell::new(FxHashMap::default()),
            functions: RefCell::new(Slab::default()),
        }
    }

    pub fn finish(mut self) -> ModuleContext {
        for function in self.functions.borrow().iter() {
            self.mcx.module.decls.push(function.decl());
        }
        for function in self.functions.into_inner() {
            self.mcx.module.decls.push(CDecl::Function(function.build()));
        }
        self.mcx
    }
}

impl<'tcx> BackendTypes for CodegenCx<'tcx> {
    type Value = CValue;
    type Function = Id<CFunctionBuilder>;
    type BasicBlock = Id<CFunctionBuilder>;
    type Type = CType;
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

pub struct CFunctionBuilder {
    pub name: String,
    pub ty: CType,
    pub params: Vec<(CType, CValue)>,
    pub body: Vec<CStmt>,
    pub var_names: FxHashMap<CValue, String>,
    var_counter: usize,
}

impl CFunctionBuilder {
    pub fn new(name: String, ty: CType, params: Vec<CType>) -> Self {
        let params: Vec<_> =
            params.into_iter().enumerate().map(|(i, ty)| (ty, CValue::Var(i))).collect();
        let var_counter = params.len();

        Self { name, ty, params, body: Vec::new(), var_counter, var_names: FxHashMap::default() }
    }

    pub fn build(self) -> CFunction {
        CFunction {
            name: self.name,
            ty: self.ty,
            params: self.params,
            body: self.body,
            var_names: self.var_names,
        }
    }

    pub fn decl(&self) -> CDecl {
        CDecl::FunctionDecl {
            name: self.name.clone(),
            ty: self.ty,
            params: self.params.iter().map(|(ty, _)| *ty).collect(),
        }
    }

    pub fn next_value(&mut self) -> CValue {
        let val = CValue::Var(self.var_counter);
        self.var_counter += 1;
        val
    }

    pub fn push_stmt(&mut self, stmt: CStmt) {
        self.body.push(stmt);
    }
}
