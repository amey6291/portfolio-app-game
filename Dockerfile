# Build the static site, then serve dist/ with static-web-server (also Rust).
#   docker build -t portfolio .
#   docker run -p 8080:80 portfolio

FROM rust:1.89-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
 && cargo install wasm-bindgen-cli --version 0.2.100 --locked \
 && apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo xtask build

FROM ghcr.io/static-web-server/static-web-server:2.32.2
COPY --from=build /src/dist /public
ENV SERVER_ROOT=/public SERVER_COMPRESSION=true
