# 数据回测与策略模块 - 设计文档

**创建日期**: 2026-01-09
**状态**: ✅ 设计完成
**预计工期**: 4 周

---

## 执行摘要

本文档定义了**短线侠**平台的数据回测与策略模块的完整设计方案。该模块支持竞价选股策略和盘中短线策略的回测，提供参数化策略模板、完整绩效评估和可视化报告。

### 核心特性
- ✅ **轻量级设计** - 基于 ClickHouse 直接回测，无需额外数据存储
- ✅ **3种策略模板** - 竞价龙头、竞价封单、盘中突破
- ✅ **完整评估** - 收益指标、交易效率、风险控制
- ✅ **用户友好** - 参数化配置，Web界面操作

---

## 1. 整体架构

### 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    前端回测页面                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ 策略选择      │  │ 参数配置      │  │ 回测报告      │      │
│  │ (竞价龙头)   │  │ (日期/资金)   │  │ (收益曲线)    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              backtest-service (Rust)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  HTTP API (Port 8086)                                 │  │
│  │  - POST /api/backtest/run    - 启动回测              │  │
│  │  - GET  /api/backtest/{id}   - 查询结果              │  │
│  │  - GET  /api/backtest/strategies - 策略列表          │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  策略引擎层                                            │  │
│  │  - AuctionStrategy: 竞价选股策略                      │  │
│  │  - IntradayStrategy: 盘中短线策略                    │  │
│  │  - SignalGenerator: 信号生成器                       │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  回测核心引擎                                          │  │
│  │  - BacktestEngine: 事件驱动回测                      │  │
│  │  - PortfolioManager: 资金管理                        │  │
│  │  - PerformanceCalculator: 绩效计算                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   ClickHouse 数据库                          │
│  - auction_data: 竞价数据                                   │
│  - stock_realtime_quotes: 实时行情                         │
│  - kline_5m: 5分钟K线                                      │
└─────────────────────────────────────────────────────────────┘
```

### 数据流

```
用户提交回测请求 (策略+参数)
  ↓
backtest-service 加载历史数据 (ClickHouse)
  ↓
逐条数据模拟交易 (事件驱动)
  ↓
记录每笔交易 (买入/卖出)
  ↓
计算绩效指标 (收益率/胜率/持仓时间)
  ↓
返回回测报告 (JSON) → 前端可视化
```

---

## 2. 数据模型

### 回测请求
```rust
pub struct BacktestRequest {
    pub strategy_type: StrategyType,
    pub strategy_params: StrategyParams,
    pub backtest_period: BacktestPeriod,
    pub initial_capital: f64,  // 初始资金
    pub commission_rate: f64,  // 手续费率 (默认 0.0003)
}

pub enum StrategyType {
    AuctionLeader,      // 竞价龙头策略
    AuctionSeal,        // 竞价封单策略
    IntradayBreakout,   // 盘中突破策略
}

pub struct StrategyParams {
    // 竞价策略参数
    pub min_strength_score: Option<i32>,      // 最低强度评分
    pub min_buy_seal_amount: Option<f64>,     // 最低买封金额(万)
    pub max_change_percent: Option<f64>,      // 最大涨幅(%)

    // 盘中策略参数
    pub volume_multiplier: Option<f64>,       // 成交量放大倍数
    pub breakout_threshold: Option<f64>,      // 突破阈值(%)

    // 通用参数
    pub holding_days: Option<i32>,            // 持仓天数
    pub stop_loss: Option<f64>,               // 止损(%)
    pub take_profit: Option<f64>,             // 止盈(%)
}

pub struct BacktestPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
```

### 回测结果 
```rust
pub struct BacktestResult {
    pub backtest_id: String,
    pub request: BacktestRequest,
    pub performance: PerformanceMetrics,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
    pub created_at: DateTime<Utc>,
}

pub struct PerformanceMetrics {
    // 收益指标
    pub total_return: f64,          // 总收益率
    pub annualized_return: f64,     // 年化收益率
    pub win_rate: f64,              // 胜率
    pub avg_profit: f64,            // 平均盈利
    pub avg_loss: f64,              // 平均亏损
    pub profit_loss_ratio: f64,     // 盈亏比

    // 交易效率
    pub avg_holding_days: f64,      // 平均持仓天数
    pub trade_count: usize,         // 交易次数
    pub turnover_rate: f64,         // 换手率

    // 风险指标
    pub max_drawdown: f64,          // 最大回撤
    pub volatility: f64,            // 波动率

