# {{project-name}}

Starter template for a Fluor function in Rust.

## Requirements

- Rust stable
- `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- `just`: `cargo install just`
- `wasm-tools`: `cargo install --locked wasm-tools`

## Build

```bash
just build
```

This produces `out/{{crate_name}}.wasm`.

The generated component implements Fluor's `v2` contract:
- exported `handler` interface
- `request`/`response` records
- `result<response, error>` return type

## Inspect

```bash
just inspect
```

## Publish

Update [fluor.toml](fluor.toml) as needed and publish the generated component with your Fluor workflow.
