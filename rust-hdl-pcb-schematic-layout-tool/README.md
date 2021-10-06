## Run native

1. Execute `cargo run`

## Run web

1. install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and [http](https://crates.io/crates/https)
2. Execute `wasm-pack build --target web --dev`
3. Execute `http`
4. Navigate browser to `localhost:8000`