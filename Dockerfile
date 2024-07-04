FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/openai_api_rust /usr/local/bin/openai_api_rust
CMD ["openai_api_rust"]
