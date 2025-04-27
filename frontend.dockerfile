FROM rust:1.85.0 AS builder

RUN cargo install wasm-pack && \
    rustup target add wasm32-unknown-unknown

COPY . /srv

WORKDIR /srv/game

RUN --mount=type=secret,id=WEBSOCKET_SERVER_ADDRESS,env=WEBSOCKET_SERVER_ADDRESS \
  wasm-pack build --release --target web --no-opt --no-pack --out-dir /srv/game/frontend/wasm

FROM node:23-alpine3.20 AS frontend

COPY --from=builder /srv/game/frontend /srv

WORKDIR /srv

RUN yarn && yarn build

FROM caddy/caddy:2-alpine

COPY --from=frontend /srv/dist /usr/share/caddy

