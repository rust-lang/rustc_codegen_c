#![feature(no_core, lang_items, rustc_attrs, intrinsics, decl_macro)]
#![no_core]
#![allow(internal_features)]

// #[rustc_builtin_macro]
// #[rustc_macro_transparency = "semitransparent"]
// pub macro stringify($($t:tt)*) { /* compiler built-in */ }

#[lang = "sized"]
pub trait Sized {}

#[lang = "copy"]
pub unsafe trait Copy {}

unsafe impl Copy for bool {}
unsafe impl Copy for u8 {}
unsafe impl Copy for u16 {}
unsafe impl Copy for u32 {}
unsafe impl Copy for u64 {}
unsafe impl Copy for usize {}
unsafe impl Copy for i8 {}
unsafe impl Copy for i16 {}
unsafe impl Copy for i32 {}
unsafe impl Copy for isize {}
unsafe impl Copy for f32 {}
unsafe impl Copy for f64 {}
unsafe impl Copy for char {}
unsafe impl<'a, T: ?Sized> Copy for &'a T {}
unsafe impl<T: ?Sized> Copy for *const T {}
unsafe impl<T: ?Sized> Copy for *mut T {}

// #[lang = "add"]
// pub trait Add<Rhs = Self> {
//     type Output;

//     #[must_use = "this returns the result of the operation, without modifying the original"]
//     fn add(self, rhs: Rhs) -> Self::Output;
// }

// impl Add for i32 {
//     type Output = i32;

//     #[inline]
//     fn add(self, other: i32) -> i32 { self + other }
// }

// #[lang = "panic"]
// #[track_caller]
// pub fn panic(_msg: &'static str) -> ! {
//     unsafe {
//         libc::puts("Panicking\n\0" as *const str as *const u8);
//         intrinsics::abort();
//     }
// }

// #[lang = "panic_location"]
// #[allow(dead_code)]
// struct PanicLocation {
//     file: &'static str,
//     line: u32,
//     column: u32,
// }

// macro_rules! panic_const {
//     ($($lang:ident = $message:expr,)+) => {
//         pub mod panic_const {
//             use super::*;

//             $(
//                 #[track_caller]
//                 #[lang = stringify!($lang)]
//                 pub fn $lang() -> ! {
//                     panic($message);
//                 }
//             )+
//         }
//     }
// }

// panic_const! {
//     panic_const_add_overflow = "attempt to add with overflow",
//     panic_const_sub_overflow = "attempt to subtract with overflow",
//     panic_const_mul_overflow = "attempt to multiply with overflow",
//     panic_const_div_overflow = "attempt to divide with overflow",
//     panic_const_rem_overflow = "attempt to calculate the remainder with overflow",
//     panic_const_neg_overflow = "attempt to negate with overflow",
//     panic_const_shr_overflow = "attempt to shift right with overflow",
//     panic_const_shl_overflow = "attempt to shift left with overflow",
//     panic_const_div_by_zero = "attempt to divide by zero",
//     panic_const_rem_by_zero = "attempt to calculate the remainder with a divisor of zero",
// }

// pub mod intrinsics {
//     use crate::Sized;

//     extern "rust-intrinsic" {
//         #[rustc_safe_intrinsic]
//         pub fn abort() -> !;
//         #[rustc_safe_intrinsic]
//         pub fn size_of<T>() -> usize;
//         pub fn size_of_val<T: ?Sized>(val: *const T) -> usize;
//         #[rustc_safe_intrinsic]
//         pub fn min_align_of<T>() -> usize;
//         pub fn min_align_of_val<T: ?Sized>(val: *const T) -> usize;
//         pub fn copy<T>(src: *const T, dst: *mut T, count: usize);
//         pub fn transmute<T, U>(e: T) -> U;
//         pub fn ctlz_nonzero<T>(x: T) -> u32;
//         #[rustc_safe_intrinsic]
//         pub fn needs_drop<T: ?Sized>() -> bool;
//         #[rustc_safe_intrinsic]
//         pub fn bitreverse<T>(x: T) -> T;
//         #[rustc_safe_intrinsic]
//         pub fn bswap<T>(x: T) -> T;
//         pub fn write_bytes<T>(dst: *mut T, val: u8, count: usize);
//         pub fn unreachable() -> !;
//     }
// }

pub mod libc {
    #[link(name = "c")]
    extern "C" {
        pub fn puts(s: *const u8) -> i32;
        pub fn printf(format: *const i8, ...) -> i32;
        pub fn malloc(size: usize) -> *mut u8;
        pub fn free(ptr: *mut u8);
        pub fn memcpy(dst: *mut u8, src: *const u8, size: usize);
        pub fn memmove(dst: *mut u8, src: *const u8, size: usize);
        pub fn strncpy(dst: *mut u8, src: *const u8, size: usize);
    }
}
