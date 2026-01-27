# K线采集服务 - 数据迁移指南

**问题**: ClickHouse `kline_*` 表中没有数据

**根本原因**: 历史回填功能 `fetch_day_klines()` 未实现（返回空数组）

**解决方案**: 将现有的 `stock_realtime_quotes` 数据迁移到 Redis Stream，服务会自动消费并聚合

---

## 🔍 问题分析

### 1. 表结构对比

| 表名 | 记录数 | 说明 |
|------|--------|------|
| kline_1m | 0 | ❌ 空表（聚合后数据） |
| kline_5m | 0 | ❌ 空表（聚合后数据） |
| kline_15m | 0 | ❌ 空表（聚合后数据） |
| kline_30m | 0 | ❌ 空表（聚合后数据） |
| kline_60m | 0 | ❌ 空表（聚合后数据） |
| kline_1d | 0 | ❌ 空表（聚合后数据） |
| **stock_realtime_quotes** | **1548** | ✅ 有数据（原始行情） |

### 2. 数据流程

```
行情数据源
    ↓
Redis Stream (stock_quotes)
    ↓
kline-collector 聚合引擎
    ↓
ClickHouse kline_* 表
```

当前状态：
- ✅ 数据源: stock_realtime_quotes (1548条)
- ❌ Redis Stream: 空
- ❌ kline_* 表: 空

---

## ✅ 解决方案

### 方案 1: 数据迁移脚本（推荐）

**步骤 1**: 创建迁移脚本

已创建：`scripts/migrate_to_redis.py`

**步骤 2**: 安装依赖

```bash
pip install redis requests
```

**步骤 3**: 运行迁移

```bash
cd /home/jackluo/data/duanxianxia/services/kline-collector
python3 scripts/migrate_to_redis.py
```

**步骤 4**: 启动 kline-collector 服务

```bash
/home/jackluo/data/duanxianxia/target/release/kline-collector
```

**步骤 5**: 验证数据

```bash
curl "http://localhost:8123/?query=SELECT%20count()%20FROM%20duanxianxia.kline_1m"
```

预期结果：应该看到 >0 的数据

---

### 方案 2: 手动迁移（如果 Python 不可用）

使用 Rust 迁移工具：

```bash
cd /home/jackluo/data/duanxianxia/services/kline-collector/scripts
cargo build --release --bin migrate_to_redis
./target/release/migrate_to_redis
```

---

### 方案 3: 启动实时行情推送（长期方案）

如果有实时行情推送服务，确保它：

1. 连接到 Redis: `localhost:6379`
2. 写入 Stream: `stock_quotes`
3. 数据格式：
   ```json
   {
     "timestamp": "1737972000",
     "code": "000001",
     "name": "平安银行",
     "price": "12.50",
     "volume": "1000.0",
     "amount": "12500.0"
   }
   ```

kline-collector 会自动：
- 从 Redis Stream 读取
- 聚合成多周期K线
- 写入 ClickHouse kline_* 表

---

## 🔧 完整实现历史回填功能

### 集成第三方数据源

修改 `src/domain/services/history_backfill.rs`:

```rust
async fn fetch_day_klines(
    &self,
    date: NaiveDate,
    period: KlinePeriod,
) -> Result<Vec<KlineData>> {
    // 方案 A: 使用 Tushare Pro
    let api_url = format!(
        "http://api.tushare.pro/api/tsapi/bar?tk={}&dt={}",
        self.tushare_token,
        date.format("%Y%m%d")
    );
    // ... 调用API并返回数据

    // 方案 B: 使用 AKShare (通过 Python bridge)
    // ... 调用 akshare.get_kline()

    // 方案 C: 使用 rustdx 实时采集并存储
    // ... 连续采集当日数据
}
```

**推荐数据源**:

1. **Tushare Pro** ⭐⭐⭐⭐⭐
   - 官方: https://tushare.pro
   - 数据质量: 高
   - 成本: 免费/付费（积分制）
   - API: 稳定

2. **AKShare** ⭐⭐⭐⭐
   - 官方: https://akshare.akfamily.xyz
   - 数据质量: 中高
   - 成本: 免费
   - API: Python

3. **东方财富API** ⭐⭐⭐
   - 官方: https://data.eastmoney.com
   - 数据质量: 中
   - 成本: 免费
   - API: HTTP接口

---

## 📊 数据验证

### 检查迁移是否成功

```bash
# 1. 检查 Redis Stream
redis-cli XLEN stock_quotes

# 2. 检查 kline 表数据量
for period in 1m 5m 15m 30m 60m 1d; do
  count=$(curl -s "http://localhost:8123/?query=SELECT%20count()%20FROM%20duanxianxia.kline_${period}")
  echo "kline_${period}: ${count}"
done

# 3. 查看最新数据
curl -s "http://localhost:8123/?query=SELECT%20*%20FROM%20duanxianxia.kline_1m%20ORDER%20BY%20timestamp%20DESC%20LIMIT%205%20FORMAT%20Pretty"
```

---

## 🎯 总结

### 问题原因
- 历史回填功能未实现（TODO）
- fetch_day_klines() 返回空数组

### 解决方案
1. ✅ 迁移现有数据到 Redis Stream
2. ✅ 服务自动消费并聚合
3. ✅ 数据写入 kline_* 表

### 下一步
1. 运行迁移脚本
2. 启动服务
3. 验证数据
4. (可选) 实现完整的历史回填功能

---

**创建日期**: 2026-01-27
**文档状态**: ✅ 完成
