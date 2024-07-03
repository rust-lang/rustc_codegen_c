//! C Module
//!
//! A module is a in-memory representation of a C file.
//!
//! The structure is derived from clang's AST.

use std::fmt::Display;

#[derive(Default)]
pub struct Module {
    pub includes: Vec<String>,
    pub decls: Vec<CDecl>,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for include in &self.includes {
            writeln!(f, "#include <{}>", include)?;
        }
        for decl in &self.decls {
            writeln!(f, "{}", decl)?;
        }
        Ok(())
    }
}

// TODO: use rustc's memory arena
// TODO: maybe split expr from stmt?

pub enum CDecl {
    Typedef { name: String, ty: CType },
    Record { name: String, fields: Vec<CDecl> },
    Field { name: String, ty: CType },
    Enum { name: String, values: Vec<CEnumConstant> },
    Function { name: String, ty: CType, params: Vec<CParamVar>, body: Option<CStmt> },
    Var { name: String, ty: CType, init: Option<CStmt> },
    Raw(String),
}

impl Display for CDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CDecl::Typedef { name, ty } => write!(f, "typedef {} {};", name, ty),
            CDecl::Record { name, fields } => {
                writeln!(f, "struct {} {{ ", name)?;
                for field in fields {
                    writeln!(f, "{}", field)?;
                }
                writeln!(f, "}};")
            }
            CDecl::Field { name, ty } => write!(f, "{} {}", name, ty),
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
            CDecl::Function { name, ty, params, body } => {
                write!(f, "{} {}(", ty, name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")?;
                if let Some(body) = body {
                    write!(f, "{}", body)
                } else {
                    write!(f, ";")
                }
            }
            CDecl::Var { name, ty, init } => {
                write!(f, "{} {}", ty, name)?;
                if let Some(init) = init {
                    write!(f, " = {}", init)?;
                }
                write!(f, ";")
            }
            CDecl::Raw(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
pub enum CType {
    Builtin(String),
    Pointer(Box<CType>),
    Record(String),
    Array(Box<CType>, usize),
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CType::Builtin(ty) => write!(f, "{}", ty),
            CType::Pointer(ty) => write!(f, "{}*", ty),
            CType::Record(ty) => write!(f, "struct {}", ty),
            CType::Array(ty, size) => write!(f, "{}[{}]", ty, size),
        }
    }
}

pub struct CEnumConstant {
    pub name: String,
    pub value: Option<CStmt>,
}

impl Display for CEnumConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }
        Ok(())
    }
}

pub struct CParamVar {
    pub name: Option<String>,
    pub ty: CType,
}

impl Display for CParamVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{} {}", self.ty, name)
        } else {
            write!(f, "{}", self.ty)
        }
    }
}

pub enum CStmt {
    Compound(Vec<CStmt>),
    If { cond: Box<CExpr>, then_: Box<CStmt>, else_: Option<Box<CStmt>> },
    Return(Option<Box<CExpr>>),
    Decl(Box<CDecl>),
    Expr(CExpr),
}

impl Display for CStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CStmt::Compound(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "{}", stmt)?;
                }
                write!(f, "}}")
            }
            CStmt::If { cond, then_, else_ } => {
                writeln!(f, "if ({}) {{", cond)?;
                writeln!(f, "{}", then_)?;
                if let Some(else_) = else_ {
                    writeln!(f, "}} else {{")?;
                    writeln!(f, "{}", else_)?;
                }
                write!(f, "}}")
            }
            CStmt::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {}", expr)?;
                }
                write!(f, ";")
            }
            CStmt::Decl(decl) => write!(f, "{}", decl),
            CStmt::Expr(expr) => write!(f, "{};", expr),
        }
    }
}

pub enum CExpr {
    Literal(String),
    DeclRef { name: String },
    BinaryOperator { lhs: Box<CExpr>, rhs: Box<CExpr>, op: String },
    Call { callee: Box<CExpr>, args: Vec<CExpr> },
    Member { expr: Box<CExpr>, arrow: bool, field: String },
}

impl Display for CExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CExpr::Literal(lit) => write!(f, "{}", lit),
            CExpr::DeclRef { name } => write!(f, "{}", name),
            CExpr::BinaryOperator { lhs, rhs, op } => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
            CExpr::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CExpr::Member { expr, arrow, field } => {
                write!(f, "{}", expr)?;
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
