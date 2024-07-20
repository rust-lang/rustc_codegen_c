#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// expect three int params
// CHECK: 42
#[no_mangle]
pub fn main() -> i32 {
    42
}
