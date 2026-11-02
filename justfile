default:
    @just --list

setup:
    rustup target add wasm32-wasip2

build:
    just setup
    mkdir -p out
    cargo build --target wasm32-wasip2 --release
    cp target/wasm32-wasip2/release/{{crate_name}}.wasm out/{{crate_name}}.wasm

inspect: build
    wasm-tools component wit out/{{crate_name}}.wasm

test:
    cargo test
