# K线采集服务 - 配置文件使用指南

## 📋 概述

K线采集服务支持灵活的配置管理，支持以下配置方式：

1. **配置文件**（推荐）- 使用TOML格式的配置文件
2. **环境变量** - 用于容器化部署或敏感信息覆盖
3. **默认值** - 当没有提供配置时使用

## 🎯 配置优先级

```
环境变量 > 配置文件 > 默认值
```

优先级说明：
- **环境变量**具有最高优先级，可以覆盖配置文件和默认值
- **配置文件**次之，适合生产环境部署
- **默认值**最低，确保服务始终可以启动

## 📂 配置文件位置

服务会按以下顺序查找配置文件（找到第一个即停止）：

1. `./config.toml` - 当前工作目录
2. `/etc/kline-collector/config.toml` - 系统级配置目录
3. `~/.config/kline-collector/config.toml` - 用户级配置目录

## 📄 配置文件示例

### 完整配置示例

```toml
# K线采集服务配置文件

[service]
# 服务名称
name = "kline-collector"
# 绑定地址
bind_address = "127.0.0.1:8080"
# 日志级别：trace, debug, info, warn, error
log_level = "info"

[datasource]
# Redis连接配置
redis_url = "redis://localhost:6379"
# Stream名称
stream_name = "stock_quotes"
# Rustdx连接池大小
rustdx_pool_size = 3

[periods]
# 启用的K线周期（逗号分隔）
enabled = ["1m", "5m", "15m", "30m", "60m", "1d"]

[batch]
# 基础批量大小（条数）
batch_size = 100
# 批量刷新基准间隔（秒）
write_interval_secs = 5

[backfill]
# 是否启用回填
enabled = true
# 启动时回填天数
startup_days = 7
# 定时回填时间（HH:MM格式）
schedule_time = "15:30"
# 最大并发任务数
max_concurrent_tasks = 5

[quality]
# 价格变动阈值（0-1之间，0.2表示20%）
price_change_threshold = 0.2
# 是否启用自动修复
enable_auto_repair = true
```

### 最小化配置示例

```toml
[datasource]
redis_url = "redis://prod-redis.example.com:6379"

[periods]
enabled = ["1m", "5m", "15m", "30m", "60m", "1d"]
```

## 🔧 环境变量配置

### 支持的环境变量列表

#### 服务配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `SERVICE_NAME` | 服务名称 | `kline-collector` | `export SERVICE_NAME=kline-prod` |
| `BIND_ADDRESS` | 绑定地址 | `127.0.0.1:8080` | `export BIND_ADDRESS=0.0.0.0:8080` |
| `LOG_LEVEL` | 日志级别 | `info` | `export LOG_LEVEL=debug` |

#### 数据源配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `REDIS_URL` | Redis连接URL | `redis://localhost:6379` | `export REDIS_URL=redis://prod:6379` |
| `STREAM_NAME` | Stream名称 | `stock_quotes` | `export STREAM_NAME=quotes` |
| `TDX_POOL_SIZE` | Rustdx池大小 | `3` | `export TDX_POOL_SIZE=5` |

#### 周期配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `ENABLED_PERIODS` | 启用的周期 | `1m,5m,15m,30m,60m,1d` | `export ENABLED_PERIODS=1m,5m` |

#### 批量配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `BATCH_SIZE` | 批量大小 | `100` | `export BATCH_SIZE=500` |
| `BATCH_INTERVAL_SECS` | 刷新间隔（秒） | `5` | `export BATCH_INTERVAL_SECS=10` |

#### 回填配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `BACKFILL_ENABLED` | 是否启用回填 | `true` | `export BACKFILL_ENABLED=false` |
| `STARTUP_DAYS` | 启动回填天数 | `7` | `export STARTUP_DAYS=30` |
| `SCHEDULE_TIME` | 定时回填时间 | `15:30` | `export SCHEDULE_TIME=16:00` |
| `MAX_CONCURRENT_TASKS` | 最大并发任务数 | `5` | `export MAX_CONCURRENT_TASKS=10` |

#### 质量配置

| 环境变量 | 说明 | 默认值 | 示例 |
|---------|------|--------|------|
| `PRICE_CHANGE_THRESHOLD` | 价格变动阈值 | `0.2` | `export PRICE_CHANGE_THRESHOLD=0.3` |
| `AUTO_REPAIR_ENABLED` | 是否自动修复 | `true` | `export AUTO_REPAIR_ENABLED=false` |

## 📖 使用场景

### 场景1：开发环境（使用默认配置）

```bash
# 直接启动，使用所有默认值
./kline-collector
```

### 场景2：生产环境（使用配置文件）

```bash
# 1. 创建配置文件
cat > config.toml << EOF
[datasource]
redis_url = "redis://prod-redis.example.com:6379"

[periods]
enabled = ["1m", "5m", "15m", "30m", "60m", "1d"]

[batch]
batch_size = 500
write_interval_secs = 10
EOF

# 2. 启动服务
./kline-collector
```

