# 短线侠平台 - 使用文档

## 📋 概述

短线侠是一个专业的 A 股短线交易平台，提供实时行情、技术指标、选股器、策略回测等功能。

**技术架构**: 六边形架构（Hexagonal Architecture）
**服务数量**: 11 个微服务
**API 风格**: RESTful + WebSocket

---

## 🚀 快速开始

### 1. 启动服务

确保所有服务已启动（参考《部署文档》）：

```bash
# 检查服务状态
curl http://localhost:8089/health  # query-service
curl http://localhost:8088/health  # limit-review-service
```

### 2. 访问 API

所有服务的 HTTP API 都可以通过 `curl` 或任何 HTTP 客户端访问。

---

## 📡 API 文档

### Query Service（查询服务）

**端口**: 8089

#### 1. 龙头高度查询

获取龙头股票排名：

```bash
curl "http://localhost:8089/api/screener/leaders?date=2025-01-16&limit=10"
```

**响应示例**：

```json
{
  "code": 0,
  "message": "success",
  "data": [
    {
      "code": "000001",
      "name": "平安银行",
      "leader_height": 95.6,
      "limit_times": 5,
      "change_percent": 10.01
    }
  ]
}
```

#### 2. 连续涨停查询

查询连续涨停股票：

```bash
curl "http://localhost:8089/api/screener/consecutive-boards?date=2025-01-16&min_days=3&limit=20"
```

#### 3. 涨停股票查询

查询当日涨停股票：

```bash
curl "http://localhost:8089/api/screener/limit-up?date=2025-01-16&limit=50"
```

#### 4. 跌停股票查询

查询当日跌停股票：

```bash
curl "http://localhost:8089/api/screener/limit-down?date=2025-01-16&limit=50"
```

#### 5. 技术指标查询

查询单只股票的技术指标：

```bash
curl "http://localhost:8089/api/indicators/000001"
```

**响应示例**：

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "code": "000001",
    "name": "平安银行",
    "date": "2025-01-16",
    "ma5": 12.50,
    "ma10": 12.30,
    "ma20": 12.10,
    "ma60": 11.80,
    "dif": 0.15,
    "dea": 0.12,
    "macd": 0.06,
    "kdj_k": 65.2,
    "kdj_d": 60.8,
    "kdj_j": 74.0,
    "rsi6": 58.3,
    "rsi12": 54.6,
    "rsi24": 52.1
  }
}
```

#### 6. 板块查询

查询所有板块：

```bash
curl "http://localhost:8089/api/sectors"
```

查询板块内股票：

```bash
curl "http://localhost:8089/api/sectors/banks/stocks?date=2025-01-16"
```

查询板块表现：

```bash
curl "http://localhost:8089/api/sectors/performance?date=2025-01-16&limit=10"
```

---

### Limit Review Service（涨停复盘服务）

**端口**: 8088

#### 1. 每日涨停复盘

获取指定日期的涨停复盘数据：

```bash
curl "http://localhost:8088/api/review/2025-01-16"
```

**响应示例**：

```json
{
  "date": "2025-01-16",
  "total_count": 45,
  "limit_up_count": 38,
  "limit_down_count": 7,
  "stocks": [
    {
      "code": "000001",
      "name": "平安银行",
      "limit_type": "StraightBoard",
      "limit_times": 5,
      "limit_price": 12.65,
      "final_price": 12.65,
      "seal_amount": 125000000,
      "open_times": 0
    }
  ]
}
```

#### 2. 龙头榜查询

查询龙头榜：

```bash
curl "http://localhost:8088/api/review/leader-board?date=2025-01-16&limit=10"
```

#### 3. 龙头详情

查询龙头股票详细信息：

```bash
curl "http://localhost:8088/api/review/leader-detail?code=000001&date=2025-01-16"
```

---

### Realtime Service（实时行情服务）

**端口**: 8090

#### WebSocket 连接

连接 WebSocket 接收实时行情：

```javascript
// JavaScript 示例
const ws = new WebSocket('ws://localhost:8090/ws/quotes');

ws.onopen = () => {
  console.log('WebSocket 已连接');

  // 订阅股票
  ws.send(JSON.stringify({
    action: 'subscribe',
    codes: ['000001', '000002', '600000']
  }));
};

ws.onmessage = (event) => {
  const quote = JSON.parse(event.data);
  console.log('实时行情:', quote);

  // 数据格式：
  // {
  //   "code": "000001",
  //   "name": "平安银行",
  //   "price": 12.65,
  //   "volume": 1234567,
  //   "amount": 15678900.00,
  //   "change_percent": 2.35,
  //   "timestamp": "2025-01-16T09:30:00+08:00"
  // }
};

ws.onerror = (error) => {
  console.error('WebSocket 错误:', error);
};

ws.onclose = () => {
  console.log('WebSocket 已关闭');
};
```

---

### Auction Realtime Service（集合竞价实时推送）

**端口**: 8081

#### WebSocket 连接

连接 WebSocket 接收集合竞价数据：

```javascript
const ws = new WebSocket('ws://localhost:8081/ws/auction');

ws.onopen = () => {
  console.log('集合竞价 WebSocket 已连接');

  // 订阅股票
  ws.send(JSON.stringify({
    action: 'subscribe',
    codes: ['000001', '000002']
  }));
};

