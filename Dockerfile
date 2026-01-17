FROM rust:1.92-slim-bookworm AS builder

RUN echo "Starting build stage."
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-musl

WORKDIR /app

COPY . .

RUN cargo build --release
RUN echo "Finished building the Rust application."

FROM ubuntu:24.04

RUN echo "Starting final stage setup."

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/auth_service ./auth-service

RUN echo "Setup complete. Ready to run the application."

EXPOSE 8080
CMD ["./auth-service"]