# Phase 4: 切换和清理 - 执行计划

**创建日期**: 2026-01-08
**预计工期**: 1-2 天
**状态**: 🚀 进行中

---

## 执行摘要

Phase 4 的目标是将流量从旧的 `data-collector` 逐步迁移到新的六边形架构服务 `hexagonal-collector`，确保零停机、零数据丢失和平滑过渡。

---

## 📋 Phase 4 任务清单

### ✅ 已完成
- [x] Phase 1-3 完全完成
- [x] 新的 bin target `hexagonal-collector` 创建完成
- [x] 六边形架构服务验证通过
- [x] 数据采集和写入正常

### 🔄 进行中
- [ ] 并行运行测试
- [ ] 数据一致性验证
- [ ] 性能对比
- [ ] 切换脚本创建

### 📅 待完成
- [ ] 流量切换 (10% → 50% → 100%)
- [ ] 监控和验证
- [ ] 旧代码备份
- [ ] 清理和更新

---

## 阶段 1: 并行运行验证 (当前)

### 目标
同时运行两个版本，对比数据质量和性能

### 执行步骤

#### 1.1 准备测试环境

```bash
# 创建测试数据库表（用于并行测试）
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
CREATE TABLE IF NOT EXISTS duanxianxia.stock_realtime_quotes_legacy
AS duanxianxia.stock_realtime_quotes
"
```

#### 1.2 启动旧版本（写入 legacy 表）

**修改旧版本配置**:
- 使用独立的测试股票列表
- 写入到 `stock_realtime_quotes_legacy` 表
- 记录详细的性能指标

#### 1.3 启动新版本（写入主表）

**使用当前配置**:
- 写入到 `stock_realtime_quotes` 表
- 记录详细的性能指标

#### 1.4 数据对比

**对比指标**:
1. 数据完整性
   - 行数对比
   - 字段完整性
   - 数据格式一致性

2. 性能指标
   - 采集延迟
   - CPU 使用率
   - 内存占用
   - 错误率

3. 业务指标
   - 数据成功率
   - 重试次数
   - 异常处理

---

## 阶段 2: 数据一致性验证

### 验证方法

#### 2.1 SQL 对比查询

```sql
-- 对比两个表的数据量
SELECT
    'legacy' as source, count(*) as total_rows,
    count(DISTINCT code) as unique_stocks,
    min(timestamp) as first_time,
    max(timestamp) as last_time
FROM duanxianxia.stock_realtime_quotes_legacy
UNION ALL
SELECT
    'hexagonal' as source, count(*) as total_rows,
    count(DISTINCT code) as unique_stocks,
    min(timestamp) as first_time,
    max(timestamp) as last_time
FROM duanxianxia.stock_realtime_quotes;
```

#### 2.2 数据质量检查

```sql
-- 检查关键字段
SELECT
    code,
    count(*) as records,
    avg(price) as avg_price,
    max(price) - min(price) as price_range
FROM duanxianxia.stock_realtime_quotes
GROUP BY code
ORDER BY code;
```

#### 2.3 实时数据同步验证

- 对比同一时间点的数据
- 验证价格差异 < 0.01
- 验证时间戳同步 < 1秒

---

## 阶段 3: 性能对比测试

### 测试方法

#### 3.1 资源监控

```bash
# 监控旧版本
pidof data-collector | xargs ps -p | awk '{print $1, $3, $4}'

# 监控新版本
pidof hexagonal-collector | xargs ps -p | awk '{print $1, $3, $4}'
```

#### 3.2 性能基准

| 指标 | 旧版本 | 新版本 | 目标 |
|------|--------|--------|------|
| 采集延迟 | ?ms | 66ms | ≤ 旧版本 |
| CPU 使用 | ?% | ?% | ≤ 旧版本 |
| 内存占用 | ?MB | ?MB | ≤ 旧版本 |
| 成功率 | ?% | 100% | ≥ 旧版本 |
| 错误率 | ?% | 0% | ≤ 旧版本 |

---

## 阶段 4: 切换计划

### Week 1: 并行运行 (当前)

**目标**: 验证新版本稳定性

- [x] 启动 `hexagonal-collector`
- [ ] 启动 `data-collector` (配置不同股票列表)
- [ ] 对比数据质量
- [ ] 性能基准测试
- [ ] 问题修复和优化

**成功标准**:
- 新版本成功率 ≥ 99.9%
- 新版本延迟 ≤ 旧版本
- 数据一致性 100%

### Week 2: 小规模切换 (10%)

**目标**: 切换 10% 流量到新版本

**方法**:
1. 选择 10% 的股票（约 50-100 只）
2. 旧版本移除这些股票
3. 新版本添加这些股票
4. 监控 24-48 小时

**回滚计划**:
- 如果成功率 < 99%
- 如果延迟增加 > 20%
- 如果出现任何数据丢失

### Week 3: 中规模切换 (50%)

**目标**: 切换 50% 流量到新版本

**前置条件**:
- Week 2 成功
- 无重大问题
- 性能达标

**方法**:
1. 选择 50% 的股票（约 500-1000 只）
2. 更新配置并重启
3. 监控 48-72 小时

### Week 4: 完全切换 (100%)

**目标**: 100% 使用新版本

**前置条件**:
- Week 3 成功
- 所有指标正常
- 团队确认

**方法**:
1. 停止旧版本
2. 新版本采集全部股票
3. 监控 72 小时
4. 确认稳定后清理旧代码

---

## 阶段 5: 监控和验证

### 关键指标

#### 5.1 业务指标

