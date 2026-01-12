# Backtest Service

数据回测与策略模块 - 支持竞价选股策略和盘中短线策略的回测服务。

## 功能特性

- ✅ **轻量级设计** - 基于 ClickHouse 直接回测,无需额外数据存储
- ✅ **3种策略模板** - 竞价龙头、竞价封单、盘中突破
- ✅ **完整评估** - 收益指标、交易效率、风险控制
- ✅ **异步任务** - 支持并发回测任务
- ✅ **REST API** - 简单易用的 HTTP 接口
- ✅ **CLI 工具** - 命令行接口支持
- ✅ **Prometheus 监控** - 完整的指标收集
- ✅ **配置热重载** - 无需重启更新配置
- ✅ **数据库迁移** - 版本化管理数据库变更

## 技术栈

- **Rust 2021** - 高性能类型安全
- **Actix-web 4.4** - Web 框架
- **ClickHouse 0.12** - 时序数据库
- **Tokio** - 异步运行时
- **Prometheus** - 监控指标
- **Clap 4.4** - CLI 框架
- **Notify 6.0** - 文件监视

## 快速开始

### 环境要求

- Rust 1.70+
- ClickHouse 24.11+
- Docker & Docker Compose (可选)

### 编译运行

```bash
# 编译
cargo build --release

# 设置 ClickHouse URL
export CLICKHOUSE_URL="http://localhost:8123"

# 运行服务
cargo run
```

服务将在 `http://localhost:8086` 启动。

### Docker 部署

```bash
# 构建镜像
docker build -t backtest-service:latest .

# 运行容器
docker run -d \
  -p 8086:8086 \
  -e CLICKHOUSE_URL=http://clickhouse:8123 \
  --name backtest-service \
  backtest-service:latest
```

## API 文档

### 健康检查

```
GET /health
```

响应:
```json
{
  "status": "ok",
  "service": "backtest-service"
}
```

### 获取策略列表

```
GET /api/backtest/strategies
```

响应:
```json
{
  "strategies": [
    {
      "id": "auction_leader",
      "name": "竞价龙头策略",
      "description": "竞价强度评分>80且买封金额>1000万",
      "params": [
        {
          "name": "min_strength_score",
          "type": "integer",
          "default": 80,
          "description": "最低强度评分 (0-100)"
        }
      ]
    }
  ]
}
```

### 启动回测

```
POST /api/backtest/run
Content-Type: application/json

{
  "strategy_type": "auction_leader",
  "strategy_params": {
    "min_strength_score": 80,
    "min_buy_seal_amount": 1000,
    "holding_days": 1
  },
  "backtest_period": {
    "start_date": "2025-10-01",
    "end_date": "2025-10-31"
  },
  "initial_capital": 100000,
  "commission_rate": 0.0003
}
```

响应:
```json
{
  "backtest_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "status": "running",
  "estimated_time": 30
}
```

### 查询回测结果

```
GET /api/backtest/{backtest_id}
```

响应 (完成状态):
```json
{
  "backtest_id": "...",
  "status": "completed",
  "result": {
    "backtest_id": "...",
    "request": { ... },
    "performance": {
      "total_return": 0.156,
      "annualized_return": 0.624,
      "win_rate": 0.65,
      "avg_profit": 2345.67,
      "avg_loss": -1234.56,
      "profit_loss_ratio": 1.9,
      "max_drawdown": -0.089,
      "final_capital": 115600
    },
    "trades": [ ... ],
    "equity_curve": [ ... ]
  }
}
```

### 回测历史

```
GET /api/backtest/history?page=1&page_size=10
```

## 数据模型

### BacktestRequest

```rust
pub struct BacktestRequest {
    pub strategy_type: StrategyType,
    pub strategy_params: StrategyParams,
    pub backtest_period: BacktestPeriod,
    pub initial_capital: f64,
    pub commission_rate: f64,
}
```

### PerformanceMetrics

