//! This crate defines the subset of the C AST used by the C codegen backend.
//!
//! Its primary purpose is to facilitate the construction and pretty-printing of C code.
//! Note that parsing is not included in this crate.
//!
//! It also provides utilities to assist with building the C AST and integrating
//! with the `rustc_codegen_ssa` backend.
#![feature(rustc_private)]

use std::fmt::{self, Display};

use crate::pretty::Print;

extern crate rustc_arena;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_type_ir;

pub mod arena;
pub mod decl;
pub mod expr;
pub mod func;
pub mod module;
pub mod pretty;
pub mod stmt;
pub mod ty;

/// A context containing a C file's AST nodes which are under construction.
#[derive(Clone, Copy)]
pub struct ModuleCtx<'mx>(pub &'mx ModuleArena<'mx>);

impl<'mx> ModuleCtx<'mx> {
    /// Get the memory arena for the context.
    pub fn arena(&self) -> &'mx arena::Arena<'mx> {
        &self.0.arena
    }

    /// Get the AST node for the module.
    pub fn module(&self) -> &'mx module::Module<'mx> {
        &self.0.module
    }

    /// Allocate a string in the arena.
    pub fn alloc_str(&self, s: &str) -> &'mx str {
        self.arena().alloc_str(s)
    }
}

impl<'mx> Display for ModuleCtx<'mx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut printer = pretty::PrinterCtx::new();
        self.module().print_to(&mut printer);
        write!(f, "{}", printer.finish())
    }
}

pub struct ModuleArena<'mx> {
    /// The memory arena for the module.
    pub arena: arena::Arena<'mx>,
    /// The module's AST node.
    pub module: module::Module<'mx>,
}

impl<'mx> ModuleArena<'mx> {
    pub fn new(helper: &'static str) -> Self {
        Self { arena: arena::Arena::default(), module: module::Module::new(helper) }
    }
}
