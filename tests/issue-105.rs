#![allow(dead_code)]

#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Clone)]
struct MyStruct<'a> {
    pub borrowed: &'a String,
}
