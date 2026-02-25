# 多阶段构建 - 构建阶段
FROM rust:1.75-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev protobuf-dev

# 设置工作目录
WORKDIR /app

# 复制Cargo配置
COPY Cargo.toml Cargo.lock ./
COPY shared ../shared/

# 复制服务源码
COPY . .

# 构建服务（使用musl静态链接）
RUN cargo build --release

# 运行阶段 - 使用最小镜像
FROM alpine:3.19

# 安装运行时依赖
RUN apk add --no-cache ca-certificates

# 创建非root用户
RUN addgroup -g 1000 gateway && \
    adduser -D -u 1000 -G gateway gateway

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/gateway-service /app/gateway-service

# 更改文件所有权
RUN chown -R gateway:gateway /app

# 切换到非root用户
USER gateway

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# 启动服务
CMD ["./gateway-service"]
