#![feature(rustc_private)]

use blessed_test::*;
use rustc_codegen_c_ast::expr::CValue;
use rustc_type_ir::IntTy;

extern crate rustc_driver;
extern crate rustc_type_ir;
mod blessed_test;

#[test]
fn test_stmt_expr() {
    printer_test("test_stmt_expr", |ctx| {
        let callee = ctx.value(CValue::Func("foo"));
        let args = vec![ctx.value(CValue::Scalar(1)), ctx.value(CValue::Scalar(2))];
        let expr = ctx.call(callee, args);
        Box::new(ctx.expr_stmt(expr))
    });
}

#[test]
fn test_stmt_decl() {
    printer_test("test_stmt_decl", |ctx| {
        let ty = ctx.get_int_type(IntTy::I32);
        let name = CValue::Local(42);
        let value = ctx.value(CValue::Scalar(42));
        let decl = ctx.var(name, ty, Some(value));
        Box::new(ctx.decl_stmt(decl))
    });
}

#[test]
fn test_stmt_block() {
    printer_test("test_stmt_block", |ctx| {
        let callee = ctx.value(CValue::Func("foo"));
        let args = vec![ctx.value(CValue::Scalar(1)), ctx.value(CValue::Scalar(2))];
        let expr = ctx.call(callee, args);
        let stmt = ctx.expr_stmt(expr);
        Box::new(ctx.compound(vec![stmt]))
    });
}

#[test]
fn test_stmt_ret() {
    printer_test("test_stmt_if", |ctx| {
        let callee = ctx.value(CValue::Func("foo"));
        let args = vec![ctx.value(CValue::Scalar(1)), ctx.value(CValue::Scalar(2))];
        let expr = ctx.call(callee, args);
        Box::new(ctx.ret(Some(expr)))
    });
}
