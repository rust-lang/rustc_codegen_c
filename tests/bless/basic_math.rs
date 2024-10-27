//@ aux-build:mini_core.rs

#![feature(no_core)]
#![no_core]
#![no_main]

extern crate mini_core;

#[no_mangle]
pub fn main() -> i32 {
    0
}

#[no_mangle]
pub fn foo(x: u8, _y: u16, _z: u32) -> i64 {
    x as i64
}
