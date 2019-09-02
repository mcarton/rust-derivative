#[cfg(feature = "use_core")]
extern crate core;


use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct A {
    #[derivative(Debug(format_with = "std::fmt::Debug::fmt"))]
    v: u64,
}