    // 资金
    pub final_capital: f64,         // 最终资金
    pub total_profit: f64,          // 总盈利
    pub total_loss: f64,            // 总亏损
}

pub struct Trade {
    pub code: String,               // 股票代码
    pub name: String,               // 股票名称
    pub buy_date: NaiveDate,        // 买入日期
    pub sell_date: NaiveDate,       // 卖出日期
    pub buy_price: f64,             // 买入价格
    pub sell_price: f64,            // 卖出价格
    pub quantity: i64,              // 数量
    pub profit: f64,                // 盈亏
    pub profit_percent: f64,        // 盈亏比例
    pub holding_days: i32,          // 持仓天数
    pub exit_reason: String,        // 卖出原因
}

pub struct EquityPoint {
    pub date: NaiveDate,
    pub equity: f64,                // 净值
    pub drawdown: f64,              // 回撤
}
```

---

## 3. HTTP API

### 3.1 启动回测
```
POST /api/backtest/run

Request:
{
  "strategy_type": "auction_leader",
  "strategy_params": {
    "min_strength_score": 80,
    "min_buy_seal_amount": 1000,
    "holding_days": 1
  },
  "backtest_period": {
    "start_date": "2025-10-01",
    "end_date": "2025-12-31"
  },
  "initial_capital": 100000,
  "commission_rate": 0.0003
}

Response (202):
{
  "backtest_id": "bt_20250109_abc123",
  "status": "running",
  "estimated_time": 30  // 秒
}
```

### 3.2 查询回测结果
```
GET /api/backtest/{backtest_id}

Response (200):
{
  "backtest_id": "bt_20250109_abc123",
  "status": "completed",
  "performance": {
    "total_return": 0.156,
    "annualized_return": 0.624,
    "win_rate": 0.65,
    "avg_profit": 2345.67,
    "avg_loss": -1234.56,
    "profit_loss_ratio": 1.9,
    "avg_holding_days": 1.2,
    "trade_count": 20,
    "max_drawdown": -0.089,
    "final_capital": 115600
  },
  "trades": [...],
  "equity_curve": [...]
}
```

### 3.3 获取策略列表
```
GET /api/backtest/strategies

Response (200):
{
  "strategies": [
    {
      "id": "auction_leader",
      "name": "竞价龙头策略",
      "description": "竞价强度评分>80且买封金额>1000万",
      "params": [
        {"name": "min_strength_score", "type": "integer", "default": 80},
        {"name": "min_buy_seal_amount", "type": "float", "default": 1000},
        {"name": "holding_days", "type": "integer", "default": 1}
      ]
    }
  ]
}
```

### 3.4 回测历史
```
GET /api/backtest/history?page=1&page_size=10

Response (200):
{
  "total": 45,
  "items": [
    {
      "backtest_id": "bt_20250109_abc123",
      "strategy_name": "竞价龙头策略",
      "created_at": "2025-01-09T10:30:00",
      "total_return": 0.156,
      "status": "completed"
    }
  ]
}
```

---

## 4. 策略模板

### 4.1 竞价龙头策略
**逻辑**:
- 条件: 竞价强度评分 > 80 AND 买封金额 > 1000万 AND 涨幅 < 8%
- 买入: 竞价结束后开盘价买入
- 卖出: 次日开盘价卖出 (持仓1天)

**参数**:
- `min_strength_score`: 最低强度评分 (0-100, 默认80)
- `min_buy_seal_amount`: 最低买封金额 (100-10000万, 默认1000)
- `max_change_percent`: 最大涨幅 (1-10%, 默认8)
- `holding_days`: 持仓天数 (1-10天, 默认1)

### 4.2 竞价封单策略
**逻辑**:
- 条件: 买封金额排名前10 AND 涨幅 < 5%
- 买入: 竞价结束后开盘价买入
- 卖出: 持仓3天后开盘价卖出

**参数**:
- `top_n`: 排名前N (1-50, 默认10)
- `max_change_percent`: 最大涨幅 (1-10%, 默认5)
- `holding_days`: 持仓天数 (1-10天, 默认3)

### 4.3 盘中突破策略
**逻辑**:
- 条件: 突破前高 + 成交量放大2倍
- 买入: 突破时买入
- 卖出: 尾盘卖出 (不持仓过夜)

**参数**:
- `volume_multiplier`: 成交量放大倍数 (1.5-5, 默认2)
- `breakout_threshold`: 突破阈值 (1-5%, 默认2)

---

## 5. 回测核心逻辑

### 5.1 回测引擎
```rust
pub struct BacktestEngine {
    data_source: ClickHouseDataSource,
    portfolio: PortfolioManager,
    calculator: PerformanceCalculator,
}

