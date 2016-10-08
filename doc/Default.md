# Custom attributes
The `Default` trait supports the following attributes:

## Container attributes
* [`Default="new"`](#new-function)

## Variant attributes
* [`Default`](#default-enumeration)

## Field attributes
* [`Default(value="<expr>")`](#ignoring-a-field)

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

# `new` function

You can use *derivative* to derive a convenience `new` method for your type
that calls `Default::default`:

```rust
#[derive(Debug, Derivative)]
#[derivative(Default(new="true")]
struct Foo {
    foo: u8,
    bar: u8,
}

println!("{:?}", Foo::new()); // Foo { foo: 0, bar: 0 }
```
