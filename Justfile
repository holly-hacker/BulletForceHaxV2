#!/usr/bin/env just --justfile

default:
    @just --list

install-deps:
    rustup target add wasm32-unknown-unknown
    cargo install trunk --locked

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
    cargo clippy --all-features
    cargo doc --all-features
    cargo nextest run --all-features --status-level fail

build-frontend:
    cd bulletforcehax2_web && trunk build

watch-frontend:
    cd bulletforcehax2_web && trunk watch

run: build-frontend
    cargo run -p server

build-release:
    cd bulletforcehax2_web && trunk build --release
    cargo build -p server --profile release-publish
