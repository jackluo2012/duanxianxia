# 短线侠 - A股实时行情分析平台

基于 Rust 后端和 React 前端的股票实时行情分析平台。

## 技术栈

**后端：**
- Rust
- Actix-web (Web 框架)
- ClickHouse (时序数据库)
- Redis (消息队列)
- PostgreSQL (用户数据库)
- WebSocket (实时推送)
- rustdx (A股数据源)

**前端：**
- React 18
- TypeScript
- Vite
- Ant Design 5
- React Router 6
- ECharts (分时图 + K线图)

## 微服务架构

1. **data-collector** - 数据采集服务
   - 从 rustdx 获取实时行情
   - 推送到 Redis Stream
   - 端口: 无

2. **storage-service** - 存储服务
   - 订阅 Redis Stream
   - 批量写入 ClickHouse
   - HTTP API 提供历史数据查询
   - 端口: 8083

3. **realtime-service** - 实时推送服务
   - WebSocket 服务
   - 订阅 Redis Stream 并广播到客户端
   - 端口: 8080

4. **auth-service** - 认证服务
   - 用户注册/登录
   - JWT 认证
   - 端口: 8082

## 快速开始

### 一键启动 (推荐)

```bash
# 启动所有服务
./start-all.sh

# 测试数据流转
./test-data-flow.sh

# 停止所有服务
./stop-all.sh
```

### 手动启动

#### 1. 启动基础设施

```bash
docker-compose up -d redis clickhouse postgres
```

#### 2. 初始化数据库

```bash
# ClickHouse
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client < db/init.sql

# PostgreSQL
docker exec -i $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users < db/init_postgres.sql
```

#### 3. 启动后端服务

```bash
# 终端1: 数据采集
cd services/data-collector && cargo run

# 终端2: 存储服务
cd services/storage-service && cargo run

# 终端3: WebSocket 服务
cd services/realtime-service && cargo run

# 终端4: 认证服务
cd services/auth-service && cargo run
```

#### 4. 启动前端

```bash
cd frontend
npm install
npm run dev
```

访问 http://localhost:3000

## API 端点

- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录
- `WS /ws/realtime` - WebSocket 实时推送

## 测试用户

- 用户名: `testuser`
- 密码: `password123`

## 数据流转说明

### 完整数据流

```
rustdx数据源
    ↓
data-collector (采集)
    ↓
Redis Stream (消息队列)
    ↓
    ├─→ storage-service (持久化到 ClickHouse)
    └─→ realtime-service (WebSocket广播到前端)
```

### 验证数据流转

运行测试脚本:
```bash
./test-data-flow.sh
```

手动检查:
```bash
# 查看 Redis Stream 数据
docker exec $(docker ps -q -f name=redis) redis-cli XLEN stock_quotes
docker exec $(docker ps -q -f name=redis) redis-cli XRANGE stock_quotes - + COUNT 5

# 查看 ClickHouse 数据
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT * FROM stock_quotes ORDER BY datetime DESC LIMIT 10"
```

## 日志

使用一键启动时,日志保存在 `logs/` 目录:
- `logs/data-collector.log` - 数据采集服务
- `logs/storage-service.log` - 存储服务
- `logs/realtime-service.log` - 实时推送服务
- `logs/auth-service.log` - 认证服务

查看实时日志:
```bash
tail -f logs/data-collector.log
tail -f logs/storage-service.log
tail -f logs/realtime-service.log
tail -f logs/auth-service.log
```

## 开发状态

✅ MVP Phase 1 已完成 (17/17 tasks)

### 最新更新

- ✅ 完成 storage-service 的 ClickHouse 批量写入逻辑
- ✅ 完成 realtime-service 的 Redis Stream 订阅和 WebSocket 广播
- ✅ 添加一键启动脚本 `start-all.sh`
- ✅ 添加测试脚本 `test-data-flow.sh`

### 已知问题

无

## 下一步计划

- [ ] 前端API封装 (frontend/src/api/auth.ts)
- [ ] 前端路由守卫
- [ ] 竞价分析模块
- [ ] 数据挖掘模块
- [ ] 复盘模块

## License

MIT

## 功能特性

### ✅ 已实现功能

1. **实时数据采集**
   - 每3秒采集A股行情数据
   - 支持多只股票同时采集
   - Redis Stream 消息队列

2. **数据持久化**
   - ClickHouse 批量写入（100条或5秒）
   - 支持历史数据查询

3. **实时WebSocket推送**
   - 前端自动连接并订阅股票
   - 断线自动重连
   - 实时更新行情数据

4. **多周期K线图表** 🆕
   - 分时图（3秒实时数据）
   - 5分钟K线（OHLC蜡烛图）
   - 日K线（每日OHLC）
   - ECharts 可视化展示

5. **用户认证**
   - 用户注册/登录
   - JWT Token 认证

## API 端点

### 数据查询

```bash
# 分时图（默认）
GET http://localhost:8083/api/quotes/000001/history?period=1m

# 5分钟K线
GET http://localhost:8083/api/quotes/000001/history?period=5m

# 日K线
GET http://localhost:8083/api/quotes/000001/history?period=1d
```

### WebSocket

```
ws://localhost:8080/ws/realtime

// 订阅股票
{
  "action": "subscribe",
  "codes": ["000001", "600000"]
}
```

## 数据流

```
rustdx (数据源)
  → data-collector (采集)
  → Redis Stream (队列)
  → storage-service (持久化 + HTTP API)
  → realtime-service (WebSocket广播)
  → 前端 (实时展示)
```

## 开发进度

- [x] Phase 1 MVP - 基础架构和实时行情 (17/17)
- [x] 多周期K线切换功能
- [ ] Phase 2 - 竞价分析模块
- [ ] Phase 3 - 数据挖掘模块

