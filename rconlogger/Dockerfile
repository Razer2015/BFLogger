# Build Stage
FROM rust:latest AS builder
WORKDIR /usr/src/
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

RUN USER=root cargo new rconlogger
WORKDIR /usr/src/rconlogger
COPY ./rconlogger/Cargo.toml ./rconlogger/Cargo.lock ./
RUN echo "openssl = { version = \"0.10\", features = [\"vendored\"] }" >> ./Cargo.toml
COPY ./battlelog ../battlelog
RUN cargo build --release

COPY ./rconlogger/src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Bundle Stage
FROM scratch

COPY --from=builder /usr/local/cargo/bin/rconlogger .
USER 1000
CMD ["./rconlogger"]