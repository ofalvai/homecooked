VERSION 0.7
FROM rust:1.71-alpine
WORKDIR /homecooked
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

install-chef:
   RUN cargo install --debug cargo-chef

prepare-cache:
    FROM +install-chef
    COPY --dir embeddings/src embeddings/Cargo.toml embeddings/
    COPY --dir focus/src focus/Cargo.toml focus/
    COPY --dir gardener/src gardener/Cargo.toml gardener/
    COPY --dir speedtest-to-influx/src speedtest-to-influx/Cargo.toml speedtest-to-influx/
    COPY Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json

build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./
    RUN cargo chef cook
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home

build-all:
    COPY --dir embeddings/src embeddings/Cargo.toml embeddings/
    COPY --dir focus/src focus/Cargo.toml focus/
    COPY --dir gardener/src gardener/Cargo.toml gardener/
    COPY --dir speedtest-to-influx/src speedtest-to-influx/Cargo.toml speedtest-to-influx/
    COPY Cargo.lock Cargo.toml .

    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN cargo build
    SAVE ARTIFACT target/debug/embeddings embeddings
    SAVE ARTIFACT target/debug/focus focus
    SAVE ARTIFACT target/debug/gardener gardener
    SAVE ARTIFACT target/debug/speedtest-to-influx speedtest-to-influx
