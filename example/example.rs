#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

#[no_mangle]
pub fn main() -> i32 {
    0
}

#[no_mangle]
pub fn foo(x: u8) -> i64 {
    x as i64
}
