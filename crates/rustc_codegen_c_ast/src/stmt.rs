use crate::decl::CDecl;
use crate::expr::CExpr;
use crate::pretty::{Printer, INDENT};
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

impl Printer {
    pub fn print_stmt(&mut self, stmt: CStmt) {
        match stmt {
            CStmtKind::Compound(stmts) => self.print_compound(stmts),
            CStmtKind::Return(ret) => {
                self.ibox(INDENT, |this| {
                    this.word("return");
                    if let Some(ret) = ret {
                        this.softbreak();
                        this.print_expr(ret);
                    }
                    this.word(";");
                });
            }
            CStmtKind::Decl(decl) => self.print_decl(decl),
            CStmtKind::Expr(expr) => {
                self.print_expr(expr);
                self.word(";");
            }
        }
    }

    pub(crate) fn print_compound(&mut self, stmts: &[CStmt]) {
        self.cbox_delim(INDENT, ("{", "}"), 1, |this| {
            if let Some((first, rest)) = stmts.split_first() {
                this.print_stmt(first);
                for stmt in rest {
                    this.hardbreak();
                    this.print_stmt(stmt);
                }
            }
        });
    }
}
