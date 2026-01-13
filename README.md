# 短线侠 - A股实时行情分析平台

基于 Rust 后端和 React 前端的股票实时行情分析平台。

## 📚 文档导航

- **[快速入门](./docs/QUICK_START.md)** - 5 分钟快速部署体验
- **[部署安装文档](./docs/DEPLOYMENT.md)** - 详细部署步骤和故障排查
- **[用户使用指南](./docs/USER_GUIDE.md)** - 功能说明和最佳实践
- **[系统架构文档](./docs/ARCHITECTURE.md)** - 技术架构和设计
- **[故障排查指南](./docs/TROUBLESHOOTING.md)** - 常见问题及解决方案 ⭐
- **[部署测试报告](./docs/DEPLOYMENT_TEST_REPORT.md)** - 真实数据测试验证 ⭐

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

### 核心服务

1. **data-collector** - 数据采集服务
   - 从 rustdx 获取实时行情
   - 推送到 Redis Stream
   - 智能调度器（交易时段3秒/次，盘后5分钟/次）
   - K线数据聚合（3秒 → 5分钟/日线）
   - 历史数据回填
   - 数据纠错和补全
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

### 竞价分析模块 🆕

5. **auction-service** - 竞价数据采集服务
   - 时序检查 (9:15-9:25 竞价时段)
   - 抢筹强度评分算法 (0-100)
   - 封单金额计算 (买封/卖封)
   - 推送到 Redis Stream: `auction_quotes`
   - 端口: 无 (后台任务)

6. **auction-storage** - 竞价数据存储服务
   - 订阅 Redis Stream `auction_quotes`
   - 批量写入 ClickHouse (100条或5秒)
   - HTTP API:
     - `GET /api/auction/rankings?type={type}&limit={limit}` - 排行榜查询
     - `GET /api/auction/details/{code}` - 详情查询
     - `GET /health` - 健康检查
     - `POST /api/auction/alerts` - 创建告警规则 🆕
     - `GET /api/auction/alerts` - 获取告警规则列表 🆕
     - `DELETE /api/auction/alerts/{id}` - 删除告警规则 🆕
     - `GET /api/auction/alerts/history` - 告警历史 🆕
     - `POST /api/auction/watchlist` - 添加自选股 🆕
     - `GET /api/auction/watchlist` - 获取自选股列表 🆕
     - `DELETE /api/auction/watchlist/{code}` - 删除自选股 🆕
     - `GET /api/auction/watchlist/{code}/check` - 检查是否在自选中 🆕
   - 端口: 8084

7. **auction-realtime** - 竞价实时推送服务
   - WebSocket 服务器
   - 订阅 Redis Stream `auction_quotes`
   - 基于订阅的智能广播
   - 端口: 8085

### 前端页面

1. **实时行情** (`/`) - 分时图和K线图
2. **竞价分析** (`/auction`) - 竞价排行榜和详情 ⭐
   - 4种排行榜 (买封/强度/涨幅/异动)
   - 竞价曲线图 (价格 + 封单量)
   - 实时数据更新 (每5秒)
   - **告警配置** 🆕 - 自定义告警规则
   - **告警历史** 🆕 - 查看告警记录
   - **自选股管理** 🆕 - 管理关注股票

## 快速开始

### ⚡ 5分钟快速部署

> **重要提示:** 所有部署脚本必须在 **bash** 环境中运行，不支持 zsh。

```bash
# 1. 启动所有服务（自动完成所有配置和初始化）
bash ./start-all.sh

# 2. 验证服务状态
bash ./health-check.sh

# 3. 测试 API
curl http://localhost:8082/api/auth/login -X POST \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

**就这么简单！** 系统将自动:
- ✅ 检查环境依赖（Docker、Rust、端口）
- ✅ 启动基础设施数据库（Redis、ClickHouse、PostgreSQL）
- ✅ 初始化数据库表结构
- ✅ 自动创建配置文件
- ✅ 编译并启动所有后端服务

### 📋 常用命令

```bash
# 停止所有服务
bash ./stop-all.sh

# 查看服务日志
tail -f logs/data-collector.log   # 数据采集服务
tail -f logs/storage-service.log  # 存储服务
tail -f logs/realtime-service.log # 实时推送服务
tail -f logs/auth-service.log     # 认证服务

# 健康检查
bash ./health-check.sh

