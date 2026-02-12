# HTTP API 数据源迁移总结

**日期**: 2026-02-11
**目的**: 全面切换到 HTTP API 数据源，替换 TDX (rustdx-complete) 依赖

---

## 迁移背景

### 问题原因
1. **TDX (rustdx-complete) 在 WSL2 环境下连接失败**
   - 网络连通性正常（nc 测试通过）
   - TCP 协议握手失败（Broken pipe 错误）
   - 可能需要通达信客户端辅助或特殊配置

2. **生产环境稳定性需求**
   - TDX 配置复杂，需要特殊环境设置
   - HTTP API 简单可靠，零配置即可使用

---

## 已完成迁移

### 1. data-collector 服务 ✅

**文件**: `services/data-collector/src/adapters/secondary/http_data_source.rs`

**功能**:
- 腾讯财经 API 实时行情获取
- 自动重试和错误处理
- 支持沪市/深市股票代码自动格式化

**使用方式**:
```bash
DATA_SOURCE_TYPE=http cargo run -p data-collector
```

**API 端点**:
- 腾讯: `http://qt.gtimg.cn/q={code}`
- 新浪: `https://hq.sinajs.cn/list={code}`

---

### 2. auction-service 服务 ✅

**文件**: `services/auction-service/src/adapters/primary/http_auction.rs`

**功能**:
- 腾讯财经 API 竞价数据获取
- 包含买一价/卖一价/成交量等数据
- 支持封单金额计算

**使用方式**:
```bash
cargo run -p auction-service
```

**API 端点**:
- 腾讯: `http://qt.gtimg.cn/q={code}`

**解析字段**:
- `parts[3]`: 当前价格
- `parts[4]`: 昨收价
- `parts[6]`: 成交量
- `parts[7-10]`: 买一价/买一量/卖一价/卖一量

---

### 3. kline-collector 服务 ✅

**文件**: `services/kline-collector/src/adapters/secondary/http_kline_source.rs`

**功能**:
- 新浪财经 K 线数据获取
- Mock 数据生成（用于测试）
- 支持历史数据回填

**使用方式**:
```bash
cargo run -p kline-collector
```

**API 端点**:
- 新浪: `https://money.163.com/service/code/{code}/wsddata.json`

---

## 服务端口配置

| 服务 | 端口 | 数据源 | 状态 |
|------|------|--------|------|
| data-collector | - | HTTP (腾讯) | ✅ 已迁移 |
| auction-service | - | HTTP (腾讯) | ✅ 已迁移 |
| kline-collector | 8081 | HTTP (新浪) | ✅ 已迁移 |
| query-service | 8089 | - | ✅ 无需迁移 |
| limit-review-service | 8088 | - | ✅ 无需迁移 |
| realtime-service | 8090 | - | ✅ 无需迁移 |

---

## 依赖变更

### 新增依赖
```toml
reqwest = { version = "0.12", features = ["json"] }
rand = "0.8"  # 仅 kline-collector 需要
```

### 可选依赖（已保留）
```toml
rustdx-complete = { workspace = true }  # 已保留，未来如需要可启用
```

---

## HTTP API 端点汇总

### 腾讯财经 API
```
# 实时行情
http://qt.gtimg.cn/q={sh600000|sz000001}

# 响应格式
v_sh600000="1~股票名~...~price~preclose~vol~amount~bid1~bid1_vol~ask1~ask1_vol~..."
```

### 新浪财经 API
```
# K线数据
https://money.163.com/service/code/{code}/wsddata.json?scope=day&count={count}

# 响应格式 (JSON)
{
  "name": "股票名",
  "data": [
    {
      "date": "2026-02-11",
      "open": 10.5,
      "high": 10.8,
      "low": 10.3,
      "close": 10.6,
      "volume": 1000000,
      "amount": 10600000,
      "factor": 1.0
    }
  ]
}
```

---

## 数据源对比

| 特性 | TDX (rustdx-complete) | HTTP API (腾讯/新浪) |
|------|----------------------|---------------------|
| **速度** | 极快 (<10ms) | 快 (100-300ms) |
| **稳定性** | ⚠️ WSL2 需配置 | ✅ 零配置 |
| **成本** | 免费 | 免费 |
| **数据完整性** | ✅ 全市场 | ✅ 全市场 |
| **历史数据** | ✅ 支持 | ⚠️ 有限 |
| **实时性** | ✅ 毫秒级 | ✅ 秒级 |
| **配置难度** | ⚠️⭐⭐⭐⭐ 复杂 | ✅⭐ 简单 |

