# Backtest Service API 使用指南

本文档提供 Backtest Service API 的详细使用示例。

## 目录

- [快速开始](#快速开始)
- [API 端点](#api-端点)
- [使用示例](#使用示例)
- [错误处理](#错误处理)
- [最佳实践](#最佳实践)

## 快速开始

### 1. 启动服务

```bash
# 使用 Docker Compose
make docker-up

# 或直接运行
make run
```

### 2. 验证服务状态

```bash
curl http://localhost:8086/health
```

## API 端点

### 1. 健康检查

**端点:** `GET /health`

**描述:** 检查服务是否正常运行

**响应:**
```json
{
  "status": "ok",
  "service": "backtest-service"
}
```

### 2. 获取策略列表

**端点:** `GET /api/backtest/strategies`

**描述:** 获取所有可用的回测策略及其参数

**响应:**
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
        },
        {
          "name": "min_buy_seal_amount",
          "type": "float",
          "default": 1000,
          "description": "最低买封金额 (万)"
        },
        {
          "name": "holding_days",
          "type": "integer",
          "default": 1,
          "description": "持仓天数 (1-10)"
        }
      ]
    }
  ]
}
```

### 3. 启动回测

**端点:** `POST /api/backtest/run`

**描述:** 提交新的回测任务

**请求体:**
```json
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

**参数说明:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| strategy_type | string | 是 | 策略类型 (auction_leader/auction_seal/intraday_breakout) |
| strategy_params | object | 是 | 策略参数 |
| backtest_period | object | 是 | 回测时间范围 |
| initial_capital | float | 是 | 初始资金 (>= 10000) |
| commission_rate | float | 否 | 手续费率 (默认 0.0003) |

**响应:**
```json
{
  "backtest_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "status": "running",
  "estimated_time": 30
}
```

### 4. 查询回测结果

**端点:** `GET /api/backtest/{backtest_id}`

**描述:** 查询指定回测任务的结果

**响应 (进行中):**
```json
{
  "backtest_id": "...",
  "status": "running",
  "result": null,
  "error": null,
  "created_at": "2025-01-09T10:30:00Z"
}
```

**响应 (完成):**
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
      "avg_holding_days": 1.2,
      "trade_count": 20,
      "turnover_rate": 0.16,
      "max_drawdown": -0.089,
      "volatility": 0.12,
      "final_capital": 115600,
      "total_profit": 15600,
      "total_loss": -4100
    },
    "trades": [
      {
        "code": "000001",
        "name": "平安银行",
        "buy_date": "2025-10-01",
        "sell_date": "2025-10-02",
        "buy_price": 10.0,
        "sell_price": 10.5,
        "quantity": 10000,
        "profit": 5000,
        "profit_percent": 5.0,
        "holding_days": 1,
        "exit_reason": "持仓到期"
      }
    ],
    "equity_curve": [
      {
        "date": "2025-10-01",
        "equity": 100000,
        "drawdown": 0
      },
      {
        "date": "2025-10-02",
        "equity": 105000,
        "drawdown": 0
      }
    ]
  },
  "error": null,
  "created_at": "2025-01-09T10:30:00Z"
}
```

### 5. 查询回测历史

**端点:** `GET /api/backtest/history`

**查询参数:**
- `page`: 页码 (默认 1)
- `page_size`: 每页数量 (默认 10)

**响应:**
```json
{
  "total": 45,
  "page": 1,
  "page_size": 10,
  "items": [...]
}
```

## 使用示例

### Python 示例

```python
import requests
import time

BASE_URL = "http://localhost:8086"

# 1. 获取策略列表
response = requests.get(f"{BASE_URL}/api/backtest/strategies")
strategies = response.json()
print(f"可用策略: {len(strategies['strategies'])} 个")

