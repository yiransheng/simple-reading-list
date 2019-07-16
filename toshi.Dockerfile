FROM rust:1.36 as build

COPY ./ ./

RUN cargo build --target x86_64-unknown-linux-musl --release --bin toshi

RUN mkdir -p /build-out

RUN cp target/x86_64-unknown-linux-musl/release/toshi /build-out

FROM alphine:3.10.1

COPY --from=build /build-out/* /

CMD ["/toshi", "-c" "toshi_config.toml"]
