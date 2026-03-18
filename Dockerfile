FROM rust:1.86-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p vigil-cli

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/vigil-cli /usr/local/bin/vigil
COPY migrations ./migrations
COPY demo ./demo
COPY scripts ./scripts
EXPOSE 8080
CMD ["vigil", "daemon", "--port", "8080", "--db-path", "/data/vigil_data", "--incident-db", "/data/vigil.db"]
