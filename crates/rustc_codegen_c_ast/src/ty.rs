//! This module defines the AST nodes for C types.

use rustc_data_structures::intern::Interned;
use rustc_type_ir::{IntTy, UintTy};

use crate::expr::CValue;
use crate::pretty::{Print, PrinterCtx};
use crate::ModuleCtx;

/// C types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CTy<'mx> {
    /// A primitive C type.
    Primitive(CPTy),
    /// A non-primitive C type, e.g. a pointer type.
    Ref(Interned<'mx, CTyKind<'mx>>),
}

/// C primitive types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CPTy {
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

impl CPTy {
    /// Whether the type is a signed integer.
    pub fn is_signed(self) -> bool {
        match self {
            CPTy::Isize | CPTy::I8 | CPTy::I16 | CPTy::I32 | CPTy::I64 => true,
            CPTy::Usize | CPTy::U8 | CPTy::U16 | CPTy::U32 | CPTy::U64 => false,
        }
    }

    /// The unsigned version of this type.
    ///
    /// ## Panic
    ///
    /// Panics if the type is not a signed integer.
    pub fn to_unsigned(self) -> CPTy {
        match self {
            CPTy::Isize => CPTy::Usize,
            CPTy::I8 => CPTy::U8,
            CPTy::I16 => CPTy::U16,
            CPTy::I32 => CPTy::U32,
            CPTy::I64 => CPTy::U64,
            _ => unreachable!(),
        }
    }

    /// Get the corresponding C type name.
    pub fn to_str(self) -> &'static str {
        match self {
            CPTy::Isize => "size_t",
            CPTy::I8 => "int8_t",
            CPTy::I16 => "int16_t",
            CPTy::I32 => "int32_t",
            CPTy::I64 => "int64_t",
            CPTy::Usize => "size_t",
            CPTy::U8 => "uint8_t",
            CPTy::U16 => "uint16_t",
            CPTy::U32 => "uint32_t",
            CPTy::U64 => "uint64_t",
        }
    }

    /// The maximum value of this type. From `<stdint.h>`.
    pub fn max_value(self) -> &'static str {
        match self {
            CPTy::Isize => "SIZE_MAX",
            CPTy::I8 => "INT8_MAX",
            CPTy::I16 => "INT16_MAX",
            CPTy::I32 => "INT32_MAX",
            CPTy::I64 => "INT64_MAX",
            CPTy::Usize => "SIZE_MAX",
            CPTy::U8 => "UINT8_MAX",
            CPTy::U16 => "UINT16_MAX",
            CPTy::U32 => "UINT32_MAX",
            CPTy::U64 => "UINT64_MAX",
        }
    }
}

/// Complex C types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTyKind<'mx> {
    /// A pointer type.
    Pointer(CTy<'mx>),
    // Record(String),
    // Array(CType<'mx>, usize),
}

impl<'mx> ModuleCtx<'mx> {
    /// Get the type of an signed integer
    pub fn get_int_type(&self, int: IntTy) -> CTy<'mx> {
        match int {
            IntTy::Isize => CTy::Primitive(CPTy::Isize),
            IntTy::I8 => CTy::Primitive(CPTy::I8),
            IntTy::I16 => CTy::Primitive(CPTy::I16),
            IntTy::I32 => CTy::Primitive(CPTy::I32),
            IntTy::I64 => CTy::Primitive(CPTy::I64),
            IntTy::I128 => unimplemented!("i128 not supported yet"),
        }
    }

    /// Get the type of an unsigned integer
    pub fn get_uint_type(&self, uint: UintTy) -> CTy<'mx> {
        match uint {
            UintTy::Usize => CTy::Primitive(CPTy::Usize),
            UintTy::U8 => CTy::Primitive(CPTy::U8),
            UintTy::U16 => CTy::Primitive(CPTy::U16),
            UintTy::U32 => CTy::Primitive(CPTy::U32),
            UintTy::U64 => CTy::Primitive(CPTy::U64),
            UintTy::U128 => unimplemented!("u128 not supported yet"),
        }
    }
}

/// Print a C declarator.
///
/// A declarator is a type with an optional identifier and pointer indirections,
/// e.g. `int *x`.
///
/// This function is necessary because the C declarator syntax is quite complex
/// when the type becomes more complex, e.g. `int (*x)[10]`.
///
/// When `val` is `None`, this prints an abstract declarator, or in other words,
/// a standalone type without an identifier.
pub(crate) fn print_declarator(mut ty: CTy, val: Option<CValue>, ctx: &mut PrinterCtx) {
    enum DeclaratorPart<'mx> {
        Ident(Option<CValue<'mx>>),
        Ptr,
    }

    impl Print for DeclaratorPart<'_> {
        fn print_to(&self, ctx: &mut PrinterCtx) {
            match self {
                DeclaratorPart::Ident(val) => {
                    if let &Some(val) = val {
                        val.print_to(ctx);
                    }
                }
                DeclaratorPart::Ptr => {
                    ctx.word("*");
                }
            }
        }
    }

    let mut decl_parts = std::collections::VecDeque::new();
    decl_parts.push_front(DeclaratorPart::Ident(val));
    while let CTy::Ref(kind) = ty {
        match kind.0 {
            CTyKind::Pointer(_) => decl_parts.push_front(DeclaratorPart::Ptr),
        }
        ty = match kind.0 {
            CTyKind::Pointer(ty) => *ty,
        };
    }

    let CTy::Primitive(base) = ty else { unreachable!() };
    ctx.word(base.to_str());
    if val.is_some() {
        ctx.nbsp();
    }
    for part in decl_parts {
        part.print_to(ctx);
    }
}
