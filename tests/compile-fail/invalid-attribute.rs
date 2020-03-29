#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Clone = not_a_string)]
struct Foo;

#[derive(Derivative)]
#[derivative(Clone = 1+2)]
struct Bar;

fn main() {}