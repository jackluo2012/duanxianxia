# 构建阶段
FROM rust:1.83 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services/auction-service ./services/auction-service

# 构建 auction-service
WORKDIR /app/services/auction-service
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/services/auction-service/target/release/auction-service /app/auction-service

EXPOSE 8080

CMD ["./auction-service"]
