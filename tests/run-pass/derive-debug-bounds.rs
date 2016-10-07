#![feature(rustc_macro)]

#[macro_use]
extern crate derivative;

use std::fmt::{Formatter, Result as FmtResult};

#[derive(Derivative)]
#[derivative(Debug)]
struct Foo<T, U> {
    foo: T,
    #[derivative(Debug(format_with="MyDebug::my_fmt", bound="U: MyDebug"))]
    bar: U,
}

trait MyDebug {
    fn my_fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("MyDebug")
    }
}

impl MyDebug for i32 { }
impl<'a, T> MyDebug for &'a T { }


trait ToDebug {
    fn to_show(&self) -> String;
}

impl<T: std::fmt::Debug> ToDebug for T {
    fn to_show(&self) -> String {
        format!("{:?}", self)
    }
}

fn main() {
    assert_eq!(Foo { foo: 42, bar: 0 }.to_show(), "Foo { foo: 42, bar: MyDebug }".to_string());
}
