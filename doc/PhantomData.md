# PhantomData
[PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) implements all of the derivable traits without placing bounds on the type it is given. Because of this, if a generic type is only used in a `PhantomData`, no bounds are placed on it. This is in contrast to Rust's `derive` which will add the trait bound regardless.

## Example
This:
```rust
# extern crate derivative;
# use derivative::Derivative;
# use std::{marker::PhantomData, fmt::Debug};
#
# struct NonDebug ();
#
#[derive(Derivative)]
#[derivative(Debug(bound = "A: Debug"))]  // bound not needed.
struct Foo<A, B> {
    a: A,
    b: PhantomData<B>,
}

fn main() {
    let foo: Foo<usize, NonDebug> = Foo{ a: 1, b: PhantomData };
    println!("{:?}", foo);
}
```

is equivalent to this:
```rust
# extern crate derivative;
# use derivative::Derivative;
# use std::{marker::PhantomData, fmt::Debug};
#
# struct NonDebug ();
#
#[derive(Derivative)]
#[derivative(Debug)]
struct Foo<A, B> {
    a: A,
    b: PhantomData<B>,
}

fn main() {
    let foo: Foo<usize, NonDebug> = Foo{ a: 1, b: PhantomData };
    println!("{:?}", foo);
}
```

Rust's derive which does not compile because it adds the bound `B: Debug`:
```rust, compile_fail
# use std::{marker::PhantomData, fmt::Debug};
#
# struct NonDebug ();
#
#[derive(Debug)] // This will added the bounds: A: Debug, B: Debug
struct Foo<A, B> {
    a: A,
    b: PhantomData<B>,
}

fn main() {
    let foo: Foo<usize, NonDebug> = Foo{ a: 1, b: PhantomData };
    println!("{:?}", foo); // Won't compile because NonDebug does not implement Debug.
}
```