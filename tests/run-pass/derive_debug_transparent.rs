#![feature(rustc_macro)]

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Debug="transparent")]
struct A(isize);

#[derive(Derivative)]
#[derivative(Debug="transparent")]
struct B([isize; 1]);

trait ToDebug {
    fn to_show(&self) -> String;
}

impl<T: std::fmt::Debug> ToDebug for T {
    fn to_show(&self) -> String {
        format!("{:?}", self)
    }
}

fn main() {
    assert_eq!(A(42).to_show(), "42".to_string());
    assert_eq!(B([42]).to_show(), "[42]".to_string());
}
