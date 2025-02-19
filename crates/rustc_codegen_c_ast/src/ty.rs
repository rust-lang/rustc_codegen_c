//! This module defines the AST nodes for C types.

use rustc_data_structures::intern::Interned;
use rustc_type_ir::{IntTy, UintTy};

use crate::expr::CValue;
use crate::pretty::{Print, PrinterCtx};
use crate::ModuleCtx;

/// C types.
///
/// A C type is either a primitive type or a complex type. Primitive types are
/// the basic types like `int` and `char`, while complex types are types that
/// are built from primitive types, like pointers and arrays.
///
/// Complex types are always interned, and thus should be unique in a specific
/// context. See [`CTyKind`] for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CTy<'mx> {
    /// The C `void` type.
    Void,
    /// The C boolean type.
    Bool,
    /// The C `char` type.
    Char,
    /// A signed integer type.
    Int(CIntTy),
    /// An unsigned integer type.
    UInt(CUintTy),
    /// A non-primitive C type, e.g. a pointer type.
    ///
    /// This is an interned reference to a complex type.
    Ref(Interned<'mx, CTyKind<'mx>>),
}

impl<'mx> CTy<'mx> {
    /// Whether the type is a signed integer.
    pub fn is_signed(self) -> bool {
        matches!(self, CTy::Int(_))
    }

    /// The unsigned version of this type.
    ///
    /// ## Panic
    ///
    /// Panics if the type is not a signed integer.
    pub fn to_unsigned(self) -> Self {
        match self {
            CTy::Int(ty) => CTy::UInt(ty.to_unsigned()),
            _ => unreachable!(),
        }
    }

    /// Get the corresponding C type name.
    ///
    /// This function should be only used for primitive types.
    ///
    /// ## Panic
    ///
    /// Panics if the type is not a primitive type.
    pub fn to_str(self) -> &'static str {
        match self {
            CTy::Void => "void",
            CTy::Bool => "_Bool",
            CTy::Char => "char",
            CTy::Int(ty) => ty.to_str(),
            CTy::UInt(ty) => ty.to_str(),
            CTy::Ref(_) => unreachable!(),
        }
    }

    /// The maximum value of this type. From `<stdint.h>`.
    ///
    /// This function should be only used for integer types (signed or unsigned).
    ///
    /// ## Panic
    ///
    /// Panics if the type is not an integer type.
    pub fn max_value(self) -> &'static str {
        match self {
            CTy::Int(ty) => ty.max_value(),
            CTy::UInt(ty) => ty.max_value(),
            _ => unreachable!(),
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, CTy::Void | CTy::Bool | CTy::Char | CTy::Int(_) | CTy::UInt(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, CTy::Ref(Interned(CTyKind::Array(_, _), _)))
    }
}

/// C primitive types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CIntTy {
    Isize,
    I8,
    I16,
    I32,
    I64,
}

impl CIntTy {
    /// Get the unsigned version of this type.
    pub fn to_unsigned(self) -> CUintTy {
        match self {
            CIntTy::Isize => CUintTy::Usize,
            CIntTy::I8 => CUintTy::U8,
            CIntTy::I16 => CUintTy::U16,
            CIntTy::I32 => CUintTy::U32,
            CIntTy::I64 => CUintTy::U64,
        }
    }

    /// Get the corresponding C type name.
    pub fn to_str(self) -> &'static str {
        match self {
            CIntTy::Isize => "size_t",
            CIntTy::I8 => "int8_t",
            CIntTy::I16 => "int16_t",
            CIntTy::I32 => "int32_t",
            CIntTy::I64 => "int64_t",
        }
    }

    /// The maximum value of this type. From `<stdint.h>`.
    pub fn max_value(self) -> &'static str {
        match self {
            CIntTy::Isize => "SIZE_MAX",
            CIntTy::I8 => "INT8_MAX",
            CIntTy::I16 => "INT16_MAX",
            CIntTy::I32 => "INT32_MAX",
            CIntTy::I64 => "INT64_MAX",
        }
    }
}

