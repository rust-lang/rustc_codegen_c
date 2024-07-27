use rustc_data_structures::intern::Interned;
use rustc_type_ir::{IntTy, UintTy};

use crate::{pretty::Printer, ModuleCtxt};

/// C types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CTy<'mx> {
    Primitive(CPTy),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTyKind<'mx> {
    Pointer(CTy<'mx>),
    // Record(String),
    // Array(CType<'mx>, usize),
}

impl<'mx> ModuleCtxt<'mx> {
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

impl Printer {
    pub fn print_ty(&mut self, ty: CTy<'_>) {
        match ty {
            CTy::Primitive(ty) => self.word(ty.to_str()),
            CTy::Ref(ty) => self.print_ty_kind(ty.0),
        }
    }

    fn print_ty_kind(&mut self, ty: &CTyKind<'_>) {
        match ty {
            CTyKind::Pointer(ty) => {
                self.word("*");
                self.print_ty(*ty);
            }
        }
    }
}