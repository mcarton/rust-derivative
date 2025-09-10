#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Eq, PartialEq, Ord, PartialOrd)]
pub struct Foo {
    x: i64,
}
