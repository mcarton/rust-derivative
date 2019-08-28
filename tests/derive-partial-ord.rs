#[cfg(feature = "use_core")]
extern crate core;

use std::marker::PhantomData;

#[macro_use]
extern crate derivative;

#[derive(PartialEq)]
#[derive(Derivative)]
#[derivative(PartialOrd)]
struct Foo {
    foo: u8,
}

#[derive(PartialEq)]
#[derive(Derivative)]
#[derivative(PartialOrd = "feature_allow_slow_enum")]
enum Option<T> {
    None,
    Some(T),
}

#[derive(Derivative)]
#[derivative(PartialEq, PartialOrd)]
struct WithPtr<T: ?Sized> {
    #[derivative(PartialEq(bound = ""))]
    #[derivative(PartialOrd(bound = ""))]
    foo: *const T,
}

#[derive(PartialEq)]
#[derive(Derivative)]
#[derivative(PartialOrd)]
struct Empty;

#[derive(PartialEq)]
#[derive(Derivative)]
#[derivative(PartialOrd)]
struct AllIgnored {
    #[derivative(PartialOrd = "ignore")]
    foo: u8,
}

#[derive(PartialEq)]
#[derive(Derivative)]
#[derivative(PartialOrd)]
struct OneIgnored {
    #[derivative(PartialOrd = "ignore")]
    foo: u8,
    bar: u8,
}

// #[derive(PartialEq)]
// #[derive(Derivative)]
// #[derivative(PartialOrd)]
// struct Parity(#[derivative(PartialOrd(compare_with = "same_parity"))] u8);

// fn same_parity(lhs: &u8, rhs: &u8) -> bool {
//     lhs % 2 == rhs % 2
// }

// #[derive(PartialEq)]
// #[derive(Derivative)]
// #[derivative(PartialOrd)]
// struct Generic<T>(#[derivative(PartialOrd(compare_with = "dummy_cmp", bound = ""))] T);

// fn dummy_cmp<T>(_: &T, _: &T) -> bool {
//     true
// }

struct NonPartialOrd;

#[derive(Derivative)]
#[derivative(PartialEq, PartialOrd)]
struct GenericIgnore<T> {
    f: u32,
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    t: PhantomData<T>,
}

trait SomeTrait {}
struct SomeType {
    #[allow(dead_code)]
    foo: u8,
}
impl SomeTrait for SomeType {}

#[test]
fn main() {
    assert!(Foo { foo: 7 } < Foo { foo: 42 });
    assert!(Foo { foo: 7 } <= Foo { foo: 42 });
    assert!(Foo { foo: 42 } > Foo { foo: 7 });
    assert!(Foo { foo: 42 } >= Foo { foo: 7 });

    let ptrs: [*const SomeTrait; 2] = [
        &SomeType { foo: 1 },
        &SomeType { foo: 0 },
    ];
    let ptr1: *const SomeTrait = ptrs[0];
    let ptr2: *const SomeTrait = ptrs[1];
    assert!(WithPtr { foo: ptr1 } == WithPtr { foo: ptr1 });
    assert!(WithPtr { foo: ptr1 } != WithPtr { foo: ptr2 });

    assert!(Empty <= Empty);
    assert!(Empty >= Empty);
    assert!(AllIgnored { foo: 0 } <= AllIgnored { foo: 42 });
    assert!(AllIgnored { foo: 0 } >= AllIgnored { foo: 42 });
    assert!(OneIgnored { foo: 0, bar: 6 } <= OneIgnored { foo: 42, bar: 6 });
    assert!(OneIgnored { foo: 0, bar: 6 } >= OneIgnored { foo: 42, bar: 6 });
    assert!(OneIgnored { foo: 0, bar: 7 } >= OneIgnored { foo: 42, bar: 6 });

    assert!(Option::Some(42) <= Option::Some(42));
    assert!(Option::Some(42) >= Option::Some(42));
    assert!(Option::Some(0) <= Option::Some(42));
    assert!(Option::Some(42) >= Option::None);
    assert!(Option::None <= Option::Some(42));
    assert!(Option::None::<u8> <= Option::None::<u8>);
    assert!(Option::None::<u8> >= Option::None::<u8>);

    // assert!(Parity(3) == Parity(7));
    // assert!(Parity(2) == Parity(42));
    // assert!(Parity(3) != Parity(42));
    // assert!(Parity(2) != Parity(7));

    // assert!(Generic(SomeType { foo: 0 }) <= Generic(SomeType { foo: 0 }));
    assert!(
        GenericIgnore {
            f: 123,
            t: PhantomData::<NonPartialOrd>::default()
        } <= GenericIgnore {
            f: 123,
            t: PhantomData::<NonPartialOrd>::default()
        }
    );
}
