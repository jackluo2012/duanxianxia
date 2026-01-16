# 短线侠平台 - A股短线交易分析系统

基于 **Rust** 和 **六边形架构** 的专业 A 股短线交易平台。

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## ⚡ 快速开始（5分钟启动）

### 前置要求

- **Docker** 20.10+ （必需）
- **Docker Compose** （必需）
- **Bash** shell 环境
- **Rust** 1.75+ （可选，仅开发需要）

### 一键启动

```bash
# 1. 克隆项目
git clone https://github.com/your-org/duanxianxia.git
cd duanxianxia

# 2. 启动所有服务
bash ./start-all.sh

# 3. 验证服务状态
bash ./health-check.sh
```

**就这么简单！** 系统将自动：
- ✅ 启动 Docker 数据库（Redis, ClickHouse, PostgreSQL）
- ✅ 初始化数据库表结构
- ✅ 编译并启动所有后端服务
- ✅ 健康检查验证

### 访问服务

| 服务 | 端口 | 健康检查 | 说明 |
|------|------|----------|------|
| auction-storage | 8084 | http://localhost:8084/health | 竞价数据存储 |
| auth-service | 8082 | http://localhost:8082/health | 用户认证 |
| query-service | 8089 | http://localhost:8089/health | 选股查询 |
| realtime-service | 8090 | http://localhost:8090/health | 实时行情推送 |
| limit-review-service | 8088 | http://localhost:8088/health | 涨停复盘 |

---

## 📚 完整文档

- **[部署文档](./DEPLOYMENT.md)** ⭐ - 详细的部署和运维指南
- **[使用文档](./USAGE.md)** - API 使用指南和示例

---

## 🏗️ 系统架构

### 微服务列表

| 服务 | 端口 | 数据库 | 功能 |
|------|------|--------|------|
| **auction-realtime** | - | Redis | 集合竞价实时推送 |
| **auction-service** | - | Redis | 竞价数据分析 |
| **auction-storage** | 8084 | ClickHouse | 竞价数据存储 |
| **auth-service** | 8082 | PostgreSQL | 用户认证授权 |
| **backtest-service** | - | ClickHouse | 策略回测引擎 |
| **data-collector** | - | ClickHouse | 全维度数据采集 |
| **kline-collector** | - | ClickHouse | K线数据采集 |
| **limit-review-service** | 8088 | ClickHouse | 涨停板复盘分析 |
| **query-service** | 8089 | ClickHouse | 选股器和查询 |
| **realtime-service** | 8090 | Redis | 实时行情推送 |
| **storage-service** | 8083 | ClickHouse | 通用存储服务 |

### 技术栈

**后端:**
- **语言**: Rust 1.75+
- **架构**: 六边形架构（Hexagonal Architecture）
- **Web 框架**: Actix-Web 4.9
- **时序数据库**: ClickHouse 24.11
- **关系数据库**: PostgreSQL 15
- **缓存**: Redis 7

---

## 💻 快速测试

### 1. 测试竞价数据

```bash
# 查询竞价排行榜
curl "http://localhost:8084/api/auction/rankings?type=buy_seal&limit=10"

# 查询股票详情
curl "http://localhost:8084/api/auction/details/000001"
```

### 2. 测试认证服务

```bash
# 注册用户
curl -X POST "http://localhost:8082/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"123456"}'

# 登录
curl -X POST "http://localhost:8082/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"123456"}'
```

### 3. 测试查询服务

```bash
# 查询龙头股票
curl "http://localhost:8089/api/screener/leaders?date=2025-01-16&limit=10"

# 查询技术指标
curl "http://localhost:8089/api/indicators/000001"
```

### 4. 测试涨停复盘

```bash
# 每日涨停复盘
curl "http://localhost:8088/api/review/2025-01-16"
```

---

## 🛠️ 常用命令

### 停止服务

```bash
# 停止所有服务
bash ./stop-all.sh

# 停止 Docker 数据库
docker-compose down
```

### 查看日志

```bash
# 实时查看所有日志
tail -f logs/*.log

# 查看特定服务日志
tail -f logs/auth-service.log
tail -f logs/query-service.log
```

### 健康检查

```bash
# 运行健康检查
bash ./health-check.sh

# 手动检查单个服务
curl http://localhost:8089/health
curl http://localhost:8088/health
```

### 开发模式

```bash
# 编译所有服务
cargo build --workspace

# 编译单个服务
cargo build -p query-service

# 运行单个服务（开发模式）
cargo run -p query-service
```

---

## 📊 项目统计

- **服务数量**: 11 个微服务
- **代码量**: 25,453 行
- **架构模式**: 六边形架构（Hexagonal Architecture）
- **编译状态**: ✅ 0 个错误

---

## ⚠️ 重要说明

### 数据采集时间

- **交易时段** (09:30-15:00): 每 3 秒采集一次
- **竞价时段** (09:15-09:25): 实时采集
- **非交易时段**: 服务休眠（正常现象）

### Shell 兼容性

- **必须使用 bash**: `bash ./start-all.sh`
- **不支持 zsh**: 脚本会自动检测并提示

### Docker 要求

- 确保 Docker Desktop 已启动
- 确保有足够的内存（建议 8GB+）

---

## 🐛 故障排查

### 问题：端口被占用

```bash
# 查看占用端口的进程
lsof -ti:8089

# 停止进程
kill -9 $(lsof -ti:8089)

# 或运行 start-all.sh 会自动清理
```

### 问题：Docker 未运行

```bash
# 启动 Docker Desktop
# macOS: 打开 Docker 应用
# Linux: sudo systemctl start docker
```

### 问题：服务启动失败

```bash
# 查看日志
tail -f logs/*.log

# 检查数据库状态
docker-compose ps

# 重启所有服务
bash ./stop-all.sh
bash ./start-all.sh
```

---

## 📖 更多文档

- **[部署文档](./DEPLOYMENT.md)** - 详细部署步骤、生产环境配置
- **[使用文档](./USAGE.md)** - 完整的 API 文档和使用示例

---

## 🤝 贡献

欢迎贡献代码！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

---

## 📄 许可证

MIT License

---

## 🎉 致谢

感谢 **Claude Code** 在六边形架构迁移中的大力支持！

---

**项目状态**: ✅ 生产就绪
**最后更新**: 2025-01-16
**版本**: v1.0.0
