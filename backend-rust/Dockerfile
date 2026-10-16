# Build stage
FROM rust:1.81-slim AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# Run stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/backend-rust .

ENV PORT=8080
EXPOSE 8080

CMD ["./backend-rust"]
