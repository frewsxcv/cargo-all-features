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
# These features should not be included in `denylist`
always_include_features = ["baz"]

# The maximum number of features to try at once. Does not count features from `always_include_features`.
# This is useful for reducing the number of combinations run for a crate with a large amount of features,
# since in most cases a bug just needs a small set of 2-3 features to reproduce.
max_combination_size = 4

# Only include certain features in the build matrix
#(incompatible with `denylist`, and `extra_features`)
allowlist = ["foo", "bar"]

# Specify rules through expressions using logical operators !, &, ^, |, <=>, =>,
# and summation +, -, >=, <=, >, <, == (in order of high to low precedence)
# Every selected feature set will fullfill ALL specified rules.
rules = [
    "A + B + C == 1",  # exactly one of the three features A, B, C is enabled
    "A + B + C >= 1",  # at least one of the three is enabled
    "A => (B|C)",  # if package A is enabled, at least one of B or C needs to be enabled too
    "'foo-bar'",  # the feature set must contain feature foo-bar, use '' quotation for feature names with hyphens
    """((A => (B|C)) <=> (A+C==1)) \
    | !'foo-bar_baz' """,  # expressions can be arbitrarily nested
    "!(A | B | C)",  # equivalent to denylist = [A, B, C]
    "A & B & C",  # equivalent to always_include_features = [A, B, C]
    "!(A & B & C)",  # equivalent to skip_feature_sets containing [A, B, C]
]
```

The project also supports chunking: `--n-chunks 3 --chunks 1` will split the crates being tested into three sets (alphabetically, currently), and run the requested command for the first set of crates only. This is useful for splitting up CI jobs or performing disk cleanups since for large workspaces `check-all-features` and friends can take a very long time and produce a ton of artifacts.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
