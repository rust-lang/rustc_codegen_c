use std::cell::{Cell, RefCell};

use rustc_data_structures::intern::Interned;

use crate::expr::CValue;
use crate::pretty::{Print, PrinterCtx};
use crate::stmt::{print_compound, CStmt};
use crate::ty::CTy;
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

impl Print for CFunc<'_> {
    fn print_to(&self, ctx: &mut PrinterCtx) {
        ctx.ibox(0, |ctx| {
            print_signature(*self, ctx);
            ctx.softbreak(); // I don't know how to avoid a newline here
            print_compound(&self.0.body.borrow(), ctx);
        })
    }
}

pub(crate) fn print_func_decl(func: CFunc, ctx: &mut PrinterCtx) {
    print_signature(func, ctx);
    ctx.word(";");
}

fn print_signature(func: CFunc, ctx: &mut PrinterCtx) {
    ctx.ibox(0, |ctx| {
        func.0.ty.print_to(ctx);
        ctx.softbreak();
        ctx.word(func.0.name.to_string());

        ctx.valign_delim(("(", ")"), |ctx| {
            ctx.seperated(",", &func.0.params, |ctx, (ty, name)| {
                ctx.ibox(0, |ctx| {
                    ty.print_to(ctx);
                    ctx.softbreak();
                    name.print_to(ctx);
                })
            })
        });
    });
}
