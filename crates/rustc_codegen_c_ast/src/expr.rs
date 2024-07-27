use crate::pretty::{Printer, INDENT};
use crate::r#type::CTy;
use crate::ModuleCtxt;

/// Values of C variable, parameters and scalars
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CValue {
    Scalar(i128),
    Local(usize),
}

pub type CExpr<'mx> = &'mx CExprKind<'mx>;

#[derive(Debug, Clone)]
pub enum CExprKind<'mx> {
    Raw(&'static str),
    Value(CValue),
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

    pub fn value(&self, value: CValue) -> CExpr<'mx> {
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

impl Printer {
    pub fn print_value(&mut self, value: CValue) {
        match value {
            CValue::Scalar(i) => self.word(i.to_string()),
            CValue::Local(i) => self.word(format!("_{}", i)),
        }
    }

    pub fn print_expr(&mut self, expr: CExpr) {
        match expr {
            CExprKind::Raw(raw) => self.word(*raw),
            CExprKind::Value(value) => self.print_value(*value),
            CExprKind::Binary { lhs, rhs, op } => self.ibox_delim(INDENT, ("(", ")"), 0, |this| {
                this.ibox(-INDENT, |this| this.print_expr(lhs));

                this.softbreak();
                this.word(*op);
                this.nbsp();

                this.print_expr(rhs);
            }),
            CExprKind::Cast { ty, expr } => self.ibox(INDENT, |this| {
                this.word("(");
                this.print_ty(*ty);
                this.word(")");

                this.nbsp();
                this.print_expr(expr);
            }),
            CExprKind::Call { callee, args } => self.ibox(INDENT, |this| {
                this.print_expr(callee);
                this.cbox_delim(INDENT, ("(", ")"), 0, |this| {
                    this.seperated(",", args, |this, arg| this.print_expr(arg))
                });
            }),
            CExprKind::Member { expr, arrow, field } => self.cbox(INDENT, |this| {
                this.print_expr(expr);
                this.zerobreak();
                if *arrow {
                    this.word("->");
                } else {
                    this.word(".");
                }
                this.word(field.to_string());
            }),
        }
    }
}
