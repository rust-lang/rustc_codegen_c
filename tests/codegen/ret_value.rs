//! Test that we can return a value from a function

//@ aux-build:mini_core.rs

#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// CHECK-LABEL: main
// CHECK: return 42;
#[no_mangle]
pub fn main() -> i32 {
    42
}

//@ run-pass
//@ exit-code: 42
