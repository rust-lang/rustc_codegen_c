//! Test that we can return a value from a function
#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// CHECK-LABEL: main
// CHECK: 42
#[no_mangle]
pub fn main() -> i32 {
    42
}
