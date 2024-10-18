use crate::expr::{CExpr, CValue};
use crate::pretty::{Print, PrinterCtx, INDENT};
use crate::ty::{print_declarator, CTy};
use crate::ModuleCtxt;

pub type CDecl<'mx> = &'mx CDeclKind<'mx>;

#[derive(Debug, Clone)]
pub enum CDeclKind<'mx> {
    // Typedef { name: String, ty: CType },
    // Record { name: String, fields: Vec<CDecl> },
    // Field { name: String, ty: CType },
    // Enum { name: String, values: Vec<CEnumConstant> },
    Var { name: CValue<'mx>, ty: CTy<'mx>, init: Option<CExpr<'mx>> },
}

impl<'mx> ModuleCtxt<'mx> {
    pub fn decl(self, decl: CDeclKind<'mx>) -> CDecl<'mx> {
        self.arena().alloc(decl)
    }

    pub fn var(self, name: CValue<'mx>, ty: CTy<'mx>, init: Option<CExpr<'mx>>) -> CDecl<'mx> {
        self.decl(CDeclKind::Var { name, ty, init })
    }
}

impl Print for CDecl<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        match self {
            CDeclKind::Var { name, ty, init } => {
                ctx.ibox(INDENT, |ctx| {
                    print_declarator(*ty, Some(*name), ctx);
                    if let Some(init) = init {
                        ctx.word(" =");
                        ctx.softbreak();
                        init.print_to(ctx);
                    }
                    ctx.word(";");
                });
            }
        }
    }
}
