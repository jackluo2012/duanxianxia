# 涨停复盘增强功能API文档

## 概述

本文档描述了涨停复盘服务增强功能的API接口，包括区间统计、题材分析、历史回溯等功能。

**基础URL**: `http://localhost:8080`

**Content-Type**: `application/json`

---

## 1. 复盘数据API

### 1.1 获取完整复盘数据

**端点**: `GET /api/review/{date}`

**描述**: 获取指定日期的完整涨停复盘数据，包括市场情绪、涨跌停股票、区间统计等。

**路径参数**:
- `date` (string): 交易日期，格式 `YYYY-MM-DD`，例如 `2025-01-16`

**响应示例**:

```json
{
  "market_sentiment": {
    "date": "2025-01-16",
    "total_limit_up": 45,
    "total_limit_down": 12,
    "max_consecutive": 8,
    "sentiment_index": 72.5
  },
  "limit_up_stocks": [
    {
      "trade_date": "2025-01-16",
      "code": "300001",
      "name": "龙头A",
      "is_limit_up": 1,
      "consecutive_days": 5,
      "sealed_amount": 150000000.0,
      "limit_direction": "up",
      "max_consecutive": 8,
      "interval_stats": {
        "days_5_count": 3,
        "days_5_consecutive": 2,
        "days_10_count": 5,
        "days_10_consecutive": 3
      }
    }
  ],
  "limit_down_stocks": [],
  "interval_stats": {
    "days_5": {
      "count_8": 2,
      "count_7": 5,
      "count_6": 8,
      "count_5": 12,
      "count_4": 15,
      "count_3": 20,
      "count_2": 25,
      "count_1": 45
    },
    "days_10": {
      "count_8": 0,
      "count_7": 0,
      "count_6": 0,
      "count_5": 0,
      "count_4": 0,
      "count_3": 0,
      "count_2": 0,
      "count_1": 0
    },
    "days_20": {
      "count_8": 0,
      "count_7": 0,
      "count_6": 0,
      "count_5": 0,
      "count_4": 0,
      "count_3": 0,
      "count_2": 0,
      "count_1": 0
    }
  }
}
```

**字段说明**:

#### MarketSentiment（市场情绪）
| 字段 | 类型 | 说明 |
|------|------|------|
| date | string | 交易日期 |
| total_limit_up | number | 涨停股票数量 |
| total_limit_down | number | 跌停股票数量 |
| max_consecutive | number | 当日最大连板数 |
| sentiment_index | number | 市场情绪指数（-100到100） |

#### IntervalStats（区间统计）
| 字段 | 类型 | 说明 |
|------|------|------|
| count_8 | number | 8板股票数量 |
| count_7 | number | 7板股票数量 |
| count_6 | number | 6板股票数量 |
| count_5 | number | 5板股票数量 |
| count_4 | number | 4板股票数量 |
| count_3 | number | 3板股票数量 |
| count_2 | number | 2板股票数量 |
| count_1 | number | 1板股票数量 |

---

## 2. 题材分析API

### 2.1 获取题材热度榜

**端点**: `GET /api/themes/{date}/hotness`

**描述**: 获取指定日期的题材热度排行榜，按热度评分降序排列。

**路径参数**:
- `date` (string): 交易日期，格式 `YYYY-MM-DD`

**查询参数**:
- `limit` (number, 可选): 返回数量，默认20，最大100

**请求示例**:
```
GET /api/themes/2025-01-16/hotness?limit=20
```

**响应示例**:

```json
[
  {
    "trade_date": "2025-01-16",
    "theme_name": "人工智能",
    "theme_type": "concept",
    "stock_count": 150,
    "limit_up_count": 8,
    "limit_down_count": 2,
    "limit_up_ratio": 0.053,
    "avg_consecutive": 3.2,
    "max_consecutive": 5,
    "total_consecutive_gte_3": 6,
    "total_consecutive_gte_5": 2,
    "total_sealed_amount": 1500000000.0,
    "avg_sealed_amount": 187500000.0,
    "leader_code": "300001",
    "leader_name": "龙头A",
    "leader_consecutive": 5,
    "cycle_stage": "climax",
    "cycle_days": 5,
    "hotness_rank": 1,
    "hotness_score": 95.6,
    "created_at": "2025-01-16T15:30:00Z"
  }
]
```

**字段说明**:

#### ThemeHotness（题材热度）
| 字段 | 类型 | 说明 |
|------|------|------|
| theme_name | string | 题材名称 |
| theme_type | string | 题材类型：industry（行业）、concept（概念） |
| stock_count | number | 该题材包含的股票总数 |
| limit_up_count | number | 当日涨停股票数量 |
| limit_down_count | number | 当日跌停股票数量 |
| limit_up_ratio | number | 涨停比例（涨停数/总股票数） |
| avg_consecutive | number | 平均连板数 |
| max_consecutive | number | 最大连板数 |
| cycle_stage | string | 周期阶段：init、fermentation、climax、differentiation、recession |
| cycle_days | number | 当前周期持续天数 |
| hotness_rank | number | 热度排名 |
| hotness_score | number | 热度评分（0-100） |

---

### 2.2 获取题材详情

**端点**: `GET /api/themes/{date}/{theme_name}`

**描述**: 获取指定题材在特定日期的详细信息。

