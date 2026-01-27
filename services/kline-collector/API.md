# K线收集器 API 文档

## 📋 目录

1. [概述](#概述)
2. [健康检查API](#健康检查api)
3. [回填管理API](#回填管理api)
4. [状态查询API](#状态查询api)
5. [指标API](#指标api)
6. [错误码](#错误码)
7. [使用示例](#使用示例)

---

## 概述

K线收集器提供 RESTful API 用于管理和监控服务。

**基础URL:** `http://localhost:8080`

**认证:** 当前版本不需要认证,生产环境建议配置。

**响应格式:** JSON

---

## 健康检查API

### GET /health

检查服务及其组件的健康状态。

**请求示例:**
```bash
curl http://localhost:8080/health
```

**响应示例:**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "components": [
    {
      "name": "redis",
      "status": "healthy",
      "message": null,
      "latency_ms": 5
    },
    {
      "name": "clickhouse",
      "status": "healthy",
      "message": null,
      "latency_ms": 10
    },
    {
      "name": "rustdx",
      "status": "degraded",
      "message": "slow response",
      "latency_ms": 200
    }
  ]
}
```

**字段说明:**

| 字段 | 类型 | 说明 |
|------|------|------|
| `status` | string | 整体健康状态: `healthy`, `degraded`, `unhealthy` |
| `uptime_seconds` | number | 服务运行时间(秒) |
| `components` | array | 各组件的健康状态 |
| `components[].name` | string | 组件名称 |
| `components[].status` | string | 组件状态 |
| `components[].message` | string\|null | 错误或警告消息 |
| `components[].latency_ms` | number\|null | 延迟(毫秒) |

**状态判断逻辑:**
- `healthy`: 所有组件都健康
- `degraded`: 至少一个组件降级,但没有组件不健康
- `unhealthy`: 至少一个组件不健康

---

## 回填管理API

### POST /api/backfill

手动触发历史数据回填。

**请求体:**
```json
{
  "days": 7,
  "periods": ["1m", "5m", "1d"]
}
```

**参数说明:**

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `days` | number | 否 | 7 | 回填天数(1-30) |
| `periods` | string[] | 否 | ["1m","5m"] | K线周期 |

**支持的周期:**
- `1m`: 1分钟
- `5m`: 5分钟
- `15m`: 15分钟
- `30m`: 30分钟
- `60m`: 1小时
- `1d`: 日线

**请求示例:**
```bash
curl -X POST http://localhost:8080/api/backfill \
  -H "Content-Type: application/json" \
  -d '{
    "days": 7,
    "periods": ["1m", "5m", "1d"]
  }'
```

**成功响应:**
```json
{
  "success": true,
  "message": "回填完成",
  "total_klines": 15000,
  "errors": null
}
```

**部分成功响应:**
```json
{
  "success": true,
  "message": "回填完成(部分失败)",
  "total_klines": 12000,
  "errors": [
    "周期 1m 回填失败: 连接超时",
    "周期 5m 回填失败: 数据源不可用"
  ]
}
```

**失败响应:**
```json
{
  "success": false,
  "message": "回填失败: Invalid period configuration",
  "total_klines": null,
  "errors": null
}
```

**字段说明:**

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | boolean | 是否成功 |
| `message` | string | 结果消息 |
| `total_klines` | number\|null | 成功回填的K线数量 |
| `errors` | string[]\|null | 错误列表 |

---

## 状态查询API

### GET /api/status

查询服务当前状态。

**请求示例:**
```bash
curl http://localhost:8080/api/status
```

**响应示例:**
```json
{
  "active_windows": 150,
  "is_healthy": true
}
```

**字段说明:**

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_windows` | number | 当前活动的聚合窗口数量 |
| `is_healthy` | boolean | 服务是否健康 |

**使用场景:**
- 监控聚合引擎活动状态
- 简单的健康检查(不需要详细组件信息)

---

## 指标API

### GET /metrics

Prometheus 格式的指标数据。

**请求示例:**
```bash
curl http://localhost:8080/metrics
```

**响应示例:**
```
# HELP kline_collector_uptime_seconds 服务运行时间
# TYPE kline_collector_uptime_seconds gauge
kline_collector_uptime_seconds 3600

# HELP kline_collector_quotes_received_total 接收的行情总数
# TYPE kline_collector_quotes_received_total counter
kline_collector_quotes_received_total{period="1m"} 15000
kline_collector_quotes_received_total{period="5m"} 3000
kline_collector_quotes_received_total{period="1d"} 150

# HELP kline_collector_klines_written 写入的K线总数
# TYPE kline_collector_klines_written counter
kline_collector_klines_written{period="1m"} 15000
kline_collector_klines_written{period="5m"} 3000

# HELP kline_collector_active_windows 当前活动窗口数
# TYPE kline_collector_active_windows gauge
kline_collector_active_windows 150

# HELP kline_collector_buffer_size 缓冲区大小
# TYPE kline_collector_buffer_size gauge
kline_collector_buffer_size 100

# HELP kline_collector_redis_latency_seconds Redis延迟
# TYPE kline_collector_redis_latency_seconds gauge
kline_collector_redis_latency_seconds 0.005

# HELP kline_collector_clickhouse_latency_seconds ClickHouse延迟
# TYPE kline_collector_clickhouse_latency_seconds gauge
kline_collector_clickhouse_latency_seconds 0.010
```

**主要指标:**

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `kline_collector_uptime_seconds` | gauge | 服务运行时间 |
| `kline_collector_quotes_received_total` | counter | 接收行情总数(按周期) |
| `kline_collector_klines_written` | counter | 写入K线总数 |
| `kline_collector_active_windows` | gauge | 活动聚合窗口数 |
| `kline_collector_buffer_size` | gauge | 缓冲区大小 |
| `kline_collector_redis_latency_seconds` | gauge | Redis延迟 |
| `kline_collector_clickhouse_latency_seconds` | gauge | ClickHouse延迟 |

**Grafana 仪表板配置:**

```promql
# 接收速率
rate(kline_collector_quotes_received_total[5m])

# 写入速率
rate(kline_collector_klines_written[5m])

# 延迟
kline_collector_redis_latency_seconds
kline_collector_clickhouse_latency_seconds

# 活动窗口趋势
kline_collector_active_windows
```

---

## 错误码

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 成功 |
| 400 | 请求参数错误 |
| 500 | 服务器内部错误 |

### 错误响应格式

```json
{
  "success": false,
  "message": "错误描述",
  "total_klines": null,
  "errors": null
}
```

### 常见错误

| 错误消息 | 原因 | 解决方案 |
|---------|------|----------|
| `Invalid period configuration` | 周期参数无效 | 检查 periods 是否包含支持的值 |
| `回填失败: Connection refused` | 数据库连接失败 | 检查 Redis/ClickHouse 服务状态 |
| `回填失败: rustdx not available` | rustdx 不可用 | 检查是否在交易时间或通达信是否运行 |
| `kline_count exceeds limit` | 超过限制 | 减少天数或周期数量 |

---

## 使用示例

### Python 客户端

```python
import requests
import json

class KlineCollectorClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url

    def health_check(self):
        """健康检查"""
        response = requests.get(f"{self.base_url}/health")
        return response.json()

    def get_status(self):
        """获取状态"""
        response = requests.get(f"{self.base_url}/api/status")
        return response.json()

    def trigger_backfill(self, days=7, periods=None):
        """触发回填"""
        if periods is None:
            periods = ["1m", "5m", "1d"]

        payload = {
            "days": days,
            "periods": periods
        }
        response = requests.post(
            f"{self.base_url}/api/backfill",
            json=payload
        )
        return response.json()

    def get_metrics(self):
        """获取 Prometheus 指标"""
        response = requests.get(f"{self.base_url}/metrics")
        return response.text

# 使用示例
client = KlineCollectorClient()

# 健康检查
health = client.health_check()
print(f"服务状态: {health['status']}")
print(f"运行时间: {health['uptime_seconds']} 秒")

# 触发回填
result = client.trigger_backfill(days=7, periods=["1d"])
if result['success']:
    print(f"回填成功: {result['total_klines']} 条K线")
else:
    print(f"回填失败: {result['message']}")
```

### cURL 示例

```bash
# 1. 健康检查
curl -X GET http://localhost:8080/health | jq '.'

# 2. 获取状态
curl -X GET http://localhost:8080/api/status | jq '.'

# 3. 触发7天回填
curl -X POST http://localhost:8080/api/backfill \
  -H "Content-Type: application/json" \
  -d '{"days": 7, "periods": ["1d"]}' | jq '.'

# 4. 获取指标
curl -X GET http://localhost:8080/metrics | head -20

# 5. 只查看健康状态
curl -s http://localhost:8080/health | jq '.status'

# 6. 查看各组件延迟
curl -s http://localhost:8080/health | jq '.components[] | {name: .name, latency_ms: .latency_ms}'
```

### JavaScript/Node.js 客户端

```javascript
const axios = require('axios');

class KlineCollectorClient {
    constructor(baseUrl = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
    }

    async healthCheck() {
        const response = await axios.get(`${this.baseUrl}/health`);
        return response.data;
    }

    async getStatus() {
        const response = await axios.get(`${this.baseUrl}/api/status`);
        return response.data;
    }

    async triggerBackfill(days = 7, periods = null) {
        if (!periods) {
            periods = ['1m', '5m', '1d'];
        }

        const response = await axios.post(
            `${this.baseUrl}/api/backfill`,
            { days, periods }
        );
        return response.data;
    }

    async getMetrics() {
        const response = await axios.get(`${this.baseUrl}/metrics`);
        return response.data;
    }
}

// 使用示例
(async () => {
    const client = new KlineCollectorClient();

    // 健康检查
    const health = await client.healthCheck();
    console.log(`服务状态: ${health.status}`);

    // 触发回填
    const result = await client.triggerBackfill(7, ['1d']);
    console.log(`回填${result.success ? '成功' : '失败'}`);
    if (result.success) {
        console.log(`K线数量: ${result.total_klines}`);
    }
})();
```

---

## 集成示例

### Kubernetes 集成

```yaml
apiVersion: v1
kind: Service
metadata:
  name: kline-collector
spec:
  selector:
    app: kline-collector
  ports:
  - port: 8080
    targetPort: 8080
---
apiVersion: v1
kind: Pod
metadata:
  name: kline-collector
  labels:
    app: kline-collector
spec:
  containers:
  - name: kline-collector
    image: kline-collector:latest
    ports:
    - containerPort: 8080
    env:
    - name: REDIS_URL
      value: "redis://redis-service:6379"
    - name: CLICKHOUSE_URL
      value: "http://clickhouse-service:8124"
    livenessProbe:
      httpGet:
        path: /health
        port: 8080
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
```

### Prometheus 监控配置

```yaml
scrape_configs:
  - job_name: 'kline-collector'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: /metrics
    scrape_interval: 15s
```

---

## 版本历史

### v1.0.0 (当前版本)
- ✅ 基础健康检查API
- ✅ 历史数据回填API
- ✅ 状态查询API
- ✅ Prometheus指标API

### 未来计划
- 📝 实时行情推送API (WebSocket)
- 📝 数据查询API
- 📝 配置管理API
- 📝 任务管理API

---

## 联系方式

- 技术支持: GitHub Issues
- 文档: `/docs`
- 示例代码: `/examples`
