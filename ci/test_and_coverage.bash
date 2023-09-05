#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
pushd $SCRIPT_DIR/..

rm target/profraw/cargo-test-*.profraw
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/profraw/cargo-test-%p-%m.profraw' cargo test

rm -rf target/coverage
grcov target/profraw/. --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o target/coverage/html

popd
