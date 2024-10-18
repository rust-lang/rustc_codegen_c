//! This module defines the AST nodes for C expressions.

use crate::pretty::{Print, PrinterCtx, INDENT};
use crate::ty::{print_declarator, CTy};
use crate::ModuleCtx;

/// Represents the values of C variables, parameters, and scalars.
///
/// There are two variants to distinguish between constants and variables,
/// as is done in LLVM IR. We follow the `rustc_codegen_ssa` convention for this representation.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CValue<'mx> {
    /// A constant scalar
    Scalar(i128),
    /// A local variable indexed by a number, in the form `_0`, `_1`, etc.
    Local(usize),
    /// A function name
    Func(&'mx str),
}

/// C expressions.
pub type CExpr<'mx> = &'mx CExprKind<'mx>;

/// C expressions.
#[derive(Debug, Clone)]
pub enum CExprKind<'mx> {
    /// A "raw" C expression, simply a string of C code, which is printed as-is.
    Raw(&'static str),
    /// A value, such as a constant, variable, or function name.
    Value(CValue<'mx>),
    /// A binary operation expression, e.g. `lhs + rhs`.
    Binary { lhs: CExpr<'mx>, rhs: CExpr<'mx>, op: &'static str },
    /// A type cast expression, e.g. `(int) x`.
    Cast { ty: CTy<'mx>, expr: CExpr<'mx> },
    /// A function call expression, e.g. `foo(x, y)`.
    Call { callee: CExpr<'mx>, args: Vec<CExpr<'mx>> },
    /// A member access expression, e.g. `foo.bar` or `foo->bar`.
    Member {
        expr: CExpr<'mx>,
        /// Whether to use the `->` operator instead of `.`.
        arrow: bool,
        field: &'mx str,
    },
}

impl<'mx> ModuleCtx<'mx> {
    /// Create a new expression.
    pub fn expr(&self, expr: CExprKind<'mx>) -> CExpr<'mx> {
        self.arena().alloc(expr)
    }

    /// Create a new raw expression.
    pub fn raw(&self, raw: &'static str) -> CExpr<'mx> {
        self.expr(CExprKind::Raw(raw))
    }

    /// Create a new value expression.
    pub fn value(&self, value: CValue<'mx>) -> CExpr<'mx> {
        self.expr(CExprKind::Value(value))
    }

    /// Create a new binary expression.
    pub fn binary(&self, lhs: CExpr<'mx>, rhs: CExpr<'mx>, op: &'static str) -> CExpr<'mx> {
        self.expr(CExprKind::Binary { lhs, rhs, op })
    }

    /// Create a new cast expression.
    pub fn cast(&self, ty: CTy<'mx>, expr: CExpr<'mx>) -> CExpr<'mx> {
        self.expr(CExprKind::Cast { ty, expr })
    }

    /// Create a new function call expression.
    pub fn call(&self, callee: CExpr<'mx>, args: Vec<CExpr<'mx>>) -> CExpr<'mx> {
        self.expr(CExprKind::Call { callee, args })
    }

    /// Create a new member access expression.
    pub fn member(&self, expr: CExpr<'mx>, field: &'mx str) -> CExpr<'mx> {
        self.expr(CExprKind::Member { expr, field, arrow: false })
    }
}

impl Print for CValue<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        match self {
            CValue::Scalar(i) => ctx.word(i.to_string()),
            CValue::Local(i) => ctx.word(format!("_{}", i)),
            CValue::Func(name) => ctx.word(name.to_string()),
        }
    }
}

impl Print for CExpr<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        match self {
            CExprKind::Raw(raw) => ctx.word(*raw),
            CExprKind::Value(value) => value.print_to(ctx),
            CExprKind::Binary { lhs, rhs, op } => ctx.ibox_delim(INDENT, ("(", ")"), 0, |ctx| {
                ctx.ibox(-INDENT, |ctx| lhs.print_to(ctx));

                ctx.softbreak();
                ctx.word(*op);
                ctx.nbsp();

                rhs.print_to(ctx);
            }),
            CExprKind::Cast { ty, expr } => ctx.ibox(INDENT, |ctx| {
                ctx.word("(");
                print_declarator(*ty, None, ctx);
                ctx.word(")");

                ctx.nbsp();
                expr.print_to(ctx);
            }),
            CExprKind::Call { callee, args } => ctx.ibox(INDENT, |ctx| {
                callee.print_to(ctx);
                ctx.cbox_delim(INDENT, ("(", ")"), 0, |ctx| {
                    ctx.seperated(",", args, |ctx, arg| arg.print_to(ctx));
                });
            }),
            CExprKind::Member { expr, arrow, field } => ctx.cbox(INDENT, |ctx| {
                expr.print_to(ctx);
                ctx.zerobreak();
                if *arrow {
                    ctx.word("->");
                } else {
                    ctx.word(".");
                }
                ctx.word(field.to_string());
            }),
        }
    }
}
