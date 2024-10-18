#![feature(rustc_private)]

use blessed_test::*;
use rustc_codegen_c_ast::expr::CValue;
use rustc_type_ir::IntTy;

extern crate rustc_driver;
extern crate rustc_type_ir;
mod blessed_test;

#[test]
fn test_decl_var() {
    printer_test("test_decl_var", |ctx| {
        let ty = ctx.get_int_type(IntTy::I32);
        let name = CValue::Local(42);
        let value = ctx.value(CValue::Scalar(42));
        Box::new(ctx.var(name, ty, Some(value)))
    });
}