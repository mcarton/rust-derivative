#[cfg(feature = "trybuild")]
extern crate trybuild as trybuild;

#[cfg(feature = "trybuild")]
#[test]
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
