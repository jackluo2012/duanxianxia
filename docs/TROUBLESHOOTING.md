# 故障排查文档

本文档记录了部署和运行过程中的常见问题及解决方案。

---

## 目录

1. [数据库相关问题](#数据库相关问题)
2. [数据采集问题](#数据采集问题)
3. [网络连接问题](#网络连接问题)
4. [性能优化建议](#性能优化建议)

---

## 数据库相关问题

### 问题 1: ClickHouse DateTime 类型序列化失败

**错误信息**:
```
schema mismatch: attempting to deserialize ClickHouse type DateTime as &str
```

**原因**:
- ClickHouse Rust 客户端在序列化 `chrono::DateTime<Utc>` 时存在类型不兼容
- 客户端期望的是特定的时间戳格式

**解决方案**:
使用 `UInt64` 存储 Unix 时间戳,查询时转换为 DateTime:

**表结构**:
```sql
CREATE TABLE stock_realtime_quotes (
    timestamp UInt64,  -- Unix timestamp (秒)
    ...
) ENGINE = MergeTree();
```

**Rust 结构体**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct StockQuote {
    pub timestamp: u64,  // Unix timestamp (秒)
    ...
}
```

**查询转换**:
```sql
SELECT
    code,
    price,
    toDateTime(timestamp, 'Asia/Shanghai') as datetime
FROM stock_realtime_quotes;
```

**相关文件**:
- `db/init.sql` - 表定义
- `services/data-collector/src/types.rs` - 数据结构

---

### 问题 2: 表缺少字段

**错误信息**:
```
database schema has no column named market
```

**原因**:
- 代码结构体包含字段,但数据库表中未定义

**解决方案**:
1. 检查代码中的结构体定义
2. 对比数据库表结构
3. 使用 `ALTER TABLE` 添加缺失字段:

```sql
ALTER TABLE stock_realtime_quotes
ADD COLUMN IF NOT EXISTS market UInt8;
```

**预防措施**:
- 在 `db/init.sql` 中保持表结构定义与代码同步
- 使用 `IF NOT EXISTS` 避免重复创建错误

---

## 数据采集问题

### 问题 1: 通达信 API 连接限制

**错误信息**:
```
Resource temporarily unavailable (os error 11)
Broken pipe (os error 32)
```

**原因**:
- 通达信服务器有并发连接限制
- 网络波动导致连接断开
- 大批量请求触发限流

**这是正常现象**,系统已有容错机制:
- 自动跳过失败批次
- 继续处理下一批次
- 不影响已采集数据的写入

**优化建议**:
1. 调整并发连接数 (`QuoteCollector::new(3, 80, 10)`)
2. 减少批次大小 (从80改为60)
3. 增加批次间延迟 (避免过快请求)

### 问题 2: 数据不完整

**错误信息**:
```
failed to fill whole buffer
```

**原因**:
- 网络不稳定
- 通达信服务器返回数据不完整
- 单批次数据量过大

**系统处理**:
- 自动跳过不完整的批次
- 下一轮重新采集
- 不影响已成功采集的数据

---

## 网络连接问题

### 问题 1: 无法连接通达信 API

**检查步骤**:

1. **检查网络连接**:
```bash
ping 119.147.212.81  # 通达信服务器
```

2. **检查防火墙**:
```bash
sudo ufw status
```

3. **检查代理设置**:
通达信API可能需要特定的网络环境

### 问题 2: Docker 网络问题

**症状**:
- 容器无法访问宿主机服务
- 无法连接 ClickHouse

**解决方案**:

1. **使用 host.docker.internal**:
```bash
REDIS_URL=redis://host.docker.internal:6379
```

2. **或者使用宿主机IP**:
```bash
# 获取 WSL2 IP
ip addr show eth0 | grep inet
```

---

## 性能优化建议

### 1. 数据采集性能

**当前配置**:
```rust
QuoteCollector::new(
    3,   // 3个 TCP 连接
    80,  // 每批 80 只股票
    10   // 超时 10 秒
)
```

**优化建议**:
- 减少批次大小: 80 → 60 (降低超时风险)
- 增加连接数: 3 → 5 (提高吞吐量)
- 调整超时时间: 根据网络状况调整

### 2. 数据写入性能

**当前配置**:
```rust
ClickHouseWriter::new(
    ch_client,
    1000,  // 批量大小 1000
    30,    // 超时 30秒
    3      // 重试 3 次
)
```

**优化建议**:
- 增加批量大小: 1000 → 2000 (提高写入效率)
- 减少超时时间: 30 → 20 (快速失败)
- 增加重试次数: 3 → 5 (提高可靠性)

### 3. 缓冲区管理

**当前配置**:
```rust
BufferManager::new(
    ch_writer,
    redis_conn,
    1000,  // 最大容量 1000
    5      // 刷新间隔 5秒
)
```

**优化建议**:
- 减小容量: 1000 → 500 (更快刷新,更实时)
- 减少间隔: 5 → 3 (更频繁写入)

---

## 日志查看

### 实时日志

```bash
# Data Collector
tail -f logs/data-collector.log

# Storage Service
tail -f logs/storage-service.log

# Realtime Service
tail -f logs/realtime-service.log
```

### 过滤错误

```bash
# 只看错误
grep "ERROR" logs/data-collector.log

# 只看警告
grep "WARN" logs/data-collector.log

# 统计错误数量
grep -c "ERROR" logs/data-collector.log
```

### 分析采集成功率

```bash
# 统计成功批次
grep -c "采集成功" logs/data-collector.log

# 统计失败批次
grep -c "采集失败" logs/data-collector.log

# 查看最新错误
grep "ERROR" logs/data-collector.log | tail -10
```

---

## 常用命令

### 数据库操作

```bash
# 查看 ClickHouse 表
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SHOW TABLES FROM duanxianxia"

# 查询表结构
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "DESCRIBE duanxianxia.stock_realtime_quotes"

# 查询数据量
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT count() FROM duanxianxia.stock_realtime_quotes"

# 查询最新数据
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT * FROM duanxianxia.stock_realtime_quotes ORDER BY timestamp DESC LIMIT 10"
```

### 服务管理

```bash
# 停止所有服务
./stop-all.sh

# 重置环境
./reset-all.sh

# 启动所有服务
./start-all.sh

# 查看服务状态
docker-compose ps

# 查看服务日志
docker-compose logs -f [service-name]
```

### 进程管理

```bash
# 查看运行中的服务
ps aux | grep -E "(data-collector|storage-service|realtime-service)"

# 根据端口查找进程
lsof -ti:8080

# 杀死进程
kill $(cat logs/data-collector.pid)
```

---

## 应急处理

### 数据库连接失败

**症状**: 无法连接 ClickHouse

**检查**:
1. 容器是否运行: `docker ps | grep clickhouse`
2. 端口是否监听: `lsof -ti:9000`
3. 网络是否连通: `telnet localhost 9000`

**解决**:
```bash
# 重启 ClickHouse
docker-compose restart clickhouse

# 重新初始化数据库
docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql
```

### 数据采集卡死

**症状**: 长时间没有新日志输出

**检查**:
1. 查看进程状态: `ps aux | grep data-collector`
2. 查看最新日志: `tail -20 logs/data-collector.log`
3. 检查网络连接: `netstat -an | grep 8080`

**解决**:
```bash
# 强制重启服务
kill -9 $(cat logs/data-collector.pid)
nohup ./target/debug/data-collector > logs/data-collector.log 2>&1 &
echo $! > logs/data-collector.pid
```

### 磁盘空间不足

**检查**:
```bash
df -h
docker system df
```

**清理**:
```bash
# 清理 Docker 未使用的资源
docker system prune -a

# 清理 ClickHouse 旧数据
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "ALTER TABLE stock_realtime_quotes DELETE WHERE timestamp < toUInt64(now() - INTERVAL 7 DAY)"
```

---

## 联系支持

如果以上方法无法解决问题,请收集以下信息:

1. 错误日志: `logs/*.log`
2. 服务状态: `docker-compose ps`
3. 系统信息: `uname -a`
4. Docker 版本: `docker --version`

并提交到项目 Issue 跟踪系统。
