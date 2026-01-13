# 短线侠系统 - 部署文档

**版本:** 1.0
**更新日期:** 2025-01-13

---

## 📋 目录

1. [快速开始](#快速开始)
2. [部署模式](#部署模式)
3. [环境要求](#环境要求)
4. [常见问题](#常见问题)
5. [故障排查](#故障排查)
6. [维护操作](#维护操作)

---

## 🚀 快速开始

### 5 分钟快速部署

```bash
# 1. 克隆代码仓库
git clone <repository-url>
cd duanxianxia

# 2. 环境检查
./check-env.sh

# 3. 部署系统
./deploy.sh

# 4. 验证服务状态
./health-check.sh
```

**就这么简单!** 系统将自动:
- ✅ 检查环境依赖
- ✅ 启动数据库服务
- ✅ 编译并启动所有后端服务
- ✅ 运行健康检查

---

## 📦 部署模式

系统提供三种部署模式,适应不同场景:

### 1. 快速部署模式 (Quick)

**适用场景:** 开发过程中的快速迭代

```bash
./deploy.sh quick
# 或简写
./deploy.sh
```

**特点:**
- 🔄 停止并重启服务
- 💾 保留所有数据和配置
- ⚡ 速度快,通常 30 秒内完成
- 📝 自动备份配置文件

**执行流程:**
1. 停止现有服务
2. 重新编译(如有代码变更)
3. 启动所有服务
4. 运行健康检查

---

### 2. 完全部署模式 (Full)

**适用场景:** 首次部署或需要完全清理

```bash
./deploy.sh full
```

**特点:**
- 🧹 清理所有数据和容器
- 🔄 完全重新部署
- 🆕 初始化数据库
- ⚠️  **会删除所有数据!**

**执行流程:**
1. 确认操作(防止误操作)
2. 停止并清理所有服务
3. 清理 Docker 容器和数据卷
4. 启动数据库服务
5. 初始化数据库表
6. 编译并启动所有服务
7. 运行健康检查

**⚠️ 注意:**
- 此模式会删除所有数据,请谨慎使用
- 建议在执行前备份重要数据
- 适合首次部署或环境重置

---

### 3. 增量更新模式 (Update)

**适用场景:** 日常代码更新

```bash
./deploy.sh update
```

**特点:**
- 📥 拉取最新代码
- 🔨 增量编译(仅编译变更部分)
- 🔄 重启服务
- 💾 保留数据

**执行流程:**
1. Git pull 拉取最新代码
2. 停止现有服务
3. 增量编译各个服务
4. 启动所有服务
5. 运行健康检查

---

## 📐 环境要求

### 硬件要求

| 资源 | 最低配置 | 推荐配置 |
|------|---------|---------|
| CPU | 2 核 | 4 核+ |
| 内存 | 4 GB | 8 GB+ |
| 磁盘 | 10 GB | 20 GB+ SSD |

### 软件依赖

#### 必需软件

- **Docker:** 20.10+
  ```bash
  docker --version
  ```

- **Docker Compose:** 2.0+
  ```bash
  docker-compose --version
  ```

- **Rust:** 1.70+
  ```bash
  rustc --version
  cargo --version
  ```

- **Git:** 2.30+
  ```bash
  git --version
  ```

#### 可选软件

- **curl:** 用于 API 健康检查
- **lsof:** 用于端口检查

### 端口占用

系统需要以下端口可用:

| 端口 | 服务 | 用途 |
|------|------|------|
| 8080 | data-collector | 数据采集 API |
| 8082 | storage-service | 存储服务 API |
| 8083 | realtime-service | WebSocket 推送 |
| 8084 | auth-service | 认证服务 |
| 6379 | Redis | 缓存 |
| 5433 | PostgreSQL | 用户数据库 |
| 8123 | ClickHouse HTTP | 时序数据库 |
| 9000 | ClickHouse Native | 时序数据库 |

---

## ❓ 常见问题

### Q1: 部署失败怎么办?

**A:** 按以下步骤排查:

1. **查看部署日志:**
   ```bash
   tail -f logs/deploy-*.log
   ```

2. **检查环境:**
   ```bash
   ./check-env.sh
   ```

3. **检查服务状态:**
   ```bash
   ./health-check.sh
   ```

4. **查看具体服务日志:**
   ```bash
   tail -f logs/data-collector.log
   tail -f logs/storage-service.log
   ```

---

### Q2: 如何停止所有服务?

**A:** 使用停止脚本:

```bash
./stop-all.sh
```

或者手动停止:
```bash
# 停止 Docker 容器
docker-compose down

# 停止后端服务
./stop-all.sh
```

---

### Q3: 如何重置系统?

**A:** 使用重置脚本:

```bash
./reset-all.sh
```

**⚠️ 警告:** 这将删除所有数据!重置前系统会要求确认。

---

### Q4: 部署太慢怎么办?

**A:** 优化建议:

1. **使用快速部署模式:**
   ```bash
   ./deploy.sh quick
   ```

2. **跳过环境检查(仅限熟练用户):**
   ```bash
   ./deploy.sh --no-check
   ```

3. **使用增量编译:**
   ```bash
   ./deploy.sh update
   ```

---

### Q5: 如何查看服务日志?

**A:** 所有日志存储在 `logs/` 目录:

```bash
# 查看实时日志
tail -f logs/data-collector.log

# 查看最近 50 行
tail -n 50 logs/storage-service.log

# 查看错误日志
grep ERROR logs/*.log
```

---

### Q6: Docker 容器无法启动?

**A:** 排查步骤:

1. **检查 Docker 状态:**
   ```bash
   docker info
   ```

2. **查看容器日志:**
   ```bash
   docker logs clickhouse
   docker logs redis
   docker logs postgres
   ```

3. **重启 Docker:**
   ```bash
   sudo systemctl restart docker
   ```

---

### Q7: 端口被占用怎么办?

**A:** 查找并停止占用端口的进程:

```bash
# 查找占用端口的进程
lsof -ti:8080

# 停止进程
kill -9 $(lsof -ti:8080)

# 或使用 stop-all.sh 清理
./stop-all.sh
```

---

### Q8: 编译错误怎么办?

**A:** 常见解决方案:

1. **更新 Rust 工具链:**
   ```bash
   rustup update
   ```

2. **清理编译缓存:**
   ```bash
   cargo clean
   ```

3. **重新编译:**
   ```bash
   ./deploy.sh update
   ```

---

## 🔍 故障排查

### 问题 1: 服务启动失败

**症状:** `./deploy.sh` 执行后服务未运行

**排查步骤:**

1. 检查服务进程:
   ```bash
   ps aux | grep data-collector
   ps aux | grep storage-service
   ```

2. 检查端口监听:
   ```bash
   lsof -ti:8080
   lsof -ti:8082
   ```

3. 查看错误日志:
   ```bash
   tail -100 logs/data-collector.log | grep ERROR
   ```

**常见原因:**
- 配置文件缺失
- 数据库未启动
- 端口被占用
- 依赖缺失

---

### 问题 2: 数据库连接失败

**症状:** 日志中出现数据库连接错误

**排查步骤:**

1. 检查 Docker 容器:
   ```bash
   docker ps | grep clickhouse
   docker ps | grep redis
   docker ps | grep postgres
   ```

2. 测试连接:
   ```bash
   # ClickHouse
   docker exec -it $(docker ps -q -f name=clickhouse) clickhouse-client

   # PostgreSQL
   docker exec -it $(docker ps -q -f name=postgres) psql -U postgres

   # Redis
   docker exec -it $(docker ps -q -f name=redis) redis-cli ping
   ```

3. 检查网络:
   ```bash
   docker network ls
   docker network inspect duanxianxia_default
   ```

---

### 问题 3: 内存不足

**症状:** 服务频繁崩溃或重启

**解决方案:**

1. **检查内存使用:**
   ```bash
   free -h
   docker stats
   ```

2. **调整 Docker 内存限制:**
   - Docker Desktop: Settings > Resources > Memory
   - Linux: 调整 swap 大小

3. **优化服务配置:**
   - 减少并发数
   - 调整缓存大小
   - 禁用不必要的服务

---

## 🔧 维护操作

### 日常维护

#### 查看系统状态

```bash
# 健康检查
./health-check.sh

# 服务状态
docker-compose ps

# 进程状态
ps aux | grep -E "(data-collector|storage-service|realtime-service|auth-service)"
```

#### 日志管理

```bash
# 清理旧日志(保留最近 7 天)
find logs/ -name "*.log" -mtime +7 -delete

# 压缩日志
gzip logs/*.log

# 查看磁盘使用
du -sh logs/
```

#### 备份数据

```bash
# 备份配置
tar -czf backup/config-$(date +%Y%m%d).tar.gz services/*/.env

# 备份 ClickHouse 数据
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client \
    --query "SELECT * FROM stock_quotes" > backup/stock_quotes-$(date +%Y%m%d).csv

# 备份 PostgreSQL 数据
docker exec $(docker ps -q -f name=postgres) pg_dump -U postgres duanxianxia_users > backup/users-$(date +%Y%m%d).sql
```

### 性能优化

#### 清理 Docker 资源

```bash
# 清理未使用的镜像
docker image prune -a

# 清理未使用的容器
docker container prune

# 清理未使用的卷
docker volume prune

# 一键清理所有
docker system prune -a --volumes
```

#### 优化编译速度

```bash
# 使用 sccache 加速编译
cargo install sccache
export RUSTC_WRAPPER=sccache

# 使用增量编译
export CARGO_INCREMENTAL=1
```

---

## 📞 获取帮助

### 文档资源

- **完整文档:** `docs/deployment/`
- **设计文档:** `docs/plans/2025-01-13-deployment-system-design.md`
- **源代码注释:** 各服务源码中的文档注释

### 调试技巧

1. **启用详细日志:**
   ```bash
   export RUST_LOG=debug
   ./deploy.sh quick
   ```

2. **查看服务依赖:**
   ```bash
   cargo tree
   ```

3. **分析编译时间:**
   ```bash
   cargo build --timings
   ```

---

## 📝 附录

### A. 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| REDIS_URL | redis://localhost:6379 | Redis 连接地址 |
| CLICKHOUSE_URL | http://localhost:8123 | ClickHouse 地址 |
| POSTGRES_URL | postgresql://postgres:password@localhost:5433/duanxianxia_users | PostgreSQL 连接 |
| RUST_LOG | info | 日志级别 |
| BIND_ADDRESS | 127.0.0.1:PORT | 服务绑定地址 |

### B. 目录结构

```
duanxianxia/
├── deploy.sh              # 主部署脚本
├── check-env.sh           # 环境检查脚本
├── health-check.sh        # 健康检查脚本
├── start-all.sh           # 启动所有服务
├── stop-all.sh            # 停止所有服务
├── reset-all.sh           # 重置系统
├── logs/                  # 日志目录
│   ├── deploy-*.log      # 部署日志
│   ├── data-collector.log
│   ├── storage-service.log
│   ├── realtime-service.log
│   └── auth-service.log
├── backup/               # 备份目录
│   ├── config-*/
│   └── db-*/
├── services/             # 后端服务
│   ├── data-collector/
│   ├── storage-service/
│   ├── realtime-service/
│   └── auth-service/
└── docs/                 # 文档
    └── deployment/
        └── DEPLOYMENT.md
```

### C. 服务依赖关系

```
┌─────────────┐
│   Frontend  │
└──────┬──────┘
       │
       ↓
┌─────────────┐     ┌──────────────┐
│ auth-service│────→│  PostgreSQL  │
└─────────────┘     └──────────────┘
       │
       ↓
┌─────────────┐
│data-collector│───→┌──────────────┐
└─────────────┘    │   ClickHouse │
       │           └──────────────┘
       ↓           ┌──────────────┐
┌─────────────┐    │    Redis     │
│storage-service│   └──────────────┘
└─────────────┘
       │
       ↓
┌─────────────┐
│realtime-    │
│ service     │
└─────────────┘
```

---

**文档版本:** 1.0
**最后更新:** 2025-01-13
**维护者:** 短线侠开发团队
