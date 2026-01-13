# 短线侠系统 - 完整部署指南

**版本:** 1.0  
**更新日期:** 2026-01-13  
**适用人群:** 开发者、运维人员

---

## 📋 目录

1. [环境要求](#环境要求)
2. [快速部署](#快速部署)
3. [详细部署步骤](#详细部署步骤)
4. [健康检查](#健康检查)
5. [常见问题](#常见问题)
6. [故障排查](#故障排查)
7. [维护操作](#维护操作)

---

## 🔧 环境要求

### 必需软件

| 软件 | 最低版本 | 推荐版本 | 用途 |
|------|----------|----------|------|
| **Docker** | 20.10+ | 最新稳定版 | 运行数据库服务 |
| **Docker Compose** | 2.0+ | 最新稳定版 | 编排数据库容器 |
| **Rust** | 1.70+ | 最新稳定版 | 编译后端服务 |
| **Bash** | 4.0+ | 5.0+ | 运行部署脚本 |
| **Git** | 2.0+ | 最新稳定版 | 版本控制 |

### 系统要求

- **操作系统:** Linux、macOS、WSL2
- **CPU:** 2 核心以上
- **内存:** 4GB 以上（推荐 8GB）
- **磁盘:** 10GB 可用空间
- **网络:** 能访问 crates.io 和 Docker Hub

### 端口要求

确保以下端口未被占用：

| 端口 | 服务 | 说明 |
|------|------|------|
| 8080 | realtime-service | WebSocket 实时推送 |
| 8082 | auth-service | 认证服务 |
| 8083 | storage-service | 数据存储服务 |
| 8084 | auction-storage | 竞价数据存储（可选） |
| 8085 | auction-realtime | 竞价实时推送（可选） |
| 6379 | Redis | 消息队列 |
| 5433 | PostgreSQL | 用户数据库 |
| 8123 | ClickHouse | HTTP 接口 |
| 9000 | ClickHouse | Native 接口 |

---

## 🚀 快速部署

### 5分钟快速部署

> **前提:** 已安装 Docker、Docker Compose、Rust

```bash
# 1. 启动所有服务
bash ./start-all.sh

# 2. 验证服务状态
bash ./health-check.sh

# 3. 测试 API
curl http://localhost:8082/api/auth/login -X POST \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

**启动脚本会自动：**
- ✅ 检查环境依赖
- ✅ 停止占用端口的旧进程
- ✅ 启动数据库容器
- ✅ 初始化数据库表结构
- ✅ 创建配置文件
- ✅ 编译并启动所有后端服务
- ✅ 显示服务状态和常用命令

---

## 📖 详细部署步骤

### 步骤 1: 环境检查

**1.1 检查 Docker**

```bash
docker info
docker --version
```

**1.2 检查 Rust 工具链**

```bash
rustc --version
cargo --version
```

**1.3 检查 Shell 类型**

```bash
echo $SHELL
# 应显示: /bin/bash
# 如果是 zsh，需要使用: bash ./start-all.sh
```

---

### 步骤 2: 启动基础设施

```bash
# 启动数据库
docker-compose up -d redis clickhouse postgres

# 等待数据库启动
sleep 10

# 验证容器状态
docker-compose ps
```

---

### 步骤 3: 配置环境变量

```bash
# 自动创建所有配置文件
for service in services/*/; do
    if [ -f "$service/.env.example" ]; then
        cp "$service/.env.example" "$service/.env"
    fi
done
```

---

### 步骤 4: 启动后端服务

```bash
# 使用启动脚本（推荐）
bash ./start-all.sh
```

---

### 步骤 5: 验证部署

```bash
# 运行健康检查
bash ./health-check.sh

# 查看日志
tail -f logs/data-collector.log
```

---

## 🏥 健康检查

### 自动健康检查

```bash
bash ./health-check.sh
```

检查内容包括:
- Docker 容器状态
- 后端服务进程状态
- API 端点响应
- 数据库连接

---

## ❓ 常见问题

### 1. Shell 兼容性问题 ⭐

**问题:** `./start-all.sh` 报错 `unknown file attribute`

**原因:** 脚本在 zsh 环境中运行

**解决方案:**
```bash
bash ./start-all.sh
```

### 2. Redis Stream 无数据

**问题:** 查询 Redis 返回 0 条数据

**原因:** 非交易时段，服务休眠（这是正常的）

**解决方案:**
- 查看日志确认: `tail -f logs/data-collector.log`
- 看到 "【非交易时段】进入休眠" 是正常的
- 在交易时段（09:30-15:00）会自动采集

### 3. 端口被占用

**问题:** 端口 8080-8085 被占用

**解决方案:**
```bash
# 查看占用端口的进程
lsof -ti:8083

# 或使用启动脚本，会自动停止旧进程
bash ./start-all.sh
```

---

## 🔧 故障排查

详细的故障排查指南请参考: [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)

---

## 🛠️ 维护操作

### 停止服务

```bash
bash ./stop-all.sh
```

### 完全重置

```bash
bash ./stop-all.sh
docker-compose down -v
rm -rf logs/*.log logs/*.pid
bash ./start-all.sh
```

### 查看日志

```bash
tail -f logs/data-collector.log
tail -f logs/storage-service.log
tail -f logs/realtime-service.log
tail -f logs/auth-service.log
```

---

## 📚 更多文档

- [系统架构文档](./ARCHITECTURE.md)
- [故障排查指南](./TROUBLESHOOTING.md)
- [部署测试报告](./DEPLOYMENT_FLOW_TEST.md)
- [用户使用指南](./USER_GUIDE.md)