# 2. 启动回测
backtest_request = {
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

response = requests.post(f"{BASE_URL}/api/backtest/run", json=backtest_request)
result = response.json()
backtest_id = result['backtest_id']
print(f"回测ID: {backtest_id}")

# 3. 等待回测完成
while True:
    response = requests.get(f"{BASE_URL}/api/backtest/{backtest_id}")
    result = response.json()
    status = result['status']

    if status == 'completed':
        print("回测完成!")
        break
    elif status == 'failed':
        print(f"回测失败: {result['error']}")
        break

    print(f"回测中... ({status})")
    time.sleep(3)

# 4. 获取结果
if result['result']:
    performance = result['result']['performance']
    print(f"总收益率: {performance['total_return']:.2%}")
    print(f"胜率: {performance['win_rate']:.2%}")
    print(f"最终资金: {performance['final_capital']:.2f}")
```

### JavaScript 示例

```javascript
const BASE_URL = 'http://localhost:8086';

// 启动回测
async function runBacktest() {
  const response = await fetch(`${BASE_URL}/api/backtest/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      strategy_type: 'auction_leader',
      strategy_params: {
        min_strength_score: 80,
        min_buy_seal_amount: 1000,
        holding_days: 1
      },
      backtest_period: {
        start_date: '2025-10-01',
        end_date: '2025-10-31'
      },
      initial_capital: 100000
    })
  });

  const { backtest_id } = await response.json();
  console.log('Backtest ID:', backtest_id);

  // 轮询结果
  while (true) {
    const result = await fetch(`${BASE_URL}/api/backtest/${backtest_id}`)
      .then(r => r.json());

    if (result.status === 'completed') {
      return result.result;
    }

    await new Promise(resolve => setTimeout(resolve, 3000));
  }
}

// 使用
runBacktest().then(result => {
  console.log('Performance:', result.performance);
});
```

### cURL 示例

```bash
# 启动回测
curl -X POST http://localhost:8086/api/backtest/run \
  -H "Content-Type: application/json" \
  -d '{
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
    "initial_capital": 100000
  }'

# 查询结果
curl http://localhost:8086/api/backtest/{backtest_id}

# 获取策略列表
curl http://localhost:8086/api/backtest/strategies
```

## 错误处理

### 错误响应格式

```json
{
  "error": "错误描述信息"
}
```

### 常见错误

| HTTP 状态码 | 错误类型 | 说明 |
|------------|---------|------|
| 400 | Bad Request | 请求参数无效 |
| 404 | Not Found | 回测任务不存在 |
| 500 | Internal Server Error | 服务器内部错误 |

### 错误示例

```json
{
  "error": "Invalid period: 开始日期不能晚于结束日期"
}
```

## 最佳实践

### 1. 轮询回测结果

建议使用指数退避策略:
```python
import time

def poll_backtest(backtest_id):
    delay = 1
    max_delay = 10

    while True:
        result = get_backtest(backtest_id)

        if result['status'] in ['completed', 'failed']:
            return result

        time.sleep(delay)
        delay = min(delay * 2, max_delay)
```

### 2. 处理大规模回测

- 使用分页查询历史记录
- 定期清理过期数据
- 避免同时提交过多回测任务

### 3. 参数优化建议

从保守参数开始:
```json
{
  "min_strength_score": 85,    // 提高阈值
  "min_buy_seal_amount": 2000, // 增加资金要求
  "holding_days": 1            // 短期持仓
}
```

### 4. 结果解读

关注以下指标:
- **总收益率**: 整体盈利能力
- **胜率**: 交易成功比例
- **盈亏比**: 平均盈利/平均亏损
- **最大回撤**: 风险控制能力
- **交易次数**: 策略活跃度

## 性能指标

| 指标 | 目标值 |
|------|--------|
| 回测速度 (3个月) | < 60秒 |
| API响应时间 | < 100ms |
| 并发回测 | 支持10个 |
| 内存占用 | < 500MB |

## 故障排查

### 服务无法启动

```bash
# 检查端口占用
lsof -i :8086

# 查看日志
make docker-logs
```

### ClickHouse 连接失败

```bash
# 测试 ClickHouse 连接
curl http://localhost:8123/ping

# 检查数据
docker exec -it duanxianxia-clickhouse-1 clickhouse-client \
  --query "SELECT count() FROM duanxianxia.auction_data"
```

### 回测任务失败

1. 检查请求参数是否有效
2. 验证时间范围内是否有数据
3. 查看错误信息获取详细原因

## 相关文档

- [README.md](../README.md) - 项目概览
- [DESIGN.md](../../docs/plans/2026-01-09-backtest-strategy-design.md) - 设计文档
- [WEEK1_PLAN.md](../../docs/plans/2026-01-09-backtest-week1-implementation.md) - 实施计划
