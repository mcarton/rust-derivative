# Custom attributes
The `Eq` trait supports the following attributes:

## Container attributes
* [`Eq(bound="<where-clause or empty>")`](#custom-bound)

## Field attributes
* [`Eq(bound="<where-clause or empty>")`](#custom-bound)

# Custom bound

Usually a `T: Eq` bound is added for each type parameter `T`. You can use
override this behaviour if the infered bound is not correct for you.

Eg. comparing raw pointers does not require the type to be `Eq`, so you could
use:

```rust
#[derive(Derivative)]
#[derivative(Eq)]
struct WithPtr<T: ?Sized> {
    #[derivative(Eq(bound=""))]
    foo: *const T
}
```

See [`Default`'s documentation](./Default.md#custom-bound) for more details.
