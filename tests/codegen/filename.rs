// CHECK: filename

#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// CHECK: function_name
#[no_mangle]
pub fn function_name() -> i32 {
    0
}

#[no_mangle]
pub fn main() {}
