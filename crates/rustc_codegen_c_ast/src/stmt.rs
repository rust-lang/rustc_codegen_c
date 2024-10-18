use crate::decl::CDecl;
use crate::expr::CExpr;
use crate::pretty::{Print, PrinterCtx, INDENT};
use crate::ModuleCtxt;

pub type CStmt<'mx> = &'mx CStmtKind<'mx>;

#[derive(Debug, Clone)]
pub enum CStmtKind<'mx> {
    Compound(Vec<CStmt<'mx>>),
    // If { cond: CExpr<'mx>, then_br: CStmt<'mx>, else_br: Option<CStmt<'mx>> },
    Return(Option<CExpr<'mx>>),
    Decl(CDecl<'mx>),
    Expr(CExpr<'mx>),
}

impl<'mx> ModuleCtxt<'mx> {
    pub fn stmt(self, stmt: CStmtKind<'mx>) -> CStmt<'mx> {
        self.arena().alloc(stmt)
    }

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

    pub fn ret(self, expr: Option<CExpr<'mx>>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Return(expr))
    }

    pub fn decl_stmt(self, decl: CDecl<'mx>) -> CStmt<'mx> {
        self.stmt(CStmtKind::Decl(decl))
    }

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
