# rustc_codegen_c

C based backend for rustc

[![CI](https://github.com/rust-lang/rustc_codegen_c/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-lang/rustc_codegen_c/actions/workflows/ci.yml)

This a C codegen backend for rustc, which lowers Rust MIR to C code and compiles
it with a C compiler.

This code is still highly experimental and not ready for production use.

## Try it

In the root directory of the project, run the following command:

```bash
./y rustc examples/basic_math.rs
./build/basic_math
```

The usage of `./y` can be viewed from `./y help`.

Note: only Linux is supported at the moment. `clang` is required to compile C code, 
and LLVM FileCheck is required to test the codegen.

## License

This project is licensed under a dual license: MIT or Apache 2.0.
