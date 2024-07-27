use std::cell::{Cell, RefCell};

use rustc_data_structures::intern::Interned;

use crate::expr::CValue;
use crate::pretty::Printer;
use crate::r#type::CTy;
use crate::stmt::CStmt;
use crate::ModuleCtxt;

pub type CFunc<'mx> = Interned<'mx, CFuncKind<'mx>>;

#[derive(Debug, Clone)]
pub struct CFuncKind<'mx> {
    pub name: &'mx str,
    pub ty: CTy<'mx>,
    pub params: Vec<(CTy<'mx>, CValue)>,
    pub body: RefCell<Vec<CStmt<'mx>>>,
    local_var_counter: Cell<usize>,
}

impl<'mx> CFuncKind<'mx> {
    pub fn new(name: &'mx str, ty: CTy<'mx>, params: impl IntoIterator<Item = CTy<'mx>>) -> Self {
        let params = params
            .into_iter()
            .enumerate()
            .map(|(i, ty)| (ty, CValue::Local(i)))
            .collect::<Vec<_>>();
        let local_var_counter = Cell::new(params.len());

        Self { name, ty, params, body: RefCell::new(Vec::new()), local_var_counter }
    }

    pub fn push_stmt(&self, stmt: CStmt<'mx>) {
        self.body.borrow_mut().push(stmt);
    }

    pub fn next_local_var(&self) -> CValue {
        let val = CValue::Local(self.local_var_counter.get());
        self.local_var_counter.set(self.local_var_counter.get() + 1);
        val
    }
}

impl<'mx> ModuleCtxt<'mx> {
    pub fn func(&self, func: CFuncKind<'mx>) -> &'mx CFuncKind<'mx> {
        self.arena().alloc(func)
    }
}

impl Printer {
    pub fn print_func_decl(&mut self, func: CFunc) {
        self.print_signature(func);
        self.word(";");
    }

    pub fn print_func(&mut self, func: CFunc) {
        self.ibox(0, |this| {
            this.print_signature(func);
            this.softbreak(); // I don't know how to avoid a newline here
            this.print_compound(&func.0.body.borrow());
        })
    }

    fn print_signature(&mut self, func: CFunc) {
        self.ibox(0, |this| {
            this.print_ty(func.0.ty);
            this.softbreak();
            this.word(func.0.name.to_string());

            this.valign_delim(("(", ")"), |this| {
                this.seperated(",", &func.0.params, |this, (ty, name)| {
                    this.ibox(0, |this| {
                        this.print_ty(*ty);
                        this.softbreak();
                        this.print_value(*name);
                    })
                })
            });
        });
    }
}
