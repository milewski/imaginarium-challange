FROM debian:bookworm-slim

RUN apt update && \
    apt install -y libssl3 curl

RUN mkdir -p --mode=0755 /usr/share/keyrings && \
    curl -fsSL https://pkgs.tailscale.com/stable/debian/bookworm.noarmor.gpg | tee /usr/share/keyrings/tailscale-archive-keyring.gpg >/dev/null && \
    curl -fsSL https://pkgs.tailscale.com/stable/debian/bookworm.tailscale-keyring.list | tee /etc/apt/sources.list.d/tailscale.list && \
    apt update && apt install -y tailscale

COPY ./server /srv/server

WORKDIR /srv

CMD ["/srv/server"]
