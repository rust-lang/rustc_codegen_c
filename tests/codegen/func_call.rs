//! Test that the generated code correctly handles function calls

//@ aux-build:mini_core.rs
//@ run-pass
//@ exit-code: 1

// CHECK-LABEL: single
// CHECK: (int32_t {{[[:alnum:]_]*}})

#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

#[no_mangle]
pub fn single(a: i32) -> i32 {
    a
}

// CHECK-LABEL: main
// CHECK: int32_t {{[[:alnum:]_]*}} = single(1);
// CHECK: return {{[[:alnum:]_]*}};
#[no_mangle]
pub fn main() -> i32 {
    single(1)
}
