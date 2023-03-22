# cargo-all-features

Cargo subcommands that build and test all feature flag combinations for a crate.

<img src=https://i.imgur.com/OVBRtEC.png width=500>

## Install

```
cargo install cargo-all-features
```

## Usage

The following commands can be run within a Cargo package or at the root of a Cargo workspace.

Build crate with all feature flag combinations:

```
cargo build-all-features <CARGO BUILD FLAGS>
```

Check crate with all feature flag combinations:

```
cargo check-all-features <CARGO CHECK FLAGS>
```

Test crate with all feature flag combinations:

```
cargo test-all-features <CARGO TEST FLAGS>
```


## Why?

If you have a crate that utilizes Rust feature flags, it’s common to set up a test matrix in your continuous integration tooling to _individually_ test all feature flags. This setup can be difficult to maintain and easy to forget to update as feature flags come and go. It’s also not exhaustive, as it’s possible enabling _combinations_ of feature flags could result in a compilation error that should be fixed. This utility was built to address these concerns.

## Options

You can add the following options to your Cargo.toml file to configure the behavior of cargo-all-features under the heading `[package.metadata.cargo-all-features]`:

```toml
[package.metadata.cargo-all-features]

# Features "foo" and "bar" are incompatible, so skip permutations including them
skip_feature_sets = [
    ["foo", "bar"],
]

# If your crate has a large number of optional dependencies, skip them for speed
skip_optional_dependencies = true

# Add back certain optional dependencies that you want to include in the permutations
extra_features = [
    "log",
]

# Exclude certain features from the build matrix
denylist = ["foo", "bar"]

# Always include these features in combinations.
# These features should not be included in `skip_feature_sets` or `denylist`, they get
# added in later
always_include_features = ["baz"]

# The maximum number of features to try at once. Does not count features from `always_include_features`.
# This is useful for reducing the number of combinations run for a crate with a large amount of features,
# since in most cases a bug just needs a small set of 2-3 features to reproduce.
max_combination_size = 4

# Only include certain features in the build matrix
#(incompatible with `denylist`, `skip_optional_dependencies`, and `extra_features`)
allowlist = ["foo", "bar"]
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
