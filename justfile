run *args:
    cargo run --bin toon -- {{args}}

run-ai *args:
    AGENT=1 cargo run --bin toon -- {{args}}

build:
    cargo build --release --bin toon
    mkdir -p bin
    cp target/release/toon bin/toon

test:
    cargo test --all-targets

lint:
    rustfmt --check --edition 2024 --config skip_children=true src/lib.rs src/main.rs tests/behavior.rs
    cargo clippy --all-targets -- -D warnings

coverage:
    cargo llvm-cov --all-targets --fail-under-lines 90

check: lint test coverage
