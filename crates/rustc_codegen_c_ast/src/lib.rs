#![feature(rustc_private)]

use std::fmt::{self, Display};

extern crate rustc_arena;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_type_ir;

pub mod arena;
pub mod decl;
pub mod expr;
pub mod func;
pub mod module;
pub mod pretty;
pub mod stmt;
pub mod r#type;

#[derive(Clone, Copy)]
pub struct ModuleCtxt<'mx>(pub &'mx ModuleArena<'mx>);

impl<'mx> ModuleCtxt<'mx> {
    pub fn arena(&self) -> &'mx arena::Arena<'mx> {
        &self.0.arena
    }

    pub fn module(&self) -> &'mx module::Module<'mx> {
        &self.0.module
    }

    pub fn alloc_str(&self, s: &str) -> &'mx str {
        self.arena().alloc_str(s)
    }
}

impl<'mx> Display for ModuleCtxt<'mx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut printer = pretty::Printer::new();
        printer.print_module(self.module());
        write!(f, "{}", printer.finish())
    }
}

pub struct ModuleArena<'mx> {
    pub arena: arena::Arena<'mx>,
    pub module: module::Module<'mx>,
}

impl<'mx> ModuleArena<'mx> {
    pub fn new(helper: &'static str) -> Self {
        Self { arena: arena::Arena::default(), module: module::Module::new(helper) }
    }
}
