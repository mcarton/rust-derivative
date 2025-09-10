#![deny(clippy::let_underscore_must_use)]
#![allow(dead_code)]

use std::{convert::Infallible, marker::PhantomData};

#[cfg(feature = "use_core")]
extern crate core;

extern crate derivative;

#[derive(derivative::Derivative)]
#[derivative(Clone)]
pub enum Enum<T> {
    /// First Choice
    First,

    /// Second Choice
    Second,

    /// Parameter Marker
    #[doc(hidden)]
    __(Infallible, PhantomData<T>),
}
