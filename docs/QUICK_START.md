# 短线侠 - 快速入门指南

> 5 分钟快速部署并体验短线侠系统

---

## 前置条件检查

在开始之前，请确保您的系统已安装以下软件：

```bash
# 检查 Docker
docker --version
# 输出示例: Docker version 20.10.x

# 检查 Docker Compose
docker-compose --version
# 输出示例: Docker Compose version v2.x.x

# 检查 Rust
rustc --version
# 输出示例: rustc 1.70.x

# 检查 Node.js
node --version
# 输出示例: v18.x.x
```

如果未安装，请参考 [部署安装文档](./DEPLOYMENT.md#环境准备)。

---

## 一键启动

### 步骤 1: 获取项目代码

```bash
# 如果已有项目代码，跳过此步骤
cd /path/to/duanxianxia
```

### 步骤 2: 启动所有服务

```bash
./start-all.sh
```

该脚本会自动完成：
- ✅ 启动数据库服务（Redis, ClickHouse, PostgreSQL）
- ✅ 初始化数据库表结构
- ✅ 创建测试用户
- ✅ 编译并启动后端服务（首次需要 5-10 分钟）
- ✅ 显示服务状态和日志命令

### 步骤 3: 验证部署

```bash
# 运行测试脚本
./test-data-flow.sh
```

看到以下输出表示部署成功：
```
✅ Redis 运行中
✅ ClickHouse 运行中
✅ PostgreSQL 运行中
✅ data-collector 运行中
✅ storage-service 运行中
✅ realtime-service 运行中
✅ auth-service 运行中
```

### 步骤 4: 启动前端

**新开一个终端窗口**：

```bash
cd frontend

# 安装依赖（首次运行）
npm install

# 启动开发服务器
npm run dev
```

看到以下输出表示前端启动成功：
```
  VITE v5.0.8  ready in 500 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

---

## 访问系统

### 浏览器访问

在浏览器中打开：

```
http://localhost:5173
```

### 测试账号登录

```
用户名: testuser
密码: password123
```

---

## 快速体验功能

### 1. 查看实时行情

1. 登录后进入首页
2. 在搜索框输入股票代码（如 `000001`）
3. 点击 "订阅" 按钮查看实时分时图
4. 切换到 "5分钟" 或 "日线" 查看 K 线图

### 2. 竞价分析（交易时段）

1. 点击顶部导航 "竞价分析"
2. 查看 4 种排行榜：买封、强度、涨幅、异动
3. 点击任意股票查看竞价详情
4. 查看竞价曲线图和实时数据

### 3. 创建告警规则

1. 进入 "竞价分析" → "告警配置"
2. 点击 "新建告警"
3. 选择告警类型（如 "价格涨幅"）
4. 设置阈值（如 "涨幅 >= 5%"）
5. 点击 "保存" 创建告警

### 4. 管理自选股

1. 进入 "竞价分析" → "自选股管理"
2. 点击 "添加股票"
3. 输入股票代码（如 `600000`）
4. 点击 "确认" 添加到自选股

---

## 常用操作

### 查看服务状态

```bash
# 查看 Docker 容器状态
docker-compose ps

# 查看后端服务日志
tail -f logs/*.log

# 查看特定服务日志
tail -f logs/data-collector.log
```

### 停止所有服务

```bash
./stop-all.sh
```

### 重启服务

```bash
# 先停止
./stop-all.sh

# 再启动
./start-all.sh
```

### 清理并重新部署

```bash
# 停止并删除所有容器和数据卷
docker-compose down -v

# 重新启动
./start-all.sh
```

---

## 故障快速排查

### 问题 1: PostgreSQL 端口冲突

**错误信息**:
```
Bind for 0.0.0.0:5432 failed: port is already allocated
```

**解决方案**: 已在 `docker-compose.yml` 中将 PostgreSQL 端口改为 5433，无需手动修改。

### 问题 2: 后端服务编译失败

**错误信息**:
```
error: linking with `cc` failed
```

**解决方案**:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
```

### 问题 3: 看不到实时数据

**原因**: 当前不在 A 股交易时段

**交易时段**:
- 上午: 9:30 - 11:30
- 下午: 13:00 - 15:00

**竞价时段**:
- 上午: 9:15 - 9:25

**解决方案**:
- 在交易时段查看
- 或启用测试模式（编辑 `services/data-collector/.env`，设置 `FORCE_MODE=true`）

### 问题 4: 前端无法连接后端

**检查清单**:
```bash
# 1. 确认后端服务运行
curl http://localhost:8082/health  # 认证服务
curl http://localhost:8083/health  # 存储服务

# 2. 查看后端服务日志
tail -f logs/*.log

# 3. 检查浏览器控制台错误
# F12 → Console 标签
```

---

## 下一步

### 学习更多

- 📖 [完整部署文档](./DEPLOYMENT.md) - 详细部署步骤和配置
- 📚 [用户使用指南](./USER_GUIDE.md) - 所有功能说明和最佳实践
- 🏗️ [系统架构文档](./ARCHITECTURE.md) - 技术架构和设计

### 进阶功能

1. **自定义告警规则**: 根据自己的策略设置告警条件
2. **API 集成**: 使用 REST API 和 WebSocket 集成到自己的应用
3. **数据分析**: 导出历史数据进行量化分析
4. **性能优化**: 根据实际需求调整系统配置

### 参与贡献

欢迎提交 Issue 和 Pull Request！

- [GitHub Issues](https://github.com/your-repo/duanxianxia/issues)
- [贡献指南](CONTRIBUTING.md)

---

## 端口速查

| 服务 | 端口 | 用途 |
|------|------|------|
| Redis | 6379 | 消息队列 |
| ClickHouse | 8123, 9000 | 时序数据库 |
| PostgreSQL | 5433 | 用户数据库 |
| auth-service | 8082 | 认证 API |
| storage-service | 8083 | 存储查询 API |
| realtime-service | 8080 | WebSocket 服务 |
| Frontend | 5173 | 前端开发服务器 |

---

## 获取帮助

遇到问题？

1. 查看 [常见问题](./USER_GUIDE.md#常见问题)
2. 查看 [故障排查](./DEPLOYMENT.md#故障排查)
3. 提交 [GitHub Issue](https://github.com/your-repo/duanxianxia/issues)

---

**祝您使用愉快！** 📈
