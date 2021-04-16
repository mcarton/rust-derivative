# PhantomData
[PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) implements all of the derivable traits without placing bounds on the type it is given. Because of this, if a generic type is only used in a `PhantomData`, no bounds are placed on it. This is in contrast to Rust's `derive` which will add the trait bound regardless.

## Example
This:
```Rust
# extern crate derivative;
# use derivative::Derivative;
# use std::marker::PhantomData;
#[derive(Derivative)]
#[derivative(Debug(bound = "A: Debug"))]  // bound not needed.
struct Foo<A, B> {
    a: A,
    _b: PhantomData<B>,
}
```

is equivalent to this:
```Rust
# extern crate derivative;
# use derivative::Derivative;
# use std::marker::PhantomData;
#[derive(Derivative)]
#[derivative(Debug)]
struct Foo<A, B> {
    a: A,
    _b: PhantomData<B>,
}
```