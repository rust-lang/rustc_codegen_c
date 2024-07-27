use crate::expr::{CExpr, CValue};
use crate::pretty::{Printer, INDENT};
use crate::r#type::CTy;
use crate::ModuleCtxt;

pub type CDecl<'mx> = &'mx CDeclKind<'mx>;

#[derive(Debug, Clone)]
pub enum CDeclKind<'mx> {
    // Typedef { name: String, ty: CType },
    // Record { name: String, fields: Vec<CDecl> },
    // Field { name: String, ty: CType },
    // Enum { name: String, values: Vec<CEnumConstant> },
    Var { name: CValue, ty: CTy<'mx>, init: Option<CExpr<'mx>> },
}

impl<'mx> ModuleCtxt<'mx> {
    pub fn decl(self, decl: CDeclKind<'mx>) -> CDecl<'mx> {
        self.arena().alloc(decl)
    }

    pub fn var(self, name: CValue, ty: CTy<'mx>, init: Option<CExpr<'mx>>) -> CDecl<'mx> {
        self.decl(CDeclKind::Var { name, ty, init })
    }
}

impl Printer {
    pub fn print_decl(&mut self, decl: CDecl) {
        match decl {
            CDeclKind::Var { name, ty, init } => {
                self.ibox(INDENT, |this| {
                    this.print_ty(*ty);
                    this.nbsp();
                    this.print_value(*name);
                    if let Some(init) = init {
                        this.word(" =");
                        this.softbreak();
                        this.print_expr(init);
                    }
                    this.word(";");
                });
            }
        }
    }
}
