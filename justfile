default:
    @just --list

build:
    cargo component build --release
    mv target/wasm32-wasip1/release/{{project-name}}.wasm out/

test:
    cargo component test

clean:
    cargo component clean

publish: build
    fluor publish
