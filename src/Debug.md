The `Debug` trait supports two attributes:

* [`ignore`](#ignoring-a-field)
* [`transparent`](#hiding-newtypes)

# Ignoring a field

You can use *derivative* to hide fields from a structure or enumeration `Debug`
implementation:

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

# Hiding newtypes

You can use *derivative* to automatically unwrap newtypes and enumeration
variants with only one field:

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