ws.onmessage = (event) => {
  const auction = JSON.parse(event.data);
  console.log('集合竞价数据:', auction);

  // 数据格式：
  // {
  //   "code": "000001",
  //   "name": "平安银行",
  //   "auction_type": "morning",
  //   "price": 12.60,
  //   "volume": 123456,
  //   "amount": 1567890.00,
  //   "timestamp": "2025-01-16T09:24:30+08:00"
  // }
};
```

---

### Auth Service（认证服务）

**端口**: 8084

#### 1. 用户注册

```bash
curl -X POST "http://localhost:8084/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user",
    "email": "user@example.com",
    "password": "secure_password123"
  }'
```

#### 2. 用户登录

```bash
curl -X POST "http://localhost:8084/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password123"
  }'
```

**响应示例**：

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": 1,
      "username": "test_user",
      "email": "user@example.com"
    }
  }
}
```

#### 3. 验证 Token

```bash
curl "http://localhost:8084/api/auth/verify" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

### Backtest Service（回测服务）

**端口**: 8085

#### 1. 创建回测任务

```bash
curl -X POST "http://localhost:8085/api/backtest/create" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "strategy_name": "涨停板策略",
    "start_date": "2024-01-01",
    "end_date": "2024-12-31",
    "initial_capital": 100000.00,
    "params": {
      "min_limit_times": 3,
      "max_open_times": 2
    }
  }'
```

#### 2. 查询回测结果

```bash
curl "http://localhost:8085/api/backtest/results/123" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**响应示例**：

```json
{
  "id": 123,
  "user_id": 1,
  "strategy_name": "涨停板策略",
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "initial_capital": 100000.00,
  "final_capital": 158360.00,
  "return_rate": 58.36,
  "max_drawdown": -12.5,
  "sharpe_ratio": 1.85,
  "trades_count": 156
}
```

---

## 🔧 使用示例

### Python 示例

```python
import requests
import json

# 基础配置
BASE_URL = "http://localhost:8089"

# 查询龙头股票
def get_leaders(date="2025-01-16", limit=10):
    url = f"{BASE_URL}/api/screener/leaders"
    params = {"date": date, "limit": limit}
    response = requests.get(url, params=params)
    return response.json()

# 查询技术指标
def get_indicators(code):
    url = f"{BASE_URL}/api/indicators/{code}"
    response = requests.get(url)
    return response.json()

# 使用示例
if __name__ == "__main__":
    # 获取龙头股票
    leaders = get_leaders()
    print("龙头股票:", json.dumps(leaders, indent=2, ensure_ascii=False))

    # 获取技术指标
    indicators = get_indicators("000001")
    print("技术指标:", json.dumps(indicators, indent=2, ensure_ascii=False))
```

### JavaScript 示例

```javascript
// 使用 fetch API
const BASE_URL = 'http://localhost:8089';

// 查询龙头股票
async function getLeaders(date = '2025-01-16', limit = 10) {
  const url = new URL(`${BASE_URL}/api/screener/leaders`);
  url.searchParams.append('date', date);
  url.searchParams.append('limit', limit);

  const response = await fetch(url);
  return await response.json();
}

// 查询技术指标
async function getIndicators(code) {
  const url = `${BASE_URL}/api/indicators/${code}`;
  const response = await fetch(url);
  return await response.json();
}

// 使用示例
(async () => {
  // 获取龙头股票
  const leaders = await getLeaders();
  console.log('龙头股票:', JSON.stringify(leaders, null, 2));

  // 获取技术指标
  const indicators = await getIndicators('000001');
  console.log('技术指标:', JSON.stringify(indicators, null, 2));
})();
```

---

## 🔍 错误处理

所有 API 遵循统一的错误响应格式：

```json
{
  "code": -1,
  "message": "错误描述",
  "details": "详细错误信息"
}
```

### 常见错误码

| 错误码 | 含义 | 解决方案 |
|--------|------|----------|
| 0 | 成功 | - |
| -1 | 服务器内部错误 | 检查服务日志 |
| 1001 | 参数错误 | 检查请求参数 |
| 1002 | 数据库错误 | 检查数据库连接 |
| 1003 | 未找到数据 | 确认数据存在 |
| 2001 | 未授权 | 检查 Token |
| 2002 | Token 过期 | 重新登录 |

### 错误处理示例

```python
import requests

try:
    response = requests.get("http://localhost:8089/api/screener/leaders")
    result = response.json()

    if result['code'] != 0:
        print(f"API 错误: {result['message']}")
    else:
        print(f"成功: {result['data']}")
except requests.exceptions.RequestException as e:
    print(f"网络错误: {e}")
```

---

## 📊 数据格式

### 日期格式

所有日期使用 ISO 8601 格式：`YYYY-MM-DD`

示例：`2025-01-16`

### 时间戳格式

时间戳使用 ISO 8601 格式，包含时区信息：`YYYY-MM-DDTHH:MM:SS+08:00`

示例：`2025-01-16T09:30:00+08:00`

### 分页格式

支持分页的接口使用以下参数：

- `page`: 页码（从 1 开始）
- `page_size`: 每页数量（默认 20，最大 100）

示例：

```bash
curl "http://localhost:8089/api/sectors/stocks?page=1&page_size=50"
```

---

## 🔗 相关资源

- **部署文档**: [DEPLOYMENT.md](./DEPLOYMENT.md)
- **架构文档**: [HEXAGONAL_ARCHITECTURE_FINAL_REPORT.md](./HEXAGONAL_ARCHITECTURE_FINAL_REPORT.md)
- **GitHub**: https://github.com/your-org/duanxianxia

---

## 📞 支持

如有问题，请联系：
- 邮件: support@duanxianxia.com
- GitHub Issues: https://github.com/your-org/duanxianxia/issues

---

**文档版本**: v1.0
**更新日期**: 2025-01-16
