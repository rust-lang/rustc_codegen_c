use std::cell::RefCell;

use crate::decl::CDecl;
use crate::func::{print_func_decl, CFunc};
use crate::pretty::{Print, PrinterCtx};

#[derive(Debug, Clone)]
pub struct Module<'mx> {
    pub includes: RefCell<Vec<&'static str>>,
    pub helper: &'static str,
    pub decls: RefCell<Vec<CDecl<'mx>>>,
    pub funcs: RefCell<Vec<CFunc<'mx>>>,
}

impl<'mx> Module<'mx> {
    pub fn new(helper: &'static str) -> Self {
        Self {
            includes: RefCell::new(Vec::new()),
            helper,
            decls: RefCell::new(Vec::new()),
            funcs: RefCell::new(Vec::new()),
        }
    }

    pub fn push_include(&self, include: &'static str) {
        self.includes.borrow_mut().push(include);
    }

    pub fn push_decl(&self, decl: CDecl<'mx>) {
        self.decls.borrow_mut().push(decl);
    }

    pub fn push_func(&self, func: CFunc<'mx>) {
        self.funcs.borrow_mut().push(func);
    }
}

impl Print for Module<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        ctx.cbox(0, |ctx| {
            for &include in self.includes.borrow().iter() {
                ctx.word("#include <");
                ctx.word(include);
                ctx.word(">");
                ctx.hardbreak();
            }

            ctx.hardbreak();

            ctx.word(self.helper);

            for &decl in self.decls.borrow().iter() {
                ctx.hardbreak();
                ctx.hardbreak();
                decl.print_to(ctx);
            }

            for &func in self.funcs.borrow().iter() {
                ctx.hardbreak();
                print_func_decl(func, ctx);
            }

            for &func in self.funcs.borrow().iter() {
                ctx.hardbreak();
                ctx.hardbreak();
                func.print_to(ctx);
            }

            ctx.hardbreak();
        });
    }
}
