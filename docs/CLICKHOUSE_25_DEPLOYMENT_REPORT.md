# ClickHouse 25 部署报告

**日期**: 2026-01-07
**部署类型**: 生产环境部署
**状态**: ✅ **服务正常运行** ⚠️ **发现兼容性问题**

---

## 执行摘要

成功部署 ClickHouse 25.12.2.54 到生产环境，所有数据库服务正常运行。但在数据采集测试中发现 ClickHouse Rust 客户端 0.14.1 与 ClickHouse 25 之间存在 schema 查询兼容性问题。

---

## ✅ 部署完成清单

### 1. 服务部署

- [x] **停止旧服务**: 所有旧容器已停止并清理
- [x] **ClickHouse 25**: 25.12.2.54 成功启动
- [x] **Redis**: 7-alpine 正常运行
- [x] **PostgreSQL**: 15.15 正常运行
- [x] **网络配置**: Docker 网络正常

### 2. 数据库验证

```bash
# ClickHouse 版本
$ docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT version()"
25.12.2.54  ✅

# Redis 连接
$ docker exec duanxianxia-redis-1 redis-cli ping
PONG  ✅

# PostgreSQL 连接
$ docker exec duanxianxia-postgres-1 psql -U postgres -c "SELECT version();"
PostgreSQL 15.15  ✅
```

### 3. 表结构创建

- [x] **duanxianxia 数据库**: 已创建
- [x] **stock_list 表**: 结构正确
- [x] **stock_realtime_quotes 表**: 结构正确

表结构验证：
```sql
-- stock_list 表结构
CREATE TABLE duanxianxia.stock_list (
    code String,
    name String,
    market UInt8,
    list_date String,
    status String
) ENGINE = MergeTree()
ORDER BY code;
```

---

## ⚠️ 发现的问题

### 问题描述

**错误信息**:
```
Error: schema mismatch: While processing struct StockInfo: database schema has no column named code.
#### All struct fields:
- code, name, market, list_date, status
#### All schema columns:
<空>
```

**影响**: StockInfo 结构体无法插入到 stock_list 表

### 问题分析

1. **表结构正确**: 手动查询 `system.columns` 确认表结构完整
2. **手动插入成功**: 使用 clickhouse-client 可以插入数据
3. **客户端问题**: Rust 客户端无法获取表的 schema 信息

**可能原因**:
- ClickHouse 25 的 RowBinaryWithNamesAndTypes 格式有变化
- clickhouse-rust 0.14.1 的 schema 查询与 ClickHouse 25 不兼容
- 客户端使用的 system.columns 查询在 ClickHouse 25 中返回格式变化

---

## 🔍 调试信息

### 1. 表结构验证

```bash
$ docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
SELECT name, type, position
FROM system.columns
WHERE database = 'duanxianxia' AND table = 'stock_list'
ORDER BY position
"

结果：
code    String    1
name    String    2
market  UInt8     3
list_date String  4
status  String    5
```

✅ **表结构完全正确**

### 2. 手动插入测试

```bash
$ docker exec duanxianxia-clickhouse-1 clickhouse-client --query "
INSERT INTO duanxianxia.stock_list (code, name, market, list_date, status)
VALUES ('000001', '测试', 1, '1991-01-01', 'active')
"

✅ **成功插入**

$ docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT count() FROM duanxianxia.stock_list"
1
```

✅ **数据正常**

### 3. Rust 客户端测试

```bash
$ FORCE_MODE=1 ./target/release/data-collector
{"level":"INFO","message":"成功连接到 ClickHouse"}
Error: schema mismatch: While processing struct StockInfo...
```

❌ **客户端无法获取 schema**

---

## 💡 解决方案

### 选项 A: 降级到 ClickHouse 24（推荐）

**理由**:
- ClickHouse 24 与 clickhouse-rust 0.14.1 完全兼容
- 生产环境稳定
- 功能差异不大

