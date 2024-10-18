//! This module defines the AST nodes for C statements.

use crate::decl::CDecl;
use crate::expr::CExpr;
use crate::pretty::{Print, PrinterCtx, INDENT};
use crate::ModuleCtx;

/// C statement.
pub type CStmt<'mx> = &'mx CStmtKind<'mx>;

/// C statement.
#[derive(Debug, Clone)]
pub enum CStmtKind<'mx> {
    /// Compound statement, which is a sequence of statements enclosed in braces.
    Compound(Vec<CStmt<'mx>>),
    // If { cond: CExpr<'mx>, then_br: CStmt<'mx>, else_br: Option<CStmt<'mx>> },
    /// Return statement.
    Return(Option<CExpr<'mx>>),
    /// Declaration statement, e.g. `int x = 42;`.
    Decl(CDecl<'mx>),
    /// Expression statement, e.g. `foo(x + 1);`.
    Expr(CExpr<'mx>),
}

impl<'mx> ModuleCtx<'mx> {
    /// Create a new statement.
    pub fn stmt(self, stmt: CStmtKind<'mx>) -> CStmt<'mx> {
        self.arena().alloc(stmt)
    }

    /// Create a compound statement.
    pub fn compound(self, stmts: Vec<CStmt<'mx>>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Compound(stmts))
    }

    // pub fn if_stmt(
    //     self,
    //     cond: CExpr<'mx>,
    //     then_br: CStmt<'mx>,
    //     else_br: Option<CStmt<'mx>>,
    // ) -> CStmt<'mx> {
    //     self.stmt(CStmtKind::If { cond, then_br, else_br })
    // }

    /// Create a return statement.
    pub fn ret(self, expr: Option<CExpr<'mx>>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Return(expr))
    }

    /// Create a declaration statement.
    pub fn decl_stmt(self, decl: CDecl<'mx>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Decl(decl))
    }

    /// Create an expression statement.
    pub fn expr_stmt(self, expr: CExpr<'mx>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Expr(expr))
    }
}

impl Print for CStmt<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        match self {
            CStmtKind::Compound(stmts) => print_compound(stmts, ctx),
            CStmtKind::Return(ret) => {
                ctx.ibox(INDENT, |ctx| {
                    ctx.word("return");
                    if let Some(ret) = ret {
                        ctx.softbreak();
                        ret.print_to(ctx);
                    }
                    ctx.word(";");
                });
            }
            CStmtKind::Decl(decl) => decl.print_to(ctx),
            CStmtKind::Expr(expr) => {
                expr.print_to(ctx);
                ctx.word(";");
            }
        }
    }
}

/// Print a compound statement.
pub(crate) fn print_compound(stmts: &[CStmt], ctx: &mut PrinterCtx) {
    ctx.cbox_delim(INDENT, ("{", "}"), 1, |ctx| {
        if let Some((first, rest)) = stmts.split_first() {
            first.print_to(ctx);
            for stmt in rest {
                ctx.hardbreak();
                stmt.print_to(ctx);
            }
        }
    });
}
