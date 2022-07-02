# cargo-all-features

Cargo subcommands that build and test all feature flag combinations for a crate.

<img src=https://i.imgur.com/OVBRtEC.png width=500>

## Install

```
cargo install cargo-all-features

# or via cargo-binstall
cargo binstall cargo-all-features
```


## Usage

The following commands can be run within a Cargo package or at the root of a Cargo workspace.

Build crate with all feature flag combinations:

```
cargo all-features build -- <CARGO BUILD FLAGS>
```

Check crate with all feature flag combinations:

```
cargo all-features check -- <CARGO CHECK FLAGS>
```

Test crate with all feature flag combinations:

```
cargo all-features test -- <CARGO TEST FLAGS>
```

<details>
    <summary markdown="title"><bold>Supported tools</bold></summary>

- First party
    - [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) cargos integrated testing tool
    - [`cargo check`](https://doc.rust-lang.org/cargo/commands/cargo-check.html) cargos integrated checking tool
    - [`cargo build`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) cargos integrated build tool
    - `cargo bench` [Used by cargos benching feature](https://doc.rust-lang.org/cargo/commands/cargo-bench.html) or crates like [citerion](https://github.com/bheisler/criterion.rs)
- Additional RustUp components
    - [`cargo miri test`](https://github.com/rust-lang/miri) for testing using miri -> _rustup component `miri` is needed_
- Cargo plugins
    - [`cargo udeps`](https://github.com/est31/cargo-udeps) to analyze for unused dependencies -> _cargo plugin `cargo-udeps` is needed_
    - [`cargo tarpaulin`](https://github.com/xd009642/tarpaulin) generate code coverage reports -> _cargo plugin `cargo-tarpaulin` is needed_
    - [`cargo nextest`](https://nexte.st/) the next generation test runner for cargo -> _cargo plugin `cargo-nextest` is needed_

> for more information run `cargo all-features --help`
</details>

<details>
    <summary markdown="span">Additional Features</summary>

### Chunking

If certain projects, features might add up and CI jobs can take longer. In order to shrink wall time of your builds you can specify `--chunks` (the total amount of junks to split into _[1..]_) and `--chunk` (the chunk nr of the one executed command _\[1..\<CHUNKS\>\]_) per execution.

I.e. in github you can use a job matrix:

```yaml
name: CI

on: [pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        chunk: [1,2,3,4]
        chunks: 4
    steps:
    - uses: actions/checkout@v2
    - name: Install stable toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
     - name: Install cargo-all-features
        uses: actions-rs/cargo@v1
        with:
          command: install
          args: cargo-all-features --version 1.8.0
    - name: Build all features for release
      run: cargo all-features build --chunks  ${{matrix.chunks}} --chunk  ${{matrix.chunk}} -- --release
```

### Dry run & Verbosity

You are not sure if you configured something correct but don't have the time to wait for all tests or builds? Use `--dry-run`, it will skip all command execution.

If you are not sure if the correct command are executed use `--verbose`

### RustUp toolchain

Don't mind to use `+<toolchain>` or any other combination of rustups toolchain selection. `cargo-all-features` will pick up on the active toolchain and use it.

> for more information run `cargo all-features --help`

### Cross
> If you never heard of [cross](https://github.com/cross-rs/cross), an almost zero setup cross compilation cli setup

While there is no way to directly know if you are calling this cargo subcommand from cross, there is a `--target-command` flag which can be set to `cross` which will forward the feature flags to `cross` instead of `cargo`
</details>

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

# Only include certain features in the build matrix
#(incompatible with `denylist`, `skip_optional_dependencies`, and `extra_features`)
allowlist = ["foo", "bar"]
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