impl BacktestEngine {
    pub async fn run(&mut self, request: BacktestRequest) -> BacktestResult {
        // 1. 加载历史数据
        let data = self.load_historical_data(&request).await?;

        // 2. 初始化资金
        self.portfolio.initialize(request.initial_capital);

        // 3. 逐日模拟交易
        for day_data in data {
            // 生成信号
            let signals = self.generate_signals(&day_data, &request.strategy_params);

            // 执行交易
            for signal in signals {
                self.execute_signal(signal, &day_data);
            }

            // 检查卖出条件
            self.portfolio.check_exit_signals(day_data.date, &request.strategy_params);

            // 更新净值
            self.portfolio.record_equity(day_data.date);
        }

        // 4. 计算绩效
        let performance = self.calculator.calculate(&self.portfolio);

        // 5. 返回结果
        BacktestResult { ... }
    }
}
```

### 5.2 信号生成
```rust
impl BacktestEngine {
    fn auction_leader_signals(&self, day_data: &DayData, params: &StrategyParams)
        -> Vec<Signal> {

        day_data.auction_data.iter()
            .filter(|auction| {
                auction.strength_score >= params.min_strength_score.unwrap_or(80)
                && auction.buy_seal_amount >= params.min_buy_seal_amount.unwrap_or(1000.0)
                && auction.change_percent <= params.max_change_percent.unwrap_or(8.0)
            })
            .map(|auction| Signal {
                code: auction.code.clone(),
                price: auction.open_price,
                date: day_data.date,
            })
            .collect()
    }
}
```

### 5.3 资金管理
```rust
impl PortfolioManager {
    // 等权重买入
    pub fn execute_buy(&mut self, signal: Signal, commission_rate: f64) {
        let buy_amount = self.capital / (self.positions.len() + 1) as f64;
        let quantity = (buy_amount / signal.price) as i64;
        let cost = quantity as f64 * signal.price * (1.0 + commission_rate);

        self.positions.insert(signal.code, Position {
            code: signal.code,
            buy_price: signal.price,
            quantity,
            buy_date: signal.date,
        });

        self.capital -= cost;
    }

