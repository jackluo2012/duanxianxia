# K线采集服务 (kline-collector)

实时K线数据采集和聚合服务，支持多周期K线计算和ClickHouse持久化存储。

## 功能特性

- ✅ **实时聚合**：从Redis Stream读取实时行情，多周期K线聚合
- ✅ **多周期支持**：1m、5m、15m、30m、60m、1d六个周期
- ✅ **智能批量**：自适应批量写入策略，优化ClickHouse性能
- ✅ **配置灵活**：支持TOML配置文件和环境变量
- ✅ **HTTP API**：服务状态监控和手动回填接口
- ✅ **定时回填**：支持历史数据回填和定时任务调度

## 快速开始

### 1. 编译服务

```bash
cd /home/jackluo/data/duanxianxia/services/kline-collector
cargo build --release
```

### 2. 配置服务

创建或修改 `config.toml`:

```toml
[datasource]
redis_url = "redis://localhost:6379"
stream_name = "stock_quotes"

[periods]
enabled = ["1m", "5m", "15m", "30m", "60m", "1d"]
```

详细配置说明请参考 [CONFIG_GUIDE.md](CONFIG_GUIDE.md)

### 3. 启动服务

```bash
# 前台运行（开发调试）
./target/release/kline-collector

# 后台运行（生产环境）
nohup ./target/release/kline-collector > production.log 2>&1 &
```

### 4. 验证服务

```bash
# 健康检查
curl http://127.0.0.1:8081/health

# 查看服务状态
curl http://127.0.0.1:8081/api/status
```

## 配置说明

服务支持三层配置优先级：

```
环境变量 > 配置文件 > 默认值
```

### 配置文件位置

服务按以下顺序查找配置文件：
1. `./config.toml` - 当前目录
2. `/etc/kline-collector/config.toml` - 系统配置
3. `~/.config/kline-collector/config.toml` - 用户配置

### 环境变量

支持通过环境变量覆盖配置：

```bash
# 覆盖Redis地址
REDIS_URL="redis://prod-redis:6379" ./kline-collector

# 覆盖日志级别
LOG_LEVEL=debug ./kline-collector

# 覆盖批量大小
BATCH_SIZE=500 ./kline-collector
```

完整的环境变量列表请参考 [CONFIG_GUIDE.md](CONFIG_GUIDE.md)

## 数据注入

服务从Redis Stream读取行情数据，数据格式：

```bash
docker exec duanxianxia-redis-1 redis-cli XADD stock_quotes * \
  timestamp "$(date +%s)" \
  code "600519" \
  name "贵州茅台" \
  price "1680.50" \
  volume "1000" \
  amount "1680500"
```

## HTTP API

### 健康检查

```bash
curl http://127.0.0.1:8081/health
```

### 服务状态

```bash
curl http://127.0.0.1:8081/api/status
```

响应示例：

```json
{
  "active_windows": 6,
  "is_healthy": true
}
```

### 手动回填

```bash
curl -X POST http://127.0.0.1:8081/api/backfill \
  -H "Content-Type: application/json" \
  -d '{"days": 7, "periods": ["1m", "5m"]}'
```

## 数据库

### ClickHouse表结构

```sql
-- 1分钟K线表
CREATE TABLE duanxianxia.kline_1m (
  timestamp UInt32,
  datetime DateTime,
  code String,
  name String,
  period String,
  open Float64,
  high Float64,
  low Float64,
  close Float64,
  volume Float64,
  amount Float64,
  trade_count UInt32,
  source String
)
ENGINE = MergeTree()
PARTITION BY toDateTime(timestamp)
ORDER BY (code, timestamp);
```

其他周期表结构类似，只需修改表名。

### 查询数据

```bash
# 查询1分钟K线
curl -s http://localhost:8123 --data "
SELECT
  toDateTime(timestamp) as time,
  code,
  name,
  open,
  high,
  low,
  close,
  volume
FROM duanxianxia.kline_1m
WHERE code = '600519'
ORDER BY time DESC
LIMIT 10
FORMAT Pretty
"
```

## 监控和日志

### 查看日志

```bash
# 实时查看日志
tail -f production.log

# 查看错误日志
grep ERROR production.log

# 查看数据处理日志
grep "从Redis读取\|处理.*行情\|闭合.*窗口" production.log
```

### 监控指标

服务内置监控指标：

- 活跃窗口数（active_windows）
- 服务健康状态（is_healthy）
- 数据读取量
- 窗口闭合数量

## 故障排除

### 服务启动失败

1. 检查Redis连接：`docker exec duanxianxia-redis-1 redis-cli PING`
2. 检查ClickHouse连接：`curl http://localhost:8123`
3. 查看错误日志：`grep ERROR production.log`

### 数据未写入ClickHouse

1. 确认数据已注入Redis：`docker exec duanxianxia-redis-1 redis-cli XLEN stock_quotes`
2. 检查消费者组状态：`docker exec duanxianxia-redis-1 redis-cli XINFO GROUPS stock_quotes`
3. 查看处理日志：`grep "从Redis读取" production.log`

### 窗口未闭合

窗口闭合需要时间跨分钟边界。确保注入的数据时间戳跨过分钟边界。

## 性能优化

### 批量写入配置

```toml
[batch]
batch_size = 500           # 增大批量大小
write_interval_secs = 10   # 延长刷新间隔
```

### 数据库优化

- 使用分区表（按日期分区）
- 定期清理旧数据
- 优化索引（code, timestamp）

## 文档

- [配置指南](CONFIG_GUIDE.md) - 详细配置说明
- [架构设计](docs/plans/2026-01-26-kline-collector-design.md) - 系统架构

## 技术栈

- **语言**: Rust 2021
- **异步运行时**: Tokio
- **数据库**: ClickHouse
- **消息队列**: Redis Streams
- **Web框架**: Actix-web
- **配置**: TOML

## 版本

- **当前版本**: 0.1.0
- **最后更新**: 2026-01-26
