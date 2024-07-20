//! C Module
//!
//! A module is a in-memory representation of a C file.
//!
//! The structure is derived from clang's AST.

use std::borrow::Cow;
use std::fmt::Display;

use rustc_hash::FxHashMap;

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
            CDecl::FunctionDecl { name, ty, params } => {
                write!(f, "{} {}(", ty, name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ");")
            }
            CDecl::Function(func) => write!(f, "{}", func),
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
    // pub value: Option<CStmt>,
}

impl Display for CEnumConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Display for CFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}(", self.ty, self.name)?;
        for (i, (ty, param)) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", ty, self.get_var_name(*param))?;
        }
        write!(f, ") {{")?;
        for stmt in &self.body {
            writeln!(f, "{}", stmt.display(self))?;
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
    pub fn display<'a>(&'a self, function: &'a CFunction) -> impl Display + '_ {
        struct CStmtDisplay<'a> {
            stmt: &'a CStmt,
            function: &'a CFunction,
        }

        impl<'a> Display for CStmtDisplay<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.stmt.fmt_with(f, self.function)
            }
        }

        CStmtDisplay { stmt: self, function }
    }

    fn fmt_with(&self, f: &mut std::fmt::Formatter<'_>, function: &CFunction) -> std::fmt::Result {
        match self {
            CStmt::Compound(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "{}", stmt.display(function))?;
                }
                write!(f, "}}")
            }
            CStmt::If { cond, then_br: then_, else_br: else_ } => {
                writeln!(f, "if ({}) {{", cond.display(function))?;
                writeln!(f, "{}", then_.display(function))?;
                if let Some(else_) = else_ {
                    writeln!(f, "}} else {{")?;
                    writeln!(f, "{}", else_.display(function))?;
                }
                write!(f, "}}")
            }
            CStmt::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {}", expr.display(function))?;
                }
                write!(f, ";")
            }
            CStmt::Decl(decl) => write!(f, "{}", decl),
            CStmt::Expr(expr) => write!(f, "{};", expr.display(function)),
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
    pub fn display<'a>(&'a self, function: &'a CFunction) -> impl Display + '_ {
        struct CExprDisplay<'a> {
            expr: &'a CExpr,
            function: &'a CFunction,
        }

        impl<'a> Display for CExprDisplay<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.expr.fmt_with(f, self.function)
            }
        }

        CExprDisplay { expr: self, function }
    }

    fn fmt_with(&self, f: &mut std::fmt::Formatter<'_>, function: &CFunction) -> std::fmt::Result {
        match self {
            CExpr::Literal(lit) => write!(f, "{}", lit),
            CExpr::Value(val) => {
                let name = function.get_var_name(*val);
                write!(f, "{}", name)
            }
            CExpr::BinaryOperator { lhs, rhs, op } => {
                write!(f, "({} {} {})", lhs.display(function), op, rhs.display(function))
            }
            CExpr::Call { callee, args } => {
                write!(f, "{}(", callee.display(function))?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.display(function))?;
                }
                write!(f, ")")
            }
            CExpr::Member { expr, arrow, field } => {
                write!(f, "{}", expr.display(function))?;
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
