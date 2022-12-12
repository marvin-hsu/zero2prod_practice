FROM rust:1.65.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release
ENTRYPOINT ["./target/release/zero2prod_practice"]
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod_practice zero2prod_practice
ENTRYPOINT ["./zero2prod_practice"]