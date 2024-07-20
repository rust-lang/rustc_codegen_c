//! C Module
//!
//! A module is a in-memory representation of a C file.
//!
//! The structure is derived from clang's AST.

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use rustc_type_ir::{IntTy, UintTy};

use crate::utils::sharded_slab::{Entry, Id, ShardedSlab};

/// Rust's primitive types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PimitiveType {
    Int(IntTy),
    Uint(UintTy),
}

pub struct ModuleContext {
    types: ShardedSlab<CTypeKind>,
    primitive_types: RwLock<FxHashMap<PimitiveType, CType>>,
    pub module: Module,
}

impl Default for ModuleContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleContext {
    pub fn new() -> Self {
        Self {
            types: ShardedSlab::default(),
            primitive_types: RwLock::new(FxHashMap::default()),
            module: Module::new(),
        }
    }

    /// Get the type
    pub fn ty(&self, id: CType) -> Entry<CTypeKind> {
        self.types.get(id).unwrap()
    }

    /// Get the type of an signed integer
    pub fn get_int_type(&self, int: IntTy) -> CType {
        if let Some(ty) = self.primitive_types.read().get(&PimitiveType::Int(int)) {
            return *ty;
        }

        let tykind = match int {
            IntTy::Isize => CTypeKind::Builtin("ssize_t".to_string()),
            IntTy::I8 => CTypeKind::Builtin("int8_t".to_string()),
            IntTy::I16 => CTypeKind::Builtin("int16_t".to_string()),
            IntTy::I32 => CTypeKind::Builtin("int32_t".to_string()),
            IntTy::I64 => CTypeKind::Builtin("int64_t".to_string()),
            IntTy::I128 => todo!(),
        };
        let ty = self.types.insert(tykind);
        self.primitive_types
            .write()
            .insert(PimitiveType::Int(int), ty);
        ty
    }

    /// Get the type of an unsigned integer
    pub fn get_uint_type(&self, uint: UintTy) -> CType {
        if let Some(ty) = self.primitive_types.read().get(&PimitiveType::Uint(uint)) {
            return *ty;
        }

        let tykind = match uint {
            UintTy::Usize => CTypeKind::Builtin("size_t".to_string()),
            UintTy::U8 => CTypeKind::Builtin("uint8_t".to_string()),
            UintTy::U16 => CTypeKind::Builtin("uint16_t".to_string()),
            UintTy::U32 => CTypeKind::Builtin("uint32_t".to_string()),
            UintTy::U64 => CTypeKind::Builtin("uint64_t".to_string()),
            UintTy::U128 => todo!(),
        };
        let ty = self.types.insert(tykind);
        self.primitive_types
            .write()
            .insert(PimitiveType::Uint(uint), ty);
        ty
    }
}

impl Display for ModuleContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.module.fmt_with(f, self)
    }
}

pub struct Module {
    pub includes: Vec<String>,
    pub decls: Vec<CDecl>,
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

impl Module {
    pub fn new() -> Self {
        let includes = vec!["stdint.h".to_string()];
        let decls = vec![];
        Self { includes, decls }
    }

    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + 'a {
        struct ModuleDisplay<'a>(&'a Module, &'a ModuleContext);
        impl<'a> Display for ModuleDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        ModuleDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        for include in &self.includes {
            writeln!(f, "#include <{}>", include)?;
        }
        for decl in &self.decls {
            writeln!(f, "{}", decl.display(ctx))?;
        }
        Ok(())
    }
}

pub enum CDecl {
    Typedef { name: String, ty: CType },
    Record { name: String, fields: Vec<CDecl> },
    Field { name: String, ty: CType },
    Enum { name: String, values: Vec<CEnumConstant> },
    FunctionDecl { name: String, ty: CType, params: Vec<CType> },
    Function(CFunction),
    // Var { name: String, ty: CType, init: Option<CStmt> },
    Raw(String),
}

impl CDecl {
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + 'a {
        struct CDeclDisplay<'a>(&'a CDecl, &'a ModuleContext);
        impl<'a> Display for CDeclDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CDeclDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        match self {
            CDecl::Typedef { name, ty } => {
                write!(f, "typedef {} {};", name, ctx.ty(*ty).display(ctx))
            }
            CDecl::Record { name, fields } => {
                writeln!(f, "struct {} {{ ", name)?;
                for field in fields {
                    writeln!(f, "{}", field.display(ctx))?;
                }
                writeln!(f, "}};")
            }
            CDecl::Field { name, ty } => write!(f, "{} {}", name, ctx.ty(*ty).display(ctx)),
            CDecl::Enum { name, values } => {
                writeln!(f, "enum {} {{ ", name)?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    writeln!(f, "{}", value)?;
                }
                writeln!(f, "}};")
            }
            CDecl::FunctionDecl { name, ty, params } => {
                write!(f, "{} {}(", ctx.ty(*ty).display(ctx), name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ctx.ty(*param).display(ctx))?;
                }
                write!(f, ");")
            }
            CDecl::Function(func) => write!(f, "{}", func.display(ctx)),
            // CDecl::Var { name, ty, init } => {
            //     write!(f, "{} {}", ty, name)?;
            //     if let Some(init) = init {
            //         write!(f, " = {}", init)?;
            //     }
            //     write!(f, ";")
            // }
            CDecl::Raw(s) => write!(f, "{}", s),
        }
    }
}

