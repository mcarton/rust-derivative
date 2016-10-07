The `Default` trait supports one attribute:

* [`value`](#ignoring-a-field)

# Setting the value of a field

You can use *derivative* to change the default value of a field in a `Default`
implementation:

```rust
#[derive(Debug, Derivative)]
#[derivative(Default)]
struct Foo {
    foo: u8,
    #[derivative(Default(value="42"))]
    bar: u8,
}

println!("{:?}", Foo::default()); // Foo { foo: 0, bar: 42 }
```

# Default enumeration

You can use *derivative* to derive a default implementation on enumerations!
This does not work with *rustc*'s `#[derive(Default)]`.
All you need is to specify what variant is the default value:

```rust
#[derive(Debug, Derivative)]
#[derivative(Default)]
enum Enum {
    A,
    #[derivative(Default)]
    B,
}

println!("{:?}", Enum::default()); // B
```
