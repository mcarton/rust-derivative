# Custom attributes
The `Eq` and `PartialEq` traits support the following attributes:

## Container attributes
* [`Trait(bound="<where-clause or empty>")`](#custom-bound)

## Field attributes
* [`Trait(bound="<where-clause or empty>")`](#custom-bound)

The `Partial` trait also supports the following attributes:

## Field attributes
* [`PartialEq="ignore"`](#ignoring-a-field)

# Ignoring a field

You can use *derivative* to ignore a field when comparing:

```rust
#[derive(Derivative)]
#[derivative(PartialEq)]
struct Foo {
    foo: u8,
    #[derivative(PartialEq="ignore")]
    bar: u8,
}

assert!(Foo { foo: 0, bar: 42 } == Foo { foo: 0, bar: 7});
assert!(Foo { foo: 42, bar: 0 } != Foo { foo: 7, bar: 0});
```

# Custom bound

Usually a `T: Eq` bound is added for each type parameter `T`. You can use
override this behaviour if the infered bound is not correct for you.

Eg. comparing raw pointers does not require the type to be `Eq`, so you could
use:

```rust
#[derive(Derivative)]
#[derivative(Eq)]
struct WithPtr<T: ?Sized> {
    #[derivative(Eq(bound=""))]
    foo: *const T
}
```

See [`Default`'s documentation](./Default.md#custom-bound) for more details.