pub type CType = Id<CTypeKind>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTypeKind {
    Builtin(String),
    Pointer(CType),
    Record(String),
    Array(CType, usize),
}

impl CTypeKind {
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + 'a {
        struct DisplayCTypeKind<'a>(&'a CTypeKind, &'a ModuleContext);
        impl<'a> Display for DisplayCTypeKind<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        DisplayCTypeKind(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        match self {
            CTypeKind::Builtin(ty) => write!(f, "{}", ty),
            CTypeKind::Pointer(ty) => write!(f, "{}*", ctx.ty(*ty).display(ctx)),
            CTypeKind::Record(ty) => write!(f, "struct {}", ty),
            CTypeKind::Array(ty, size) => write!(f, "{}[{}]", ctx.ty(*ty).display(ctx), size),
        }
    }
}

pub struct CEnumConstant {
    pub name: String,
    // pub value: Option<CStmt>,
}

impl Display for CEnumConstant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        // if let Some(value) = &self.value {
        //     write!(f, " = {}", value)?;
        // }
        Ok(())
    }
}

/// Values of C variable, parameters and scalars
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CValue {
    Scalar(i128),
    Var(usize),
}

pub struct CFunction {
    pub name: String,
    pub ty: CType,
    pub params: Vec<(CType, CValue)>,
    pub body: Vec<CStmt>,
    pub var_names: FxHashMap<CValue, String>,
}

impl CFunction {
    pub fn get_var_name(&self, var: CValue) -> Cow<str> {
        if let Some(name) = self.var_names.get(&var) {
            Cow::Borrowed(name)
        } else {
            Cow::Owned(match var {
                CValue::Scalar(scalar) => format!("{}", scalar),
                CValue::Var(index) => format!("_{}", index),
            })
        }
    }
}

impl CFunction {
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + '_ {
        struct CFunctionDisplay<'a>(&'a CFunction, &'a ModuleContext);
        impl<'a> Display for CFunctionDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CFunctionDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        write!(f, "{} {}(", ctx.ty(self.ty).display(ctx), self.name)?;
        for (i, (ty, param)) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", ctx.ty(*ty).display(ctx), self.get_var_name(*param))?;
        }
        write!(f, ") {{")?;
        for stmt in &self.body {
            writeln!(f, "{}", stmt.display(self, ctx))?;
        }
        write!(f, "}}")
    }
}

pub enum CStmt {
    Compound(Vec<CStmt>),
    If { cond: Box<CExpr>, then_br: Box<CStmt>, else_br: Option<Box<CStmt>> },
    Return(Option<Box<CExpr>>),
    Decl(Box<CDecl>),
    Expr(CExpr),
}

impl CStmt {
    pub fn display<'a>(&'a self, fun: &'a CFunction, ctx: &'a ModuleContext) -> impl Display + '_ {
        struct CStmtDisplay<'a>(&'a CStmt, &'a CFunction, &'a ModuleContext);
        impl<'a> Display for CStmtDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1, self.2)
            }
        }
        CStmtDisplay(self, fun, ctx)
    }

    fn fmt_with(
        &self,
        f: &mut Formatter<'_>,
        fun: &CFunction,
        ctx: &ModuleContext,
    ) -> std::fmt::Result {
        match self {
            CStmt::Compound(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "{}", stmt.display(fun, ctx))?;
                }
                write!(f, "}}")
            }
            CStmt::If { cond, then_br: then_, else_br: else_ } => {
                writeln!(f, "if ({}) {{", cond.display(fun))?;
                writeln!(f, "{}", then_.display(fun, ctx))?;
                if let Some(else_) = else_ {
                    writeln!(f, "}} else {{")?;
                    writeln!(f, "{}", else_.display(fun, ctx))?;
                }
                write!(f, "}}")
            }
            CStmt::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {}", expr.display(fun))?;
                }
                write!(f, ";")
            }
            CStmt::Decl(decl) => write!(f, "{}", decl.display(ctx)),
            CStmt::Expr(expr) => write!(f, "{};", expr.display(fun)),
        }
    }
}

pub enum CExpr {
    Literal(String),
    Value(CValue),
    BinaryOperator { lhs: Box<CExpr>, rhs: Box<CExpr>, op: String },
    Call { callee: Box<CExpr>, args: Vec<CExpr> },
    Member { expr: Box<CExpr>, arrow: bool, field: String },
}

impl CExpr {
    pub fn display<'a>(&'a self, fun: &'a CFunction) -> impl Display + '_ {
        struct CExprDisplay<'a>(&'a CExpr, &'a CFunction);
        impl<'a> Display for CExprDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CExprDisplay(self, fun)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, fun: &CFunction) -> std::fmt::Result {
        match self {
            CExpr::Literal(lit) => write!(f, "{}", lit),
            CExpr::Value(val) => {
                let name = fun.get_var_name(*val);
                write!(f, "{}", name)
            }
            CExpr::BinaryOperator { lhs, rhs, op } => {
                write!(f, "({} {} {})", lhs.display(fun), op, rhs.display(fun))
            }
            CExpr::Call { callee, args } => {
                write!(f, "{}(", callee.display(fun))?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.display(fun))?;
                }
                write!(f, ")")
            }
            CExpr::Member { expr, arrow, field } => {
                write!(f, "{}", expr.display(fun))?;
                if *arrow {
                    write!(f, "->")?;
                } else {
                    write!(f, ".")?;
                }
                write!(f, "{}", field)
            }
        }
    }
}
