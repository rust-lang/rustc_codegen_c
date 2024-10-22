#![feature(rustc_private)]

use blessed_test::*;
use rustc_codegen_c_ast::expr::CValue;
use rustc_codegen_c_ast::func::{CFunc, CFuncKind};
use rustc_type_ir::IntTy;

extern crate rustc_driver;
extern crate rustc_type_ir;
mod blessed_test;

#[test]
fn test_module() {
    printer_test("test_module", |ctx| {
        let module = ctx.module();
        module.push_include("stdio.h");

        module.push_decl(ctx.var(CValue::Local(42), ctx.get_int_type(IntTy::I32), None));

        let func = ctx.func(CFuncKind::new(
            "foo",
            ctx.get_int_type(IntTy::I32),
            vec![ctx.get_int_type(IntTy::I32)],
        ));
        let x = func.next_local_var();
        func.push_stmt(ctx.decl_stmt(ctx.var(x, ctx.get_int_type(IntTy::I32), None)));
        func.push_stmt(ctx.expr_stmt(ctx.binary(ctx.value(x), ctx.value(CValue::Scalar(1)), "=")));
        func.push_stmt(ctx.ret(Some(ctx.value(x))));
        module.push_func(CFunc::new_unchecked(func));
        Box::new(module.clone())
    });
}