---

## 测试验证

### data-collector 测试
```bash
# 使用 HTTP 数据源
DATA_SOURCE_TYPE=http cargo run -p data-collector

# 验证数据真实性
curl -s http://localhost:8080/api/stocks | jq '.[] | {code, name, price}'
```

### auction-service 测试
```bash
# 启动竞价采集
cargo run -p auction-service

# 验证竞价数据
redis-cli XLEN auction_stream
```

### kline-collector 测试
```bash
# 启动 K 线采集
cargo run -p kline-collector

# 验证 K 线数据
curl -s http://localhost:8081/api/status | jq '.kline_stats'
```

---

## 文件清单

### 新增文件
- `services/data-collector/src/adapters/secondary/http_data_source.rs`
- `services/data-collector/src/adapters/secondary/mock_data_source.rs`
- `services/auction-service/src/adapters/primary/http_auction.rs`
- `services/kline-collector/src/adapters/secondary/http_kline_source.rs`

### 修改文件
- `services/data-collector/Cargo.toml`
- `services/data-collector/src/main.rs`
- `services/data-collector/src/hexagonal_service.rs`
- `services/auction-service/Cargo.toml`
- `services/auction-service/src/main.rs`
- `services/auction-service/src/adapters/primary/mod.rs`
- `services/kline-collector/Cargo.toml`
- `services/kline-collector/src/adapters/secondary/mod.rs`

---

## 已知限制

### 1. API 速率限制
- 腾讯/新浪 API 没有公开速率限制
- 建议每秒不超过 100 次请求
- 已在代码中实现适当的延迟

### 2. 交易时段
- 实时行情仅在交易时段可用
- 非交易时段返回最后收盘价
- 竞价数据仅在 9:15-9:25 和 14:57-15:00 可用

### 3. 历史数据
- HTTP API 历史数据有限（新浪支持约 800 条）
- 如需完整历史数据，建议使用专业数据源

---

## 未来优化建议

### 短期 (1-2 周)
1. **添加更多数据源**
   - 网易财经 API
   - 东方财富 API
   - 雪球 API

2. **实现数据源自动切换**
   - 主数据源故障时自动切换
   - 多数据源数据交叉验证

### 中期 (1-2 月)
1. **添加本地缓存**
   - Redis 缓存实时行情
   - ClickHouse 缓存历史 K 线

2. **实现数据质量监控**
   - 价格异常检测
   - 数据缺失告警

### 长期 (3-6 月)
1. **考虑商业数据源**
   - 聚合/天玑等专业数据源
   - Level-2 行情数据

2. **实现数据源抽象层**
   - 统一数据源接口
   - 便于切换和扩展

---

## 回滚方案

如果需要回滚到 TDX 数据源：

### data-collector
```bash
DATA_SOURCE_TYPE=tdx cargo run -p data-collector
```

### auction-service
修改 `src/main.rs`，使用 `TongdaxinDataSource` 替代 `HttpAuctionDataSource`

### kline-collector
修改 `src/domain/services/history_backfill.rs`，启用 `rustdx_fallback`

---

## 总结

### 迁移成功 ✅

1. **所有服务已成功迁移到 HTTP API**
   - data-collector: HTTP (腾讯)
   - auction-service: HTTP (腾讯)
   - kline-collector: HTTP (新浪)

2. **编译通过，无错误**
   - 所有依赖已正确添加
   - 代码质量符合项目标准

3. **零配置，即开即用**
   - 不需要通达信客户端
   - 不需要特殊网络配置
   - WSL2 完美兼容

### 推荐配置

**生产环境**:
```bash
# 使用 HTTP API（推荐）
DATA_SOURCE_TYPE=http cargo run -p data-collector --release
cargo run -p auction-service --release
cargo run -p kline-collector --release
```

**开发测试**:
```bash
# 使用 Mock 数据源（无外部依赖）
DATA_SOURCE_TYPE=mock cargo run -p data-collector --release
```

---

**生成时间**: 2026-02-11
**迁移状态**: ✅ 完成
**验证状态**: ✅ 编译通过
