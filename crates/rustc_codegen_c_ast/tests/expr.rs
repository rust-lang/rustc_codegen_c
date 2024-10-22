#![feature(rustc_private)]

use blessed_test::*;
use rustc_codegen_c_ast::expr::CValue;
use rustc_type_ir::IntTy;

extern crate rustc_driver;
extern crate rustc_type_ir;
mod blessed_test;

#[test]
fn test_value_scalar() {
    printer_test("test_value_scalar", |_| Box::new(CValue::Scalar(42)));
}

#[test]
fn test_value_local() {
    printer_test("test_value_local", |_| Box::new(CValue::Local(42)));
}

#[test]
fn test_value_func() {
    printer_test("test_value_func", |_| Box::new(CValue::Func("foo")));
}

#[test]
fn test_expr_raw() {
    printer_test("test_expr_raw", |ctx| Box::new(ctx.raw("42")));
}

#[test]
fn test_expr_binary() {
    printer_test("test_expr_binary", |ctx| {
        let lhs = ctx.value(CValue::Scalar(1));
        let rhs = ctx.value(CValue::Scalar(2));
        Box::new(ctx.binary(lhs, rhs, "+"))
    });
}

#[test]
fn test_expr_cast() {
    printer_test("test_expr_cast", |ctx| {
        let ty = ctx.get_int_type(IntTy::I32);
        let expr = ctx.value(CValue::Scalar(42));
        Box::new(ctx.cast(ty, expr))
    });
}

#[test]
fn test_expr_call() {
    printer_test("test_expr_call", |ctx| {
        let callee = ctx.value(CValue::Func("foo"));
        let args = vec![ctx.value(CValue::Scalar(1)), ctx.value(CValue::Scalar(2))];
        Box::new(ctx.call(callee, args))
    });
}

#[test]
fn test_expr_member() {
    printer_test("test_expr_member", |ctx| {
        let expr = ctx.value(CValue::Local(42));
        Box::new(ctx.member(expr, "foo"))
    });
}

#[test]
fn test_expr_complex() {
    printer_test("test_expr_complex", |ctx| {
        let lhs = ctx.value(CValue::Scalar(1));
        let rhs = ctx.value(CValue::Scalar(2));
        let expr = ctx.binary(lhs, rhs, "+");

        let ty = ctx.get_int_type(IntTy::I32);
        let cast = ctx.cast(ty, expr);

        let callee = ctx.value(CValue::Func("foo"));
        let args = vec![ctx.value(CValue::Scalar(1)), cast];
        let call = ctx.call(callee, args);

        let member = ctx.member(call, "bar");

        Box::new(member)
    });
}
