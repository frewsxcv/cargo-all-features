# cargo-test-all-features

A [Cargo] subcommand to test all feature flag combinations.

<img src=https://i.imgur.com/OVBRtEC.png width=500>

## Install

```
cargo install cargo-test-all-features
```

## Usage

```
cargo test-all-features <CARGO TEST FLAGS>
```

[Cargo]: https://doc.rust-lang.org/cargo/

## Why?

If you have a crate that utilizes Rust feature flags, it’s common to set up a test matrix in your continuous integration tooling to _individually_ test all feature flags. This setup can be difficult to maintain and easy to forget to update as feature flags come and go. It’s also not exhaustive, as it’s possible enabling _combinations_ of feature flags could result in a compilation error that should be fixed. This utility was built to address these concerns.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
