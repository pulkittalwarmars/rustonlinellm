# Build stage
FROM rust:1.72 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/openai_api_rust /usr/local/bin/openai_api_rust
ENV PORT=8080
EXPOSE 8080
CMD ["openai_api_rust"]
