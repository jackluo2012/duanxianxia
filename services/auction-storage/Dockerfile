# 构建阶段
FROM rust:1.83 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services/auction-storage ./services/auction-storage

# 构建 auction-storage
WORKDIR /app/services/auction-storage
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/services/auction-storage/target/release/auction-storage /app/auction-storage

EXPOSE 8084

CMD ["./auction-storage"]
