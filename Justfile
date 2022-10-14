#!/usr/bin/env just --justfile

default:
    @just --list

cover-photon $RUSTFLAGS='-Cinstrument-coverage' $CARGO_INCREMENTAL='0' $LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw': && clear-coverage
    rm -r -f target/coverage/html
    rm -r -f lcov.info
    mkdir -p target/coverage/html
    cargo test -p photon_lib
    grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
    grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o lcov.info

clear-coverage:
    find . -name "*.profraw" -type f -delete

check:
    cargo clippy
    cargo doc
    cargo nextest run
