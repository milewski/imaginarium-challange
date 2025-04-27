serve:
   WASM_SERVER_RUNNER_ADDRESS=0.0.0.0 cargo run -p game --target wasm32-unknown-unknown --release

run:
    cargo run -p game --release

run-server:
    cargo run -p server --release

start:
    cd game/frontend && yarn dev

build:
    cd game && wasm-pack build --release --target web --no-opt --no-pack --out-dir frontend/wasm
    cp -r assets ./game/frontend/public

build-optimized:
    cd game && wasm-pack build --release --target web --no-pack --out-dir frontend/wasm
    cp -r assets ./game/frontend/public
#    cargo build -p game --release --target wasm32-unknown-unknown
#    just optimize

optimize:
    wasm-opt -Oz -o build/game.optimized.wasm target/wasm32-unknown-unknown/release/game.wasm