# Kaleidoscope.rs

A implementation of the [Kaleidoscope](https://llvm.org/docs/tutorial/index.html) language.
It's meant to show how to use different crates, that makes compiler engineering in Rust
much easier.

## Crates used

- [codespan-reporting](https://docs.rs/codespan-reporting)
- [salsa](https://github.com/salsa-rs/salsa)
- [pretty](https://docs.rs/pretty)
- [logos](https://docs.rs/logos)
- [cranelift](https://docs.rs/cranelift) or [inkwell](https://github.com/TheDan64/inkwell) or some other llvm bindings
- probably some other crates like [lasso](https://docs.rs/lasso) or [lexical_core](https://docs.rs/lexical_core)
