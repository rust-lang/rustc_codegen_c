use crate::pretty::{Print, PrinterCtx, INDENT};
use crate::ty::{print_declarator, CTy};
use crate::ModuleCtxt;

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

pub type CExpr<'mx> = &'mx CExprKind<'mx>;

#[derive(Debug, Clone)]
pub enum CExprKind<'mx> {
    Raw(&'static str),
    Value(CValue<'mx>),
    Binary { lhs: CExpr<'mx>, rhs: CExpr<'mx>, op: &'static str },
    Cast { ty: CTy<'mx>, expr: CExpr<'mx> },
    Call { callee: CExpr<'mx>, args: Vec<CExpr<'mx>> },
    Member { expr: CExpr<'mx>, arrow: bool, field: &'mx str },
}

impl<'mx> ModuleCtxt<'mx> {
    pub fn expr(&self, expr: CExprKind<'mx>) -> CExpr<'mx> {
        self.arena().alloc(expr)
    }

    pub fn raw(&self, raw: &'static str) -> CExpr<'mx> {
        self.expr(CExprKind::Raw(raw))
    }

    pub fn value(&self, value: CValue<'mx>) -> CExpr<'mx> {
        self.expr(CExprKind::Value(value))
    }

    pub fn binary(&self, lhs: CExpr<'mx>, rhs: CExpr<'mx>, op: &'static str) -> CExpr<'mx> {
        self.expr(CExprKind::Binary { lhs, rhs, op })
    }

    pub fn cast(&self, ty: CTy<'mx>, expr: CExpr<'mx>) -> CExpr<'mx> {
        self.expr(CExprKind::Cast { ty, expr })
    }

    pub fn call(&self, callee: CExpr<'mx>, args: Vec<CExpr<'mx>>) -> CExpr<'mx> {
        self.expr(CExprKind::Call { callee, args })
    }

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
