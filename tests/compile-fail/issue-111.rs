#[cfg(feature = "use_core")]
extern crate core;

extern crate derivative;

struct DoesNotImplClone {}
impl DoesNotImplClone {
    fn clone(&self) -> Self {
        Self {}
    }
}

#[derive(derivative::Derivative)]
#[derivative(Clone)]
struct Derivative {
    x: DoesNotImplClone,
}

#[derive(Clone)]
struct Derive {
    x: DoesNotImplClone,
}

fn main() {}