**路径参数**:
- `date` (string): 交易日期
- `theme_name` (string): 题材名称，需要URL编码，例如 `人工智能`

**请求示例**:
```
GET /api/themes/2025-01-16/人工智能
```

**响应示例**:

```json
{
  "theme_info": {
    "theme_name": "人工智能",
    "theme_type": "concept",
    "cycle_stage": "climax",
    "cycle_days": 5,
    "created_at": "2025-01-16T15:30:00Z"
  },
  "stocks": [
    {
      "role": "leader",
      "code": "300001",
      "name": "龙头A",
      "consecutive_days": 5,
      "is_limit_up": true
    },
    {
      "role": "mid",
      "code": "300002",
      "name": "中军B",
      "consecutive_days": 3,
      "is_limit_up": true
    }
  ],
  "statistics": {
    "total_stocks": 150,
    "limit_up_count": 8,
    "avg_consecutive": 3.2,
    "total_sealed_amount": 1500000000.0
  }
}
```

**字段说明**:

#### StockRole（股票角色）
| 值 | 说明 |
|----|------|
| leader | 龙头股（连板数最高） |
| mid | 中军股（跟涨但不是龙头） |
| follower | 跟风股（涨停较晚） |

---

### 2.3 获取题材关联图谱

**端点**: `GET /api/themes/relations`

**描述**: 获取题材之间的关联关系，用于构建题材关联网络图。

**查询参数**:
- `date` (string, 必需): 交易日期
- `theme` (string, 必需): 题材名称

**请求示例**:
```
GET /api/themes/relations?date=2025-01-16&theme=人工智能
```

**响应示例**:

```json
[
  {
    "trade_date": "2025-01-16",
    "parent_theme": "人工智能",
    "child_theme": "芯片",
    "relation_type": "upstream",
    "correlation_strength": 0.85,
    "common_stocks": 25,
    "common_limit_count": 8,
    "created_at": "2025-01-16T15:30:00Z"
  },
  {
    "trade_date": "2025-01-16",
    "parent_theme": "人工智能",
    "child_theme": "5G",
    "relation_type": "related",
    "correlation_strength": 0.72,
    "common_stocks": 18,
    "common_limit_count": 5,
    "created_at": "2025-01-16T15:30:00Z"
  }
]
```

**字段说明**:

#### RelationType（关联类型）
| 值 | 说明 |
|----|------|
| upstream | 上游关系 |
| downstream | 下游关系 |
| related | 相关关系 |

---

## 3. 错误响应

所有API在发生错误时返回统一的错误格式：

```json
{
  "error": "错误描述信息",
  "code": "ERROR_CODE",
  "timestamp": "2025-01-16T15:30:00Z"
}
```

**常见错误码**:

| 错误码 | HTTP状态码 | 说明 |
|--------|-----------|------|
| INVALID_DATE | 400 | 日期格式无效 |
| DATE_NOT_TRADED | 404 | 指定日期不是交易日 |
| DATABASE_ERROR | 500 | 数据库查询失败 |
| INTERNAL_ERROR | 500 | 服务器内部错误 |

---

## 4. 使用示例

### 4.1 获取今日复盘数据

```bash
curl -X GET "http://localhost:8080/api/review/2025-01-16" \
  -H "Content-Type: application/json"
```

### 4.2 获取题材热度榜Top10

```bash
curl -X GET "http://localhost:8080/api/themes/2025-01-16/hotness?limit=10" \
  -H "Content-Type: application/json"
```

### 4.3 获取题材详情

```bash
curl -X GET "http://localhost:8080/api/themes/2025-01-16/人工智能" \
  -H "Content-Type: application/json"
```

### 4.4 获取题材关联关系

```bash
curl -X GET "http://localhost:8080/api/themes/relations?date=2025-01-16&theme=人工智能" \
  -H "Content-Type: application/json"
```

---

## 5. 数据更新策略

### 5.1 更新频率

- **实时数据**: 交易时段每分钟更新
- **区间统计**: 每日15:30盘后更新
- **题材热度**: 每日15:30盘后更新
- **历史回溯**: 按需执行

### 5.2 缓存策略

- **Redis缓存**: 5分钟过期
- **CDN缓存**: 静态数据缓存1小时
- **浏览器缓存**: Cache-Control设置为1分钟

---

## 6. 性能指标

| API | 响应时间 (P95) | QPS限制 |
|-----|---------------|---------|
| GET /api/review/{date} | < 200ms | 100 |
| GET /api/themes/{date}/hotness | < 150ms | 200 |
| GET /api/themes/{date}/{theme_name} | < 100ms | 300 |
| GET /api/themes/relations | < 300ms | 50 |

---

## 7. 版本历史

| 版本 | 日期 | 变更说明 |
|------|------|----------|
| v2.0.0 | 2025-01-16 | 新增区间统计、题材分析功能 |
| v1.0.0 | 2024-12-01 | 初始版本，基础复盘功能 |

---

## 8. 相关文档

- [架构设计文档](../ARCHITECTURE.md)
- [部署指南](../DEPLOYMENT.md)
- [用户手册](../USER_GUIDE.md)
- [故障排查](../TROUBLESHOOTING.md)

---

**最后更新**: 2025-01-16
**维护者**: 开发团队
**联系方式**: support@example.com
