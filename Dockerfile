FROM rust:1.36 as build

COPY ./src ./src Cargo.toml ./

RUN cargo build --target x86_64-unknown-linux-musl --release --all

RUN mkdir -p /build-out

RUN cp target/x86_64-unknown-linux-musl/release/server \
  target/x86_64-unknown-linux-musl/release/create-admin-user \
  target/x86_64-unknown-linux-musl/release/create-toshi-index /build-out

FROM alphine:3.10.1

COPY --from=build /build-out/* /

CMD /server
