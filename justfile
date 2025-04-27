set dotenv-load

serve:
   WASM_SERVER_RUNNER_ADDRESS=0.0.0.0 cargo run -p game --target wasm32-unknown-unknown --release

run:
    cargo run -p game --release

run-server:
   API_SERVER_ADDRESS=https://api.docker.localhost \
   COMFYUI_HOST_URL=http://192.168.50.230:8188 \
   cargo run -p server --release

start:
    cd game/frontend && yarn dev

build:
    cd game && \
    WEBSOCKET_SERVER_ADDRESS=wss://websocket.docker.localhost \
    wasm-pack build --release --target web --no-opt --no-pack --out-dir frontend/wasm
    cp -r assets ./game/frontend/public

build-optimized:
    cd game && wasm-pack build --release --target web --no-pack --out-dir frontend/wasm
    cp -r assets ./game/frontend/public
#    cargo build -p game --release --target wasm32-unknown-unknown
#    just optimize

optimize:
    wasm-opt -Oz -o build/game.optimized.wasm target/wasm32-unknown-unknown/release/game.wasm

deploy-server:
    cargo build -p server --release
    docker build -f docker/server.dockerfile -t $REGISTRY/imaginarium/server:latest ./target/release
    docker push $REGISTRY/imaginarium/server:latest

deploy-wasm:
    cd game && wasm-pack build --release --target web --no-pack --out-dir frontend/wasm
    cd game/frontend && yarn build
