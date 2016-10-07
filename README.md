This crate provides a set of alternative `#[derive]` attribute.

## Stability

This crate is not stable yet and the attributes might change at any time.

For now the crate only works on *nightly* but it uses *rustc*'s Macros 1.1 and
is meant to be usable on *stable* as soon as it is possible.

## Implemented traits

For now only `Debug` is supported.

### `#[derivative(Debug)]`

#### Ignoring a field

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

#### Don't show new-types

```rust
#[derive(Derivative)]
#[derivative(Debug="transparent")]
struct A(isize);

#[derive(Derivative)]
#[derivative(Debug)]
enum C {
    Foo(u8),
    #[derivative(Debug="transparent")]
    Bar(u8),
}

println!("{:?}", A(42)); // 42
println!("{:?}", C::Bar(42)); // 42

// But:
println!("{:?}", C::Foo(42)); // Foo(42)
```

This only works when the structure or variant only has one field.

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
