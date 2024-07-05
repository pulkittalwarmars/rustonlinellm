# Build stage
FROM rust:1.72 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04
# Update packages and install necessary dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/openai_api_rust /usr/local/bin/openai_api_rust
# Set the port environment variable
ENV PORT=8080
# Expose port 8080
EXPOSE 8080
# Set the log level for the Rust application
ENV RUST_LOG=info
# Command to run the executable
CMD ["openai_api_rust"]