# 完全重置（清理所有数据）
./stop-all.sh
docker-compose down -v
rm -rf logs/*.log logs/*.pid
bash ./start-all.sh
```

### ⚠️ 重要说明

**Shell 兼容性:**
- 脚本必须在 **bash** 环境中运行：`bash ./start-all.sh`
- 不支持 zsh，如果遇到错误请检查当前 shell 类型

**数据采集时间:**
- **交易时段 (09:30-15:00):** 每 3 秒采集一次实时行情
- **竞价时段 (09:15-09:25):** 实时采集竞价数据
- **非交易时段:** 服务自动休眠（这是正常行为）

**查看日志确认状态:**
```bash
# 查看调度器状态
tail -f logs/data-collector.log | grep "调度"

# 看到 "【非交易时段】进入休眠" 是正常的
# 看到 "【交易时段】开始高频采集" 表示正在采集
```

### 🌐 启动前端

```bash
cd frontend
npm install  # 首次运行需要
npm run dev
```

访问:
- 实时行情: http://localhost:5173
- 竞价分析: http://localhost:5173/auction

### 📖 详细文档

- **[完整部署指南](./docs/DEPLOYMENT.md)** - 详细部署步骤、环境要求、故障排查 ⭐
- **[故障排查指南](./docs/TROUBLESHOOTING.md)** - 常见问题及解决方案 ⭐
- **[系统架构文档](./docs/ARCHITECTURE.md)** - 技术架构和设计
- **[部署测试报告](./docs/DEPLOYMENT_FLOW_TEST.md)** - 真实部署测试验证

## API 端点

### 认证服务 (Port 8082)

- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录

### 存储服务 (Port 8083)

#### 实时行情
- `GET /api/quotes/{code}/history?period={period}` - 历史行情查询
  - period: `1m` (1分钟), `5m` (5分钟), `1d` (日线)
  - 示例: `/api/quotes/000001/history?period=5m&start=2026-01-01&end=2026-01-03`

#### K线数据
- `GET /api/kline/{code}?period={period}` - K线数据查询
  - period: `5m`, `1d`
  - 示例: `/api/kline/000001?period=5m&limit=100`

#### 健康检查
- `GET /health` - 服务健康状态

### 竞价存储服务 (Port 8084)

#### 竞价数据
- `GET /api/auction/rankings?type={type}&limit={limit}` - 竞价排行榜
  - type: `buy_seal` (买封), `strength` (强度), `change` (涨幅), `abnormal` (异动)
  - 示例: `/api/auction/rankings?type=buy_seal&limit=20`

- `GET /api/auction/details/{code}` - 竞价详情
  - 示例: `/api/auction/details/000001`

#### 告警规则
- `POST /api/auction/alerts` - 创建告警规则
- `GET /api/auction/alerts` - 获取告警规则列表
- `DELETE /api/auction/alerts/{id}` - 删除告警规则
- `GET /api/auction/alerts/history` - 告警历史

#### 自选股管理
- `POST /api/auction/watchlist` - 添加自选股
- `GET /api/auction/watchlist` - 获取自选股列表
- `DELETE /api/auction/watchlist/{code}` - 删除自选股
- `GET /api/auction/watchlist/{code}/check` - 检查是否在自选中

#### 健康检查
- `GET /health` - 服务健康状态

### WebSocket

#### 实时推送服务 (Port 8080)
- `WS /ws/realtime` - WebSocket 实时推送

```javascript
// 订阅股票
ws.send(JSON.stringify({
  action: "subscribe",
  codes: ["000001", "600000"]
}));

// 取消订阅
ws.send(JSON.stringify({
  action: "unsubscribe",
  codes: ["000001"]
}));
```

#### 竞价推送服务 (Port 8085)
- `WS /ws/auction` - 竞价数据实时推送

```javascript
// 订阅竞价数据
ws.send(JSON.stringify({
  action: "subscribe",
  codes: ["000001", "600000"]
}));
```

**测试账号:**
- 用户名: `testuser`
- 密码: `password123`

## 系统特性

### 智能过滤
- ✅ **自动过滤非股票数据**：仅保留真实的 A 股股票（约 4700 只）
- ✅ **深市股票**：000xxx（主板）、002xxx（中小板）、300xxx（创业板）
- ✅ **沪市股票**：600xxx/601xxx/603xxx（主板）、688xxx（科创板）
- ❌ 过滤：基金、ETF、转债、理财等金融产品

### 时区支持
- ✅ **北京时间存储**：所有 DateTime 字段使用 `Asia/Shanghai` 时区
- ✅ **自动时间转换**：UTC 时间自动转换为北京时间（UTC+8）
- ✅ **一致的时间显示**：确保数据时间和日志时间一致

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

✅ Phase 2 Week 2 完成 (20/20 tasks) - 100% 🎉
✅ Phase 2 Week 1 完成 (21/21 tasks) - 100% 🎉
✅ Phase 1 MVP 完成 (17/17 tasks) - 100% 🎉

### 最新功能 (2026-01-03)

**智能调度系统 (Task 12-15)**
- ✅ 交易时段智能切换（3秒/次）
- ✅ 盘后时段降频（5分钟/次）
- ✅ 节假日自动暂停
- ✅ 减少无效请求90%+

**K线数据管理 (Task 16-19)**
- ✅ 5分钟K线实时聚合
- ✅ 日K线收盘更新
- ✅ 历史数据批量回填
- ✅ 缺失K线自动修复
- ✅ 异常K线数据纠错

**历史数据API (Task 20-21)**
- ✅ 查询服务集成
- ✅ ClickHouse直接查询
- ✅ 多周期支持（1m/5m/1d）
- ✅ 性能优化（< 100ms）

### 最新更新 🆕

**Day 5: 功能完善与优化 (2026-01-01)**

- ✅ **告警系统** (Task 5.1)
  - 后端：AlertManager 核心管理类（397行）
  - 4种告警规则类型：价格涨幅、封单金额、强度评分、异动检测
  - 告警风暴抑制（5分钟最多3次）
  - 前端：告警配置和历史页面
  - 单元测试：6/6 通过

- ✅ **自选股管理** (Task 5.2)
  - 后端：WatchlistManager + REST API（191行）
  - 默认自选股池：15只沪深300成分股
  - 前端：自选股管理UI组件（135行）
  - 集成测试：API 全部通过

- ✅ **集成测试** (Task 5.3)
  - 完整数据流测试：采集 → 存储 → 推送 → 展示
  - 并发测试：10+ 并发请求
  - 边界条件测试：空数据、无效输入、重复添加
  - 测试通过率：8/9 (89%)

- ✅ **性能优化** (Task 5.4)
  - Rust Release 优化配置（LTO、codegen-units=1）
  - 性能优化文档（ClickHouse、WebSocket、前端）
  - API 响应时间： < 100ms

- ✅ **文档更新** (Task 5.5)
  - 更新 README：新增 API 端点和功能说明
  - 性能优化方案文档
  - 集成测试脚本

### 已知问题

无

## 系统架构

详细架构文档请参阅：[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

### 核心特性

1. **智能调度系统** 🆕
   - 交易时段高频采集（3秒/次）
   - 盘后时段降频采集（5分钟/次）
   - 节假日自动暂停
   - CPU使用率降低60%+

2. **K线数据管理** 🆕
   - 实时聚合5分钟K线
   - 日K线自动更新
   - 历史数据批量回填
   - 缺失数据自动修复
   - 异常数据智能纠错

3. **数据质量监控** 🆕
   - 完整性检查（预期vs实际）
   - 有效性验证（价格/OHLC/涨跌幅）
   - 异常数据日志记录
   - 数据修复审计追踪

4. **高性能查询**
   - ClickHouse批量写入优化
   - API响应时间 < 100ms
   - 缓存命中率 > 90%

## 性能指标

- **采集延迟**: < 3秒（交易时段）
- **API响应**: < 100ms（P95）
- **缓存命中**: > 90%
- **数据完整性**: > 99.9%
- **系统可用性**: > 99.5%

详细性能基准测试：[docs/PERFORMANCE.md](docs/PERFORMANCE.md)

## 下一步计划

- [x] Phase 2 Week 1 - 竞价分析模块
- [x] Phase 2 Week 2 - 数据质量监控与优化
- [ ] Phase 3 Week 3 - 数据回测与策略模块
  - [ ] 历史数据回测引擎
  - [ ] 策略配置和回测
  - [ ] 策略绩效评估

- [ ] Phase 3 - 社区功能和移动端
  - [ ] 用户分享和讨论
  - [ ] 移动端适配
  - [ ] 推送通知

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

