#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

macro_rules! gen {
    ($name:ident) => {
        #[derive(Derivative)]
        #[derivative(Debug)]
        pub struct $name {
            a: i32,
        }
    };
}

gen!(Test);

#[test]
fn call_it() {
    println!("{:?}", Test { a: 42 });
}
