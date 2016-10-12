#![feature(proc_macro)]

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(PartialEq)]
struct Foo {
    foo: u8
}

#[derive(Derivative)]
#[derivative(PartialEq)]
struct WithPtr<T: ?Sized> {
    #[derivative(PartialEq(bound=""))]
    foo: *const T
}

trait SomeTrait {}
struct SomeType {
    foo: u8
}
impl SomeTrait for SomeType {}

fn main() {
    assert!(Foo { foo: 7 } == Foo { foo: 7 });
    assert!(Foo { foo: 7 } != Foo { foo: 42 });

    let ptr1: *const SomeTrait = &SomeType { foo: 0 };
    let ptr2: *const SomeTrait = &SomeType { foo: 1 };
    assert!(WithPtr { foo: ptr1 } == WithPtr { foo: ptr1 });
    assert!(WithPtr { foo: ptr1 } != WithPtr { foo: ptr2 });
}
