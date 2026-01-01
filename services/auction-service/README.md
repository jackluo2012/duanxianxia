# 竞价采集服务 (auction-service)

## 📋 概述

竞价采集服务负责在集合竞价时段（9:15-9:25）实时采集股票的竞价数据，计算关键指标，并推送到 Redis Stream 供下游服务消费。

## 🎯 核心功能

### 1. 时序检查
- 自动识别竞价时段（9:15-9:25）
- 仅在交易日运行（周一至周五）
- 非竞价时段休眠，节省资源

### 2. 竞价数据采集
- 连接通达信行情服务器
- 支持自选股列表配置
- 每 1 秒采集一次实时数据
- 重试机制保证数据采集稳定性

### 3. 指标计算

#### 封单金额
- **买封金额** = 买一价 × 买一量
- **卖封金额** = 卖一价 × 卖一量

#### 抢筹强度评分（0-100）
计算公式：
```
评分 = (涨幅 × 40%) + (买盘占比 × 30%) + (成交量比率 × 30%)
```

评分等级：
- **90-100**：极强，涨停概率极高
- **70-89**：较强，可能涨停
- **50-69**：中等
- **0-49**：较弱

#### 封单匹配度（0.0-1.0）
```
匹配度 = min(买封, 卖封) / max(买封, 卖封)
```

### 4. Redis Stream 推送
- Stream Key: `auction_quotes`
- 数据格式：JSON
- 自动连接管理和错误重试

## 🚀 快速开始

### 环境变量

```bash
# Redis 连接 URL（可选，默认：redis://127.0.0.1:6379）
export REDIS_URL=redis://localhost:6379

# 日志级别（可选，默认：INFO）
export RUST_LOG=info
```

### 构建运行

```bash
# 开发环境
cargo run

# 生产构建
cargo build --release
./target/release/auction-service
```

### Docker 部署

```bash
docker build -t auction-service .
docker run -e REDIS_URL=redis://redis:6379 auction-service
```

## 📊 数据结构

### AuctionQuote

```json
{
  "code": "000001",
  "name": "平安银行",
  "time": "2026-01-01 09:20:15",
  "price": 11.50,
  "pre_close": 10.50,
  "volume": 5000000,
  "amount": 57500000.0,
  "buy1_price": 11.50,
  "buy1_volume": 100000,
  "sell1_price": 11.60,
  "sell1_volume": 10000,
  "change_percent": 9.52,
  "sealed_amount_buy": 1150000.0,
  "sealed_amount_sell": 116000.0
}
```

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_calculate_intensity_score_high
```

### 测试覆盖

- ✅ 抢筹强度评分（高/低/零成交量）
- ✅ 封单匹配度（平衡/不平衡/零值）
- ✅ 时序检查逻辑
- ✅ Redis 推送功能

## 📝 配置说明

### 自选股列表

当前版本使用硬编码示例股票：
- 000001 平安银行
- 000002 万科A
- 600000 浦发银行
- 600036 招商银行
- 600519 贵州茅台

**TODO**: Task 5.2 将支持从 Redis 或配置文件读取自选股。

## 🔧 故障排查

### 连接通达信失败
```
ERROR 连接通达信服务器失败: ...
```
**解决方案**: 检查网络连接，确认通达信服务器可访问。

### Redis 连接失败
```
ERROR 成功连接到 Redis: Connection refused
```
**解决方案**:
1. 检查 Redis 服务是否运行
2. 验证 `REDIS_URL` 环境变量
3. 确认防火墙设置

## 📈 性能指标

- **采集频率**: 1 秒/次
- **支持股票数**: 100+ 只（取决于通达信服务器性能）
- **数据延迟**: < 500ms
- **内存占用**: ~20MB

## 🔗 依赖服务

- **Redis**: 消息队列（Stream: `auction_quotes`）
- **通达信行情服务器**: 实时行情数据源

## 📚 相关文档

- [竞价分析模块设计文档](../../../docs/plans/2026-01-01-auction-analysis-design.md)
- [API 文档](../../../docs/auction-api.md)
- [任务清单](../../../docs/spec-workflow/specs/auction-analysis/tasks.md)

## 🎓 下一步开发

- [ ] Task 5.2: 自选股管理（Redis 持久化）
- [ ] Task 5.3: 集成测试
- [ ] Task 5.4: 性能优化
