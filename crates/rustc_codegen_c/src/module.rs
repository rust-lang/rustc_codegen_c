//! C Module
//!
//! A module is a in-memory representation of a C file.
//!
//! The structure is derived from clang's AST.

use std::fmt::{Display, Formatter};

use rustc_type_ir::{IntTy, UintTy};

use crate::utils::sharded_slab::{Entry, Id, ShardedSlab};

/// Needed helper functions
const HELPER: &str = include_str!("./helper.h");

pub struct ModuleContext {
    types: ShardedSlab<CTypeKind>,
    pub module: Module,
}

impl Default for ModuleContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleContext {
    pub fn new() -> Self {
        Self { types: ShardedSlab::default(), module: Module::new() }
    }

    /// Get the type
    pub fn get_type(&self, id: Id<CTypeKind>) -> Entry<CTypeKind> {
        self.types.get(id).unwrap()
    }

    /// Get the type of an signed integer
    pub fn get_int_type(&self, int: IntTy) -> CType {
        match int {
            IntTy::Isize => CType::Primitive(CPrimitiveType::Isize),
            IntTy::I8 => CType::Primitive(CPrimitiveType::I8),
            IntTy::I16 => CType::Primitive(CPrimitiveType::I16),
            IntTy::I32 => CType::Primitive(CPrimitiveType::I32),
            IntTy::I64 => CType::Primitive(CPrimitiveType::I64),
            IntTy::I128 => unimplemented!("i128 not supported yet"),
        }
    }

    /// Get the type of an unsigned integer
    pub fn get_uint_type(&self, uint: UintTy) -> CType {
        match uint {
            UintTy::Usize => CType::Primitive(CPrimitiveType::Usize),
            UintTy::U8 => CType::Primitive(CPrimitiveType::U8),
            UintTy::U16 => CType::Primitive(CPrimitiveType::U16),
            UintTy::U32 => CType::Primitive(CPrimitiveType::U32),
            UintTy::U64 => CType::Primitive(CPrimitiveType::U64),
            UintTy::U128 => unimplemented!("u128 not supported yet"),
        }
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
        writeln!(f, "{}", HELPER)?;
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
    Var { name: CValue, ty: CType, init: Option<CExpr> },
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
                write!(f, "typedef {} {};", name, ty.display(ctx))
            }
            CDecl::Record { name, fields } => {
                writeln!(f, "struct {} {{ ", name)?;
                for field in fields {
                    writeln!(f, "{}", field.display(ctx))?;
                }
                writeln!(f, "}};")
            }
            CDecl::Field { name, ty } => write!(f, "{} {}", name, ty.display(ctx)),
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
                write!(f, "{} {}(", ty.display(ctx), name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.display(ctx))?;
                }
                write!(f, ");")
            }
            CDecl::Function(func) => write!(f, "{}", func.display(ctx)),
            CDecl::Var { name, ty, init } => {
                write!(f, "{} {}", ty.display(ctx), name)?;
                if let Some(init) = init {
                    write!(f, " = {}", init.display(ctx))?;
                }
                write!(f, ";")
            }
            CDecl::Raw(s) => write!(f, "{}", s),
        }
    }
}

/// A C type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CType {
    Primitive(CPrimitiveType),
    Ref(Id<CTypeKind>),
}

