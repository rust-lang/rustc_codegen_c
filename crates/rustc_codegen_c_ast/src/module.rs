use std::cell::RefCell;

use crate::decl::CDecl;
use crate::func::CFunc;
use crate::pretty::Printer;

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

impl Printer {
    pub fn print_module(&mut self, module: &Module) {
        self.cbox(0, |this| {
            for &include in module.includes.borrow().iter() {
                this.word("#include <");
                this.word(include);
                this.word(">");
                this.hardbreak();
            }

            this.hardbreak();

            this.word(module.helper);

            for &decl in module.decls.borrow().iter() {
                this.hardbreak();
                this.hardbreak();
                this.print_decl(decl);
            }

            for &func in module.funcs.borrow().iter() {
                this.hardbreak();
                this.print_func_decl(func);
            }

            for &func in module.funcs.borrow().iter() {
                this.hardbreak();
                this.hardbreak();
                this.print_func(func);
            }

            this.hardbreak();
        });
    }
}
