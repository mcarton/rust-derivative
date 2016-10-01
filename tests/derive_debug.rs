#![feature(rustc_macro)]

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Debug)]
struct Foo {
    foo: u8,
    #[derivative(ignore_for(Debug))]
    bar: u8,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Bar (
    u8,
    #[derivative(ignore_for(Debug))]
    u8,
);

trait ToDebug {
    fn to_show(&self) -> String;
}

impl<T: std::fmt::Debug> ToDebug for T {
    fn to_show(&self) -> String {
        format!("{:?}", self)
    }
}

fn main() {
    assert_eq!(Foo { foo: 42, bar: 1 }.to_show(), "Foo { foo: 42 }".to_string());
    assert_eq!(Bar(42, 1).to_show(), "Bar(42)".to_string());
}
