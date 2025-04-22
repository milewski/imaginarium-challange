serve:
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner cargo run -p game --target wasm32-unknown-unknown --release

run:
    cargo run -p game --release