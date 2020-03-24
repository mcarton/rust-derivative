# Change Log
All notable changes to this project will be documented in this file.

## Unreleased - 2.1.0
* `feature_allow_slow_enum` is not required anymore on `enum` with `PartialEq`.

## 2.0.2
* Fix a bug with `format_with` on `Debug` derives with generic types with trait bounds.

## 2.0.1
* Fix a hygiene bug with `Debug`. ([#60])

## 2.0.0
This release should be compatible with version 1.*, but now requires rustc version 1.34 or later.
* Update `syn`, `quote`, and `proc-macro2` dependencies. ([#59])

## 1.0.4
This is the last version to support rustc versions 1.15 to 1.33.

* Implement `PartialOrd` and `Ord` deriving.

## 1.0.3
* Do not require `syn`'s `full` feature anymore. ([#38], [#45])
* Fix an issue with using `#[derivative(Debug(format_with = "…"))]` on non-generic types. ([#40])
* Fix some warnings in the library with recent versions of `rustc`.
* Fix some `clippy::pedantic` warnings in generated code. ([#46])

## 1.0.2
* Add `use_core` feature to make `Derivative` usable in `core` crates.

## 1.0.1
* Updated `syn` to `0.15`. ([#25])
* Updated `quote` to `0.6`. ([#25])

## 1.0.0
* Make stable

## 0.3.1
* Fix a warning in `derivative(Debug)`
* Remove all `feature`s, this makes the crate usable on `beta`

[#25]: https://github.com/mcarton/rust-derivative/issues/25
[#38]: https://github.com/mcarton/rust-derivative/pull/38
[#40]: https://github.com/mcarton/rust-derivative/pull/40
[#45]: https://github.com/mcarton/rust-derivative/pull/45
[#46]: https://github.com/mcarton/rust-derivative/pull/46
[#59]: https://github.com/mcarton/rust-derivative/pull/59
[#60]: https://github.com/mcarton/rust-derivative/pull/60
[#61]: https://github.com/mcarton/rust-derivative/pull/61
