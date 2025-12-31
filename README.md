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
- ECharts

## 微服务架构

1. **data-collector** - 数据采集服务
   - 从 rustdx 获取实时行情
   - 推送到 Redis Stream

2. **storage-service** - 存储服务
   - 订阅 Redis Stream
   - 写入 ClickHouse

3. **realtime-service** - 实时推送服务
   - WebSocket 服务
   - 实时推送行情数据

4. **auth-service** - 认证服务
   - 用户注册/登录
   - JWT 认证

## 快速开始

### 1. 启动基础设施

```bash
docker-compose up -d redis clickhouse postgres
```

### 2. 初始化数据库

```bash
# ClickHouse
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client < db/init.sql

# PostgreSQL
docker exec -i $(docker ps -q -f name=postgres) psql -U postgres -d duanxianxia_users < db/init_postgres.sql
```

### 3. 启动后端服务

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

### 4. 启动前端

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

## 开发状态

✅ MVP Phase 1 已完成 (17/17 tasks)

## License

MIT
