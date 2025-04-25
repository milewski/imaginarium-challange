serve:
   WASM_SERVER_RUNNER_ADDRESS=0.0.0.0 cargo run -p game --target wasm32-unknown-unknown --release

run:
    cargo run -p game --release

optimize:
    wasm-opt -Oz -o target/wasm32-unknown-unknown/release/game.optimized.wasm target/wasm32-unknown-unknown/release/game.wasm