```rust
pub struct PerformanceMetrics {
    // 收益指标
    pub total_return: f64,
    pub annualized_return: f64,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub profit_loss_ratio: f64,

    // 交易效率
    pub avg_holding_days: f64,
    pub trade_count: usize,
    pub turnover_rate: f64,

    // 风险指标
    pub max_drawdown: f64,
    pub volatility: f64,

    // 资金
    pub final_capital: f64,
    pub total_profit: f64,
    pub total_loss: f64,
}
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test models
cargo test portfolio
cargo test performance
cargo test strategies
cargo test api

# 查看测试覆盖率
cargo test -- --nocapture
```

## 性能指标

| 指标 | 目标值 |
|------|--------|
| 回测速度 (3个月) | < 60秒 |
| API响应时间 | < 100ms |
| 并发回测 | 支持10个 |
| 内存占用 | < 500MB |

## 项目结构

```
services/backtest-service/
├── Cargo.toml              # 项目配置
├── README.md               # 本文档
├── Makefile                # 构建脚本
├── Dockerfile              # Docker 镜像
├── docker-compose.yml      # Docker Compose
├── config/                 # 配置文件
│   └── development.toml
├── migrations/             # 数据库迁移
│   ├── 001_create_stock_auction_data.sql
│   └── 002_create_stock_daily_data.sql
├── docs/                   # 文档
│   ├── ARCHITECTURE.md      # 架构文档
│   ├── DEPLOYMENT.md        # 部署文档
│   ├── API.md               # API 文档
│   └── COMPLETION_REPORT.md # 完成报告
└── src/
    ├── main.rs              # Web服务入口
    ├── lib.rs               # 模块导出
    ├── models.rs            # 数据模型 + 验证
    ├── portfolio.rs         # 资金管理
    ├── performance.rs       # 绩效计算
    ├── strategies.rs        # 策略引擎
    ├── data_source.rs       # ClickHouse数据源
    ├── engine.rs            # 回测引擎核心
    ├── api.rs               # API接口 + 任务管理
    ├── cli.rs               # CLI 工具
    ├── metrics.rs           # 指标收集
    ├── config.rs            # 配置管理
    ├── config_watcher.rs    # 配置热重载
    └── migrations.rs        # 数据库迁移
```

## 开发路线图

### ✅ Week 1: 核心引擎开发 (已完成)
- [x] 项目结构和依赖
- [x] 数据模型定义
- [x] 请求验证和错误处理
- [x] 资金管理器实现
- [x] 绩效计算器实现
- [x] 策略信号生成器实现
- [x] 回测引擎核心实现
- [x] 单元测试 (16个测试全部通过)

### ✅ Week 2: API 和数据集成 (已完成)
- [x] HTTP API 路由和处理器
- [x] 任务管理器 (异步任务处理)
- [x] 回测结果内存存储
- [x] API 与引擎集成
- [x] 策略列表 API
- [x] API 测试 (18个测试全部通过)

### ✅ Week 2+: 增强功能 (已完成)
- [x] CLI 工具支持
- [x] Prometheus 监控指标
- [x] 配置热重载
- [x] 数据库迁移工具
- [x] 完整架构文档
- [x] 详细部署文档
- [x] 26 个单元测试全部通过

### 🔄 Week 3: 前端开发 (待实现)
- [ ] 回测配置页面
- [ ] 回测报告页面
- [ ] 回测历史页面
- [ ] 前后端联调

### 📋 Week 4: 优化和测试 (待实现)
- [ ] 性能优化
- [ ] 端到端测试
- [ ] 文档完善
- [ ] 部署验证

## 故障排查

### 服务无法启动

```bash
# 检查端口占用
lsof -i :8086

# 停止旧服务
pkill -f backtest-service
```

### ClickHouse 连接失败

```bash
# 检查 ClickHouse 服务状态
docker ps | grep clickhouse

# 查看 ClickHouse 日志
docker logs duanxianxia-clickhouse-1

# 测试连接
curl http://localhost:8123/ping
```

### 回测任务失败

```bash
# 查看任务详情
curl http://localhost:8086/api/backtest/{backtest_id}

# 检查 ClickHouse 数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
  SELECT count() FROM duanxianxia.auction_data
  WHERE timestamp >= toUnixTimestamp('2025-10-01 00:00:00')
"
```

## 许可证

MIT License

## 联系方式

- 项目: 短线侠
- 作者: AI Assistant (Claude Code)
- 更新时间: 2026-01-09
