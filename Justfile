# Just a command runner: https://github.com/casey/just

build:
    cargo build --all-features

check:
    cargo check
    cargo +nightly udeps --all-targets

fmt:
    cargo +nightly fmt --all

test:
    cargo test --all-features
