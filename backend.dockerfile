FROM rust:1.85.0

RUN apt update && \
    apt install -y \
    librust-alsa-sys-dev \
    libudev-dev

COPY . /srv

WORKDIR /srv

RUN cargo build --release -p server

FROM debian:bookworm-slim

COPY --from=0 /srv/target/release/server /srv/server

RUN apt update && \
    apt install -y libssl3

CMD ["/srv/server"]
