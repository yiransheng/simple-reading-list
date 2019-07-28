FROM rust:1.36 as build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./

COPY ./src ./src

RUN mkdir -p ./.cargo

COPY cargo_config ./.cargo/config

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl --all

RUN mkdir -p /build-out

RUN cp -t /build-out \
  target/x86_64-unknown-linux-musl/release/server \
  target/x86_64-unknown-linux-musl/release/create-admin-user \
  target/x86_64-unknown-linux-musl/release/create-toshi-index

FROM alpine:latest

RUN apk add --no-cache libpq

COPY --from=build /build-out/* /

CMD /server