```sql
-- 实时监控查询
SELECT
    toStartOfMinute(toDateTime(timestamp)) as minute,
    count(*) as quotes_per_minute,
    count(DISTINCT code) as unique_stocks,
    avg(price) as avg_price
FROM duanxianxia.stock_realtime_quotes
WHERE timestamp > now() - 300
GROUP BY minute
ORDER BY minute DESC
LIMIT 30;
```

#### 5.2 性能指标

**监控脚本** (创建 `scripts/monitor_hexagonal.sh`):
```bash
#!/bin/bash
# Monitor hexagonal-collector performance

while true; do
    echo "=== $(date) ==="

    # Check if process is running
    if pgrep -f hexagonal-collector > /dev/null; then
        echo "✅ Service: Running"
    else
        echo "❌ Service: Not Running"
    fi

    # Check memory usage
    ps aux | grep hexagonal-collector | grep -v grep | awk '{print "Memory:", $6/1024 "MB"}'

    # Check recent data
    docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
        SELECT count(*) as recent_quotes
        FROM duanxianxia.stock_realtime_quotes
        WHERE timestamp > unix_timestamp(now() - 60)
    "

    sleep 60
done
```

#### 5.3 错误监控

**日志监控**:
```bash
# 查看错误日志
docker logs duanxianxia-clickhouse-1 2>&1 | grep -i error | tail -20

# 查看服务日志
journalctl -u data-collector -f | grep -E "ERROR|WARN"
```

---

## 阶段 6: 清理和更新

### 6.1 备份旧代码

```bash
# 创建 legacy 分支
git checkout -b legacy/data-collector-backup

# 移动旧代码到 legacy 文件夹
git mv src/main.rs src/main.legacy.rs
git mv src/quote_collector.rs src/quote_collector.legacy.rs
git mv src/clickhouse_writer.rs src/clickhouse_writer.legacy.rs

# 提交备份
git commit -m "backup: Legacy code before hexagonal migration"

# 切换回主分支
git checkout main
```

### 6.2 更新 Cargo.toml

```toml
[[bin]]
name = "hexagonal-collector"
path = "src/hexagonal_main.rs"

# 移除旧的 bin target
# [[bin]]
# name = "data-collector"
# path = "src/main.rs"
```

### 6.3 更新文档

- `README.md`: 更新启动命令
- `DEPLOYMENT.md`: 更新部署流程
- `ARCHITECTURE.md`: 更新架构说明
- `OPERATIONS.md`: 添加运维手册

### 6.4 清理旧代码

**保留文件** (暂时):
- `src/main.legacy.rs` - 备份
- `src/quote_collector.legacy.rs` - 备份
- `src/clickhouse_writer.legacy.rs` - 备份

**最终清理** (1个月后):
- 如果新版本稳定运行 1 个月
- 删除所有 `.legacy.rs` 文件
- 从 Git 历史中清理（可选）

---

## 📊 执行检查清单

### Week 1: 并行运行
- [ ] 启动旧版本（写入 legacy 表）
- [ ] 启动新版本（写入主表）
- [ ] 数据一致性验证
- [ ] 性能对比测试
- [ ] 问题修复
- [ ] 团队评审

### Week 2: 10% 切换
- [ ] 选择测试股票列表
- [ ] 更新配置
- [ ] 执行切换
- [ ] 监控 24 小时
- [ ] 数据验证
- [ ] 性能验证

### Week 3: 50% 切换
- [ ] 准备切换计划
- [ ] 更新配置
- [ ] 执行切换
- [ ] 监控 48 小时
- [ ] 问题处理

### Week 4: 100% 切换
- [ ] 准备完全切换
- [ ] 停止旧版本
- [ ] 监控 72 小时
- [ ] 备份旧代码
- [ ] 清理旧代码
- [ ] 更新文档

---

## 🎯 成功标准

### 技术指标
- ✅ 数据采集成功率 ≥ 99.9%
- ✅ 数据一致性 100%
- ✅ 延迟 ≤ 旧版本
- ✅ CPU 使用 ≤ 旧版本
- ✅ 内存占用 ≤ 旧版本

### 业务指标
- ✅ 零数据丢失
- ✅ 零停机时间
- ✅ 错误率 < 0.1%
- ✅ 重试次数 < 1%

### 架构指标
- ✅ 所有测试通过
- ✅ 代码覆盖率 > 80%
- ✅ 文档完整
- ✅ 团队培训完成

---

## ⚠️ 风险和缓解

### 风险 1: 数据丢失

**概率**: 低
**影响**: 高
**缓解**:
- 并行运行验证
- 实时数据对比
- 快速回滚机制

### 风险 2: 性能下降

**概率**: 低
**影响**: 中
**缓解**:
- 性能基准测试
- 逐步切换
- 资源监控

### 风险 3: 兼容性问题

**概率**: 低
**影响**: 中
**缓解**:
- 充分测试
- 保留旧代码
- 回滚计划

---

## 📞 支持和联系

### 紧急回滚

```bash
# 停止新版本
pkill -f hexagonal-collector

# 启动旧版本
cargo run --bin data-collector
```

### 监控命令

```bash
# 查看服务状态
systemctl status data-collector

# 查看日志
journalctl -u data-collector -f

# 查看数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
    SELECT count(*) FROM duanxianxia.stock_realtime_quotes
    WHERE timestamp > unix_timestamp(now() - 300)
"
```

---

**文档状态**: 🚀 进行中
**最后更新**: 2026-01-08
**当前阶段**: Week 1 - 并行运行验证
**下一步**: 启动并行运行测试