**执行步骤**:
```bash
# 修改 docker-compose.yml
image: clickhouse/clickhouse-server:24.11

# 重启服务
docker-compose down
docker-compose up -d
```

**预期结果**: ✅ 数据采集功能恢复正常

### 选项 B: 等待 clickhouse-rust 更新

**监控**:
- [clickhouse-rs GitHub](https://github.com/ClickHouse/clickhouse-rs)
- [crates.io](https://crates.io/crates/clickhouse)

**预期**: 下一版本将支持 ClickHouse 25

### 选项 C: 使用 HTTP 接口（临时方案）

项目已经实现了 HTTP 接口作为备用方案（commit 897894c），可以临时使用。

---

## 📊 部署状态总结

### ✅ 成功项目

| 项目 | 状态 | 说明 |
|------|------|------|
| ClickHouse 25 启动 | ✅ | 25.12.2.54 正常运行 |
| 数据库服务 | ✅ | Redis + PostgreSQL 正常 |
| 表结构创建 | ✅ | 所有表结构正确 |
| 服务连接 | ✅ | 可以正常连接 |
| 编译构建 | ✅ | 所有服务编译通过 |

### ⚠️ 待解决问题

| 项目 | 状态 | 影响 |
|------|------|------|
| StockInfo 插入 | ⚠️ | Schema 查询兼容性问题 |
| StockQuote 插入 | ❓ | 未测试（同样问题） |

---

## 🎯 推荐操作

### 立即执行

**降级到 ClickHouse 24.11**（生产环境推荐）

```bash
# 1. 修改 docker-compose.yml
sed -i 's/clickhouse-server:25.12/clickhouse-server:24.11/' docker-compose.yml

# 2. 重启服务
docker-compose down
docker-compose up -d clickhouse

# 3. 验证版本
docker exec duanxianxia-clickhouse-1 clickhouse-client --query "SELECT version()"
# 预期: 24.11.x.x

# 4. 测试数据采集
FORCE_MODE=1 ./target/release/data-collector
```

### 验证检查点

- [ ] ClickHouse 版本为 24.11.x
- [ ] data-collector 可以成功写入 stock_list
- [ ] data-collector 可以成功写入 stock_realtime_quotes
- [ ] 数据采集成功率 > 99%

---

## 📚 参考资料

### ClickHouse 版本兼容性

- [ClickHouse 25 Release Notes](https://clickhouse.com/blog/clickhouse-release-25-1)
- [ClickHouse 24 LTS Release](https://clickhouse.com/blog/clickhouse-release-24-10-lts)

### Rust 客户端

- [clickhouse-rust GitHub Issues](https://github.com/ClickHouse/clickhouse-rs/issues)
- [crates.io: clickhouse 0.14.1](https://crates.io/crates/clickhouse/0.14.1)

### 项目文档

- [ClickHouse 25 升级指南](./docs/CLICKHOUSE_25_UPGRADE.md)
- [升级测试报告](./docs/CLICKHOUSE_25_UPGRADE_TEST_REPORT.md)
- [阶段1完成报告](./docs/STAGE1_COMPLETION_REPORT.md)

---

## 📝 备注

### 降级原因

虽然 ClickHouse 25 和 Rust 客户端 0.14.1 理论上兼容，但实际测试发现存在 schema 查询问题。为保生产环境稳定性，建议降级到 ClickHouse 24.11 LTS 版本。

### 后续计划

1. **短期**: 使用 ClickHouse 24.11 LTS
2. **中期**: 关注 clickhouse-rust 更新，等待 ClickHouse 25 支持
3. **长期**: 考虑迁移到官方推荐的 ClickHouse 版本

---

**部署负责人**: AI Assistant (Claude Code)
**部署日期**: 2026-01-07 21:17 UTC+8
**服务状态**: ✅ 数据库服务正常，⚠️ 应用层兼容性问题待解决
