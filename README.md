This crate provides a set of alternative `#[derive]` attributes for Rust.

## [Documentation][documentation]
## Stability

This crate is not stable yet and the attributes might change at any time.

For now the crate only works on *nightly* but it uses *rustc*'s Macros 1.1 and
is meant to be usable on *stable* as soon as it is possible.

## What it does

```rust
#[derive(Derivative)]
#[derivative(Debug)]
struct Foo {
    foo: u8,
    #[derivative(Debug="ignore")]
    bar: u8,
}

println!("{:?}", Foo { foo: 42, bar: 1 }); // Foo { foo: 42 }
```

Check the [documentation] for more!

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[documentation]: https://mcarton.github.io/rust-derivative/