/// C primitive types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CUintTy {
    Usize,
    U8,
    U16,
    U32,
    U64,
}

impl CUintTy {
    /// Get the corresponding C type name.
    pub fn to_str(self) -> &'static str {
        match self {
            CUintTy::Usize => "size_t",
            CUintTy::U8 => "uint8_t",
            CUintTy::U16 => "uint16_t",
            CUintTy::U32 => "uint32_t",
            CUintTy::U64 => "uint64_t",
        }
    }

    /// The maximum value of this type. From `<stdint.h>`.
    pub fn max_value(self) -> &'static str {
        match self {
            CUintTy::Usize => "SIZE_MAX",
            CUintTy::U8 => "UINT8_MAX",
            CUintTy::U16 => "UINT16_MAX",
            CUintTy::U32 => "UINT32_MAX",
            CUintTy::U64 => "UINT64_MAX",
        }
    }
}

/// Complex C types, e.g. pointers and arrays.
///
/// This type is interned, and thus should be unique in a specific context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CTyKind<'mx> {
    /// A pointer type.
    Pointer(CTy<'mx>),
    /// An array type with element type and size.
    Array(CTy<'mx>, usize),
}

impl<'mx> ModuleCtx<'mx> {
    /// Get the type of an signed integer
    pub fn get_int_type(&self, int: IntTy) -> CTy<'mx> {
        match int {
            IntTy::Isize => CTy::Int(CIntTy::Isize),
            IntTy::I8 => CTy::Int(CIntTy::I8),
            IntTy::I16 => CTy::Int(CIntTy::I16),
            IntTy::I32 => CTy::Int(CIntTy::I32),
            IntTy::I64 => CTy::Int(CIntTy::I64),
            IntTy::I128 => unimplemented!("i128 not supported yet"),
        }
    }

    /// Get the type of an unsigned integer
    pub fn get_uint_type(&self, uint: UintTy) -> CTy<'mx> {
        match uint {
            UintTy::Usize => CTy::UInt(CUintTy::Usize),
            UintTy::U8 => CTy::UInt(CUintTy::U8),
            UintTy::U16 => CTy::UInt(CUintTy::U16),
            UintTy::U32 => CTy::UInt(CUintTy::U32),
            UintTy::U64 => CTy::UInt(CUintTy::U64),
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
        Ident(CValue<'mx>),
        Ptr,
        ArrayDim(usize),
        Lp,
        Rp,
    }

    impl Print for DeclaratorPart<'_> {
        fn print_to(&self, ctx: &mut PrinterCtx) {
            match self {
                DeclaratorPart::Ident(val) => {
                    val.print_to(ctx);
                }
                DeclaratorPart::Ptr => {
                    ctx.word("*");
                }
                DeclaratorPart::ArrayDim(dim) => {
                    ctx.word(format!("[{}]", dim.to_string()));
                }
                DeclaratorPart::Lp => ctx.word("("),
                DeclaratorPart::Rp => ctx.word(")"),
            }
        }
    }

    let mut decl_parts = std::collections::VecDeque::new();
    if let Some(val) = val {
        decl_parts.push_front(DeclaratorPart::Ident(val));
    }
    while let CTy::Ref(kind) = ty {
        match kind.0 {
            CTyKind::Pointer(ty) => {
                decl_parts.push_front(DeclaratorPart::Ptr);
                if ty.is_array() {
                    decl_parts.push_front(DeclaratorPart::Lp);
                    decl_parts.push_back(DeclaratorPart::Rp);
                }
            }
            CTyKind::Array(_, dim) => decl_parts.push_back(DeclaratorPart::ArrayDim(*dim)),
        }
        ty = match kind.0 {
            CTyKind::Pointer(ty) => *ty,
            CTyKind::Array(ty, _) => *ty,
        };
    }

    ctx.word(ty.to_str()); // `ty` should be a primitive type here
    if val.is_some() || !decl_parts.is_empty() {
        ctx.nbsp();
    }
    for part in decl_parts {
        part.print_to(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::CValue;
    use rustc_data_structures::intern::Interned;

    fn setup_printer_ctx() -> PrinterCtx {
        PrinterCtx::new()
    }

    #[test]
    fn test_print_declarator_primitive() {
        let mut ctx = setup_printer_ctx();
        // Test void type without identifier
        print_declarator(CTy::Void, None, &mut ctx);
        assert_eq!(ctx.finish(), "void");

        // Test int type with identifier
        ctx = setup_printer_ctx();
        let val = CValue::Local(0);
        print_declarator(CTy::Int(CIntTy::I32), Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t _0");
    }

    #[test]
    fn test_print_declarator_pointer() {
        let mut ctx = setup_printer_ctx();
        // Test pointer type without identifier
        let ptr_kind = CTyKind::Pointer(CTy::Int(CIntTy::I32));
        let ptr_type = CTy::Ref(Interned::new_unchecked(&ptr_kind));
        print_declarator(ptr_type, None, &mut ctx);
        assert_eq!(ctx.finish(), "int32_t *");

        // Test pointer type with identifier
        ctx = setup_printer_ctx();
        let val = CValue::Local(1);
        print_declarator(ptr_type, Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t *_1");

        // Test multi-level pointer type
        ctx = setup_printer_ctx();
        let ptr2_kind = CTyKind::Pointer(ptr_type);
        let ptr2_type = CTy::Ref(Interned::new_unchecked(&ptr2_kind));
        print_declarator(ptr2_type, None, &mut ctx);
        assert_eq!(ctx.finish(), "int32_t **");

        // Test multi-level pointer type with identifier
        ctx = setup_printer_ctx();
        let val = CValue::Local(2);
        print_declarator(ptr2_type, Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t **_2");
    }

    #[test]
    fn test_print_declarator_array() {
        let mut ctx = setup_printer_ctx();
        // Test array type without identifier
        let i32_ty = CTy::Int(CIntTy::I32);
        let array_kind = CTyKind::Array(i32_ty, 10);
        let array_type = CTy::Ref(Interned::new_unchecked(&array_kind));
        print_declarator(array_type, None, &mut ctx);
        assert_eq!(ctx.finish(), "int32_t [10]");

        // Test array type with identifier
        ctx = setup_printer_ctx();
        let val = CValue::Local(2);
        print_declarator(array_type, Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t _2[10]");
    }

    #[test]
    fn test_print_declarator_complex() {
        let mut ctx = setup_printer_ctx();
        // Test pointer to array
        let array_kind = CTyKind::Array(CTy::Int(CIntTy::I32), 10);
        let array_type = CTy::Ref(Interned::new_unchecked(&array_kind));
        let ptr_kind = CTyKind::Pointer(array_type);
        let ptr_type = CTy::Ref(Interned::new_unchecked(&ptr_kind));
        let val = CValue::Local(3);
        print_declarator(ptr_type, Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t (*_3)[10]");

        // Test array of pointers to array
        ctx = setup_printer_ctx();
        let inner_array_kind = CTyKind::Array(CTy::Int(CIntTy::I32), 5);
        let inner_array_type = CTy::Ref(Interned::new_unchecked(&inner_array_kind));
        let ptr_to_array_kind = CTyKind::Pointer(inner_array_type);
        let ptr_to_array_type = CTy::Ref(Interned::new_unchecked(&ptr_to_array_kind));
        let array_of_ptrs_kind = CTyKind::Array(ptr_to_array_type, 3);
        let array_of_ptrs_type = CTy::Ref(Interned::new_unchecked(&array_of_ptrs_kind));
        let val = CValue::Local(4);
        print_declarator(array_of_ptrs_type, Some(val), &mut ctx);
        assert_eq!(ctx.finish(), "int32_t (*_4[3])[5]");
    }
}