    // 持仓到期卖出
    pub fn check_exit_signals(&mut self, current_date: NaiveDate, holding_days: i32) {
        let to_sell: Vec<_> = self.positions.iter()
            .filter(|(_, pos)| {
                (current_date - pos.buy_date).num_days() >= holding_days as i64
            })
            .map(|(code, _)| code.clone())
            .collect();

        for code in to_sell {
            self.sell_position(&code, current_date, "持仓到期");
        }
    }
}
```

### 5.4 绩效计算
```rust
impl PerformanceCalculator {
    pub fn calculate(&self, portfolio: &PortfolioManager) -> PerformanceMetrics {
        let trades = portfolio.get_closed_trades();

        // 收益指标
        let total_return = (portfolio.equity - portfolio.initial_capital)
            / portfolio.initial_capital;

        let winning_trades: Vec<_> = trades.iter().filter(|t| t.profit > 0).collect();
        let losing_trades: Vec<_> = trades.iter().filter(|t| t.profit <= 0).collect();

        let win_rate = winning_trades.len() as f64 / trades.len() as f64;
        let avg_profit = winning_trades.iter().map(|t| t.profit).sum::<f64>()
            / winning_trades.len() as f64;
        let avg_loss = losing_trades.iter().map(|t| t.profit).sum::<f64>()
            / losing_trades.len() as f64;

        // 最大回撤
        let max_drawdown = self.calculate_max_drawdown(&portfolio.equity_curve);

        PerformanceMetrics {
            total_return,
            win_rate,
            avg_profit,
            avg_loss,
            max_drawdown,
            ...
        }
    }
}
```

---

## 6. 前端设计

### 6.1 页面结构

#### 回测配置页面 (`/backtest`)
- 策略选择下拉框
- 策略参数表单
- 回测设置 (日期范围、初始资金)
- 开始回测按钮

#### 回测报告页面 (`/backtest/result/:id`)
- 收益指标卡片 (总收益率、胜率、盈亏比)
- 收益曲线图 (ECharts)
- 回撤曲线图 (ECharts)
- 交易明细表格

#### 回测历史页面 (`/backtest/history`)
- 历史回测列表
- 状态显示
- 查看详情按钮

### 6.2 核心组件

#### BacktestConfig.tsx
```tsx
export function BacktestConfig() {
  const [strategy, setStrategy] = useState(null);
  const [params, setParams] = useState({});
  const [loading, setLoading] = useState(false);

  const handleStartBacktest = async () => {
    setLoading(true);
    const response = await fetch('/api/backtest/run', {
      method: 'POST',
      body: JSON.stringify({
        strategy_type: strategy.id,
        strategy_params: params,
        backtest_period: period,
        initial_capital: 100000,
      }),
    });

    const { backtest_id } = await response.json();
    navigate(`/backtest/result/${backtest_id}`);
  };

  return (
    <Card title="策略回测">
      <StrategySelector value={strategy} onChange={setStrategy} />
      <StrategyParamsForm strategy={strategy} params={params} onChange={setParams} />
      <Button onClick={handleStartBacktest}>开始回测</Button>
    </Card>
  );
}
```

#### BacktestReport.tsx
```tsx
export function BacktestReport({ backtestId }: { backtestId: string }) {
  const [result, setResult] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const pollResult = setInterval(async () => {
      const response = await fetch(`/api/backtest/${backtestId}`);
      const data = await response.json();

      if (data.status === 'completed') {
        setResult(data);
        setLoading(false);
        clearInterval(pollResult);
      }
    }, 2000);

    return () => clearInterval(pollResult);
  }, [backtestId]);

  return (
    <div>
      <PerformanceMetricsCard metrics={result.performance} />
      <EquityCurveChart data={result.equity_curve} />
      <TradesTable trades={result.trades} />
    </div>
  );
}
```

---

## 7. 错误处理

### 7.1 数据验证
- 日期范围验证 (不超过3个月)
- 初始资金验证 (不少于10000元)
- 策略参数验证 (范围检查)

### 7.2 数据可用性检查
- 竞价数据存在性验证
- 行情数据充足性验证
- 数据完整性验证

### 7.3 错误响应
```rust
pub enum BacktestError {
    InvalidPeriod(String),
    InvalidCapital(String),
    InvalidParam(String),
    NoData(String),
    InsufficientData(String),
    InternalError(String),
}
```

---

## 8. 测试策略

### 8.1 单元测试
- 请求验证测试
- 资金管理测试
- 绩效计算测试

### 8.2 集成测试
- 完整回测流程测试
- API端点测试
- 数据库集成测试

### 8.3 性能测试
- 回测速度测试 (目标: < 60秒)
- 并发回测测试
- 内存占用测试

---

## 9. 实施计划

### Week 1: 核心引擎开发
- Day 1-2: 创建项目结构和依赖
- Day 3-4: 实现回测引擎核心逻辑
- Day 5: 实现资金管理和绩效计算
- Day 6-7: 单元测试

### Week 2: API 和数据集成
- Day 1-2: 实现 HTTP API 端点
- Day 3-4: 集成 ClickHouse 数据查询
- Day 5-6: 实现策略模板
- Day 7: 集成测试

### Week 3: 前端开发
- Day 1-2: 回测配置页面
- Day 3-4: 回测报告页面
- Day 5-6: 回测历史页面
- Day 7: 前后端联调

### Week 4: 优化和测试
- Day 1-2: 性能优化
- Day 3-4: 端到端测试
- Day 5-6: 文档编写
- Day 7: 部署和验证

---

## 10. 技术栈

### 后端
- **语言**: Rust
- **框架**: Actix-web
- **数据库**: ClickHouse (现有)
- **日期处理**: Chrono

### 前端
- **框架**: React 18
- **语言**: TypeScript
- **UI库**: Ant Design 5
- **图表**: ECharts

---

## 11. 性能目标

| 指标 | 目标值 |
|------|--------|
| 回测速度 (3个月) | < 60秒 |
| API响应时间 | < 100ms |
| 并发回测 | 支持10个 |
| 内存占用 | < 500MB |

---

## 12. 未来扩展

### Phase 2
- 更多策略模板
- 参数优化功能
- 策略对比功能

### Phase 3
- 自定义策略构建器
- 策略分享和导入
- 实盘交易接口

---

**文档状态**: ✅ 设计完成
**设计人**: AI Assistant (Claude Code)
**最后更新**: 2026-01-09
