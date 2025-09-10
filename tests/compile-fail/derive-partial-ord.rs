#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

#[derive(Derivative, PartialEq)]
#[derivative(PartialOrd)]
enum Option {
    Some,
    None,
}

fn main() {}
