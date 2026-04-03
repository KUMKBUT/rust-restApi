# Stage 1: Сборка
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from:builder /app/target/release/sweet-bonanza-backend .

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
CMD ["./sweet-bonanza-backend"]