impl CType {
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + 'a {
        struct CTypeDisplay<'a>(&'a CType, &'a ModuleContext);
        impl<'a> Display for CTypeDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CTypeDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        match self {
            CType::Primitive(ty) => write!(f, "{}", ty),
            CType::Ref(id) => write!(f, "{}", ctx.get_type(*id).display(ctx)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CPrimitiveType {
    Isize,
    I8,
    I16,
    I32,
    I64,
    Usize,
    U8,
    U16,
    U32,
    U64,
}

impl CPrimitiveType {
    /// Whether the type is a signed integer.
    pub fn is_signed(self) -> bool {
        match self {
            CPrimitiveType::Isize
            | CPrimitiveType::I8
            | CPrimitiveType::I16
            | CPrimitiveType::I32
            | CPrimitiveType::I64 => true,
            CPrimitiveType::Usize
            | CPrimitiveType::U8
            | CPrimitiveType::U16
            | CPrimitiveType::U32
            | CPrimitiveType::U64 => false,
        }
    }

    /// The unsigned version of this type.
    ///
    /// ## Panic
    ///
    /// Panics if the type is not a signed integer.
    pub fn to_unsigned(self) -> CPrimitiveType {
        match self {
            CPrimitiveType::Isize => CPrimitiveType::Usize,
            CPrimitiveType::I8 => CPrimitiveType::U8,
            CPrimitiveType::I16 => CPrimitiveType::U16,
            CPrimitiveType::I32 => CPrimitiveType::U32,
            CPrimitiveType::I64 => CPrimitiveType::U64,
            _ => unreachable!(),
        }
    }

    /// The maximum value of this type. From `<stdint.h>`.
    pub fn max_value(self) -> &'static str {
        match self {
            CPrimitiveType::Isize => "SIZE_MAX",
            CPrimitiveType::I8 => "INT8_MAX",
            CPrimitiveType::I16 => "INT16_MAX",
            CPrimitiveType::I32 => "INT32_MAX",
            CPrimitiveType::I64 => "INT64_MAX",
            CPrimitiveType::Usize => "SIZE_MAX",
            CPrimitiveType::U8 => "UINT8_MAX",
            CPrimitiveType::U16 => "UINT16_MAX",
            CPrimitiveType::U32 => "UINT32_MAX",
            CPrimitiveType::U64 => "UINT64_MAX",
        }
    }
}

impl Display for CPrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CPrimitiveType::Isize => write!(f, "ssize_t"),
            CPrimitiveType::I8 => write!(f, "int8_t"),
            CPrimitiveType::I16 => write!(f, "int16_t"),
            CPrimitiveType::I32 => write!(f, "int32_t"),
            CPrimitiveType::I64 => write!(f, "int64_t"),
            CPrimitiveType::Usize => write!(f, "size_t"),
            CPrimitiveType::U8 => write!(f, "uint8_t"),
            CPrimitiveType::U16 => write!(f, "uint16_t"),
            CPrimitiveType::U32 => write!(f, "uint32_t"),
            CPrimitiveType::U64 => write!(f, "uint64_t"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTypeKind {
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
            CTypeKind::Pointer(ty) => write!(f, "{}*", ty.display(ctx)),
            CTypeKind::Record(ty) => write!(f, "struct {}", ty),
            CTypeKind::Array(ty, size) => write!(f, "{}[{}]", ty.display(ctx), size),
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
    Local(usize),
}

impl Display for CValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CValue::Scalar(scalar) => write!(f, "{}", scalar),
            CValue::Local(index) => write!(f, "_{}", index),
        }
    }
}

pub struct CFunction {
    pub name: String,
    pub ty: CType,
    pub params: Vec<(CType, CValue)>,
    pub body: Vec<CStmt>,
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
        write!(f, "{} {}(", self.ty.display(ctx), self.name)?;
        for (i, (ty, param)) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", ty.display(ctx), param)?;
        }
        writeln!(f, ") {{")?;
        for stmt in &self.body {
            writeln!(f, "{}", stmt.display(ctx))?;
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
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + '_ {
        struct CStmtDisplay<'a>(&'a CStmt, &'a ModuleContext);
        impl<'a> Display for CStmtDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CStmtDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        match self {
            CStmt::Compound(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "{}", stmt.display(ctx))?;
                }
                write!(f, "}}")
            }
            CStmt::If { cond, then_br: then_, else_br: else_ } => {
                writeln!(f, "if ({}) {{", cond.display(ctx))?;
                writeln!(f, "{}", then_.display(ctx))?;
                if let Some(else_) = else_ {
                    writeln!(f, "}} else {{")?;
                    writeln!(f, "{}", else_.display(ctx))?;
                }
                write!(f, "}}")
            }
            CStmt::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {}", expr.display(ctx))?;
                }
                write!(f, ";")
            }
            CStmt::Decl(decl) => write!(f, "{}", decl.display(ctx)),
            CStmt::Expr(expr) => write!(f, "{};", expr.display(ctx)),
        }
    }
}

pub enum CExpr {
    Raw(String),
    Value(CValue),
    BinaryOperator { lhs: Box<CExpr>, rhs: Box<CExpr>, op: String },
    Cast { ty: CType, expr: Box<CExpr> },
    Call { callee: Box<CExpr>, args: Vec<CExpr> },
    Member { expr: Box<CExpr>, arrow: bool, field: String },
}

impl CExpr {
    pub fn display<'a>(&'a self, ctx: &'a ModuleContext) -> impl Display + '_ {
        struct CExprDisplay<'a>(&'a CExpr, &'a ModuleContext);
        impl<'a> Display for CExprDisplay<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_with(f, self.1)
            }
        }
        CExprDisplay(self, ctx)
    }

    fn fmt_with(&self, f: &mut Formatter<'_>, ctx: &ModuleContext) -> std::fmt::Result {
        match self {
            CExpr::Raw(lit) => write!(f, "{}", lit),
            CExpr::Value(val) => write!(f, "{}", val),
            CExpr::BinaryOperator { lhs, rhs, op } => {
                write!(f, "({} {} {})", lhs.display(ctx), op, rhs.display(ctx))
            }
            CExpr::Cast { ty, expr } => {
                write!(f, "(({}) {})", ty.display(ctx), expr.display(ctx))
            }
            CExpr::Call { callee, args } => {
                write!(f, "{}(", callee.display(ctx))?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.display(ctx))?;
                }
                write!(f, ")")
            }
            CExpr::Member { expr, arrow, field } => {
                write!(f, "{}", expr.display(ctx))?;
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

#[macro_export]
macro_rules! c_expr {
    ($expr: expr) => {
        CExpr::Raw($expr.into()).into()
    };
}
