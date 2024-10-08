//! Test that the generated code has the right number of parameters

#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// CHECK-LABEL: foo
// CHECK-LABEL: main

// expect three int params
// CHECK-LABEL: foo
// CHECK: (int32_t {{[a-zA-Z_][a-zA-Z0-9_]*}}, int32_t {{[a-zA-Z_][a-zA-Z0-9_]*}}, int32_t {{[a-zA-Z_][a-zA-Z0-9_]*}})
// CHECK: return 0;
#[no_mangle]
pub fn foo(_x: i32, _y: i32, _z: i32) -> i32 {
    0
}

#[no_mangle]
pub fn main() -> i32 {
    0
}
