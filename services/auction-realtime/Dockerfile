# 构建阶段
FROM rust:1.83 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY services/auction-realtime ./services/auction-realtime

# 构建 auction-realtime
WORKDIR /app/services/auction-realtime
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/services/auction-realtime/target/release/auction-realtime /app/auction-realtime

EXPOSE 8085

CMD ["./auction-realtime"]
