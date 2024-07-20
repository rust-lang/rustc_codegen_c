#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

// expect three int params
// CHECK: {{((int32_t .*,?\s?){3})}}
#[no_mangle]
pub fn foo(_x: i32, _y: i32, _z: i32) -> i32 {
    0
}

#[no_mangle]
pub fn main() -> i32 {
    0
}
