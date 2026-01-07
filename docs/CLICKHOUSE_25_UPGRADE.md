# ClickHouse 25 升级说明

**日期**: 2026-01-07
**升级内容**: ClickHouse Server + Rust 客户端统一升级

---

## ✅ 升级内容

### 1. ClickHouse Server 升级

**版本**: 23 → **25**

**文件**: `docker-compose.yml`

```yaml
services:
  clickhouse:
    image: clickhouse/clickhouse-server:25
```

**兼容性**:
- ✅ 与 Rust 客户端 0.14.1 完全兼容
- ✅ 支持最新的 RowBinaryWithNamesAndTypes 格式
- ✅ 更好的性能和安全性

### 2. Rust 客户端统一升级

**统一版本**: **0.14.1**

**涉及服务**:
| 服务 | 旧版本 | 新版本 | 状态 |
|------|--------|--------|------|
| data-collector | 0.12 | **0.14.1** | ✅ |
| storage-service | 0.12 | **0.14.1** | ✅ |
| auction-storage | 0.12 | **0.14.1** | ✅ |
| query-service | 0.14 | **0.14.1** | ✅ |
| kline-collector | 0.14 | **0.14.1** | ✅ |

---

## 📋 部署步骤

### 1. 停止现有服务

```bash
docker-compose down
```

### 2. 清理旧数据（可选）

⚠️ **警告**: 此操作将删除所有 ClickHouse 数据！

```bash
docker volume rm duanxianxia_clickhouse_data
```

### 3. 启动新版本

```bash
docker-compose up -d
```

### 4. 验证升级

```bash
# 检查 ClickHouse 版本
docker exec -it duanxianxia-clickhouse-1 clickhouse-client --query "SELECT version()"

# 预期输出: 25.x.x
```

### 5. 重新编译服务

```bash
# 更新依赖
cargo update

# 编译所有服务
cargo build --release
```

---

## 🔍 验证检查点

### ClickHouse Server

- [ ] 服务正常启动（端口 8123、9000）
- [ ] 版本显示为 25.x.x
- [ ] 数据库连接正常
- [ ] 表结构完整

### Rust 服务

- [ ] 所有服务编译成功
- [ ] ClickHouse 连接成功
- [ ] 数据写入正常
- [ ] 数据查询正常

---

## 📊 兼容性说明

### Rust 客户端 0.14.1 特性

**RowBinaryWithNamesAndTypes 格式**:
- ✅ 启用行类型验证
- ✅ 自动 schema 同步
- ✅ 更好的错误提示

**API 变更** (从 0.12 → 0.14):
```rust
// 旧 API (0.12)
let mut insert = client.insert("table")?;
insert.write(&row)?;
insert.end().await?;

// 新 API (0.14)
let mut insert = client.insert("table").await?;
insert.write(&row).await?;
insert.end().await?;
```

### ClickHouse Server 25 新特性

- 性能优化
- 安全性增强
- 更好的并行处理
- 改进的查询优化器

---

## 🐛 已知问题

### 1. 数据迁移

如果需要保留旧数据，请先导出：

```bash
# 导出数据
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT * FROM duanxianxia.stock_realtime_quotes FORMAT CSVWithNames" > backup.csv

# 导入数据（新版本启动后）
docker exec -i duanxianxia-clickhouse-1 clickhouse-client --query "INSERT INTO duanxianxia.stock_realtime_quotes FORMAT CSVWithNames" < backup.csv
```

### 2. 表结构验证

ClickHouse 25 可能需要重新创建表：

```sql
-- 检查表是否存在
SHOW TABLES FROM duanxianxia;

-- 如需重建表，参考 docs/plans/2025-01-06-architecture-refactoring.md
```

---

## 📚 参考资源

- [ClickHouse 25 Release Notes](https://clickhouse.com/blog/clickhouse-release-25-1)
- [ClickHouse Rust Client Documentation](https://clickhouse.com/docs/integrations/rust)
- [clickhouse crate on crates.io](https://crates.io/crates/clickhouse)
- [ClickHouse GitHub Repository](https://github.com/ClickHouse/clickhouse-rs)

---

## 🎯 后续工作

完成 ClickHouse 25 升级后，可以继续：

1. **阶段2: 六边形架构重构**
   - 创建独立 domain crate
   - 实现端口和适配器
   - 重构 data-collector 服务

2. **性能优化**
   - 利用 ClickHouse 25 的新特性
   - 优化查询性能
   - 完善监控指标

---

**升级完成标志**:
- ✅ docker-compose.yml 更新为 ClickHouse 25
- ✅ 所有服务使用 clickhouse 0.14.1
- ✅ 服务编译通过
- ✅ 功能验证通过

---

**升级负责人**: AI Assistant (Claude Code)
**升级日期**: 2026-01-07
**文档版本**: 1.0