### 场景3：容器化部署（使用环境变量）

```bash
# Docker方式
docker run -d \
  -e REDIS_URL="redis://prod-redis:6379" \
  -e BATCH_SIZE="500" \
  -e LOG_LEVEL="debug" \
  -v /path/to/config.toml:/app/config.toml \
  kline-collector:latest

# Kubernetes ConfigMap + Secret方式
kubectl create configmap kline-config \
  --from-literal=REDIS_URL=redis://prod:6379 \
  --from-literal=BATCH_SIZE=500

kubectl create secret generic kline-secret \
  --from-literal=REDIS_PASSWORD=secret123
```

### 场景4：混合配置（配置文件 + 环境变量覆盖）

```bash
# config.toml 包含基础配置
[datasource]
redis_url = "redis://localhost:6379"

[batch]
batch_size = 100

# 使用环境变量覆盖敏感配置
export REDIS_URL="redis://secure-prod-redis:6379"
export BATCH_SIZE="500"

./kline-collector
# 最终配置: redis_url=redis://secure-prod-redis:6379, batch_size=500
```

## 🔍 配置验证

服务启动时会自动验证配置的有效性，包括：

- ✅ Redis URL不能为空
- ✅ 至少启用一个K线周期
- ✅ 批量间隔和批量大小必须大于0
- ✅ 回填天数必须大于0
- ✅ 价格变动阈值必须在0-1之间

如果配置无效，服务会拒绝启动并显示详细错误信息：

```bash
$ ./kline-collector
Error: 配置验证失败

Caused by:
   REDIS_URL cannot be empty
```

## 📊 配置加载日志

服务启动时会显示配置加载情况：

```log
✅ 配置加载完成
  🏷️  服务: kline-collector (127.0.0.1:8080)
  📡 Redis: redis://localhost:6379
  ⏱️  周期: ["1m", "5m", "15m", "30m", "60m", "1d"]
  📦 批量: 5秒 或 100条
  📜 回填: 7天
```

通过查看这些日志，可以确认配置是否按预期加载。

## 🛠️ 配置调试

### 查看实际加载的配置

```bash
# 启动时添加详细日志
RUST_LOG=debug ./kline-collector
```

### 测试配置文件语法

```bash
# 使用Python验证TOML语法
python3 << EOF
import tomli
with open('config.toml', 'rb') as f:
    config = tomli.load(f)
    print("✅ TOML语法正确")
    print(config)
EOF
```

### 常见配置错误

1. **TOML语法错误**
   ```
   Error: 解析TOML配置文件失败
   Caused by:
     TOML parse error at line 10, column 1
   ```
   **解决**：检查TOML语法，特别是引号和括号匹配

2. **配置文件未找到**
   ```
   # 如果所有位置都找不到配置文件，服务会使用默认值
   ✅ 配置加载完成
   ```
   **解决**：确认配置文件路径正确，或使用绝对路径

3. **环境变量类型错误**
   ```
   Error: Invalid BATCH_SIZE
   ```
   **解决**：确保环境变量的值类型正确（数字、布尔值等）

## 📚 最佳实践

1. **生产环境**
   - ✅ 使用配置文件存储所有非敏感配置
   - ✅ 使用环境变量或Secret管理敏感信息（密码、密钥）
   - ✅ 将配置文件纳入版本控制（排除敏感信息）
   - ✅ 使用配置验证确保配置正确

2. **开发环境**
   - ✅ 使用默认配置快速启动
   - ✅ 通过环境变量临时调整配置进行测试
   - ✅ 为不同环境创建不同的配置文件（dev.toml, test.toml, prod.toml）

3. **容器化部署**
   - ✅ 使用ConfigMap存储配置文件
   - ✅ 使用Secret存储敏感信息
   - ✅ 通过环境变量覆盖配置文件中的敏感字段

4. **配置管理**
   - ✅ 定期审查配置文件
   - ✅ 记录配置变更历史
   - ✅ 使用配置验证防止错误配置上线

## 🆘 故障排除

### 问题1：配置修改后不生效

**原因**：服务启动时读取配置，运行时修改不会自动生效

**解决**：重启服务使新配置生效

```bash
pkill -f kline-collector
./kline-collector
```

### 问题2：环境变量覆盖不生效

**原因**：环境变量名称拼写错误或值格式不正确

**解决**：
1. 检查环境变量名称拼写（区分大小写）
2. 确认环境变量已设置：`echo $REDIS_URL`
3. 验证值格式是否正确

### 问题3：配置文件权限问题

**错误**：`无法读取配置文件: Permission denied`

**解决**：
```bash
chmod 644 config.toml
```

---

**文档版本**: 1.0.0
**更新时间**: 2026-01-26
**服务版本**: kline-collector v0.1.0
