# 多阶段构建 Dockerfile for backtest-service

# 构建阶段
FROM rust:1.75-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# 设置工作目录
WORKDIR /app

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./

# 创建虚拟源目录来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制实际源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM alpine:3.19

# 安装运行时依赖
RUN apk add --no-cache ca-certificates

# 创建非 root 用户
RUN addgroup -S backtest && \
    adduser -S backtest -G backtest

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/backtest-service /app/

# 更改文件所有者
RUN chown -R backtest:backtest /app

# 切换到非 root 用户
USER backtest

# 暴露端口
EXPOSE 8086

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8086/health || exit 1

# 设置环境变量
ENV CLICKHOUSE_URL=http://localhost:8123
ENV RUST_LOG=info

# 启动应用
CMD ["./backtest-service"]
