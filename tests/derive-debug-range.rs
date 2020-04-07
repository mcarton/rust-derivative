#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

#[derive(Derivative)]
#[derivative(Debug)]
struct Beginning {
    #[derivative(Debug(range = "3.."))]
    bar: Vec<usize>,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct End {
    #[derivative(Debug(range = "..4"))]
    bar: Vec<usize>,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Both {
    #[derivative(Debug(range = "3..4"))]
    bar: Vec<usize>,
}

#[derive(Derivative)]
#[derivative(Debug)]
struct None {
    #[derivative(Debug(range = ".."))]
    bar: Vec<usize>,
}

#[test]
fn beginning() {
    let expected = [
        "Beginning { bar: [] }",
        "Beginning { bar: [0] }",
        "Beginning { bar: [0, 1] }",
        "Beginning { bar: [0, 1, 2] }",
    ];

    let expected = expected
        .iter()
        .chain(std::iter::repeat(&"Beginning { bar: [0, 1, 2, ..] }"));

    for (i, &expected) in expected.take(10).enumerate() {
        assert_eq!(
            format!("{:?}", Beginning { bar: (0..i).collect() }),
            expected
        );
    }
}

#[test]
fn end() {
    let expected = [
        "End { bar: [] }",
        "End { bar: [0] }",
        "End { bar: [0, 1] }",
        "End { bar: [0, 1, 2] }",
        "End { bar: [0, 1, 2, 3] }",
        "End { bar: [.., 1, 2, 3, 4] }",
        "End { bar: [.., 2, 3, 4, 5] }",
    ];

    for (i, &expected) in expected.iter().enumerate() {
        assert_eq!(
            format!("{:?}", End { bar: (0..i).collect() }),
            expected
        );
    }
}

#[test]
fn both() {
    let expected = [
        "Both { bar: [] }",
        "Both { bar: [0] }",
        "Both { bar: [0, 1] }",
        "Both { bar: [0, 1, 2] }",
        "Both { bar: [0, 1, 2, 3] }",
        "Both { bar: [0, 1, 2, 3, 4] }",
        "Both { bar: [0, 1, 2, 3, 4, 5] }",
        "Both { bar: [0, 1, 2, 3, 4, 5, 6] }",
        "Both { bar: [0, 1, 2, .., 4, 5, 6, 7] }",
        "Both { bar: [0, 1, 2, .., 5, 6, 7, 8] }",
        "Both { bar: [0, 1, 2, .., 6, 7, 8, 9] }",
        "Both { bar: [0, 1, 2, .., 7, 8, 9, 10] }",
    ];

    for (i, &expected) in expected.iter().enumerate() {
        assert_eq!(
            format!("{:?}", Both { bar: (0..i).collect() }),
            expected
        );
    }
}

#[test]
fn none() {
    let expected = [
        "None { bar: [] }",
        "None { bar: [..] }",
        "None { bar: [..] }",
        "None { bar: [..] }",
    ];

    for (i, &expected) in expected.iter().enumerate() {
        assert_eq!(
            format!("{:?}", None { bar: (0..i).collect() }),
            expected
        );
    }
}
