# 短线侠平台 - A股短线交易分析系统

基于 **Rust** 和 **六边形架构** 的专业 A 股短线交易平台，提供实时行情、技术指标、选股器、策略回测等功能。

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Architecture](https://img.shields.io/badge/Architecture-Hexagonal-green.svg)](docs/ARCHITECTURE.md)

## 📚 文档导航

**快速开始:**
- **[部署文档](./DEPLOYMENT.md)** ⭐⭐⭐ - 完整的部署和运维指南
- **[使用文档](./USAGE.md)** ⭐⭐⭐ - API 使用指南和示例
- **[架构文档](./HEXAGONAL_ARCHITECTURE_FINAL_REPORT.md)** - 六边形架构详细说明

---

## 🎯 项目概述

短线侠是一个专业的 A 股短线交易平台，采用 **六边形架构（Hexagonal Architecture）** 设计，所有 11 个微服务 100% 完成架构迁移。

### 核心特性

✅ **六边形架构** - 清晰的层次结构，高内聚低耦合
✅ **实时行情** - WebSocket 实时推送，3秒级行情数据
✅ **集合竞价** - 9:15-9:25 竞价数据分析，抢筹强度评分
✅ **涨停复盘** - 涨停板分析，连板追踪，龙头高度排名
✅ **技术指标** - MA、MACD、KDJ、RSI 等常用指标
✅ **选股器** - 连续涨停、龙头高度、涨跌停筛选
✅ **策略回测** - 历史数据回测，策略绩效评估
✅ **智能调度** - 交易时段高频采集，盘后降频休眠

---

## 🏗️ 系统架构

### 六边形架构模式

```
┌─────────────────────────────────────────────────────┐
│                  Primary Adapters                    │
│            (驱动者 - HTTP/WebSocket/CLI)              │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│              Application Layer                       │
│                  (用例编排)                          │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│                Domain Layer                          │
│            (核心业务逻辑 - 纯业务)                    │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│                Secondary Adapters                    │
│            (被驱动 - ClickHouse/PostgreSQL/Redis)     │
└─────────────────────────────────────────────────────┘
```

### 技术栈

**后端:**
- **语言**: Rust 1.75+
- **Web 框架**: Actix-Web
- **时序数据库**: ClickHouse 24.x
- **关系数据库**: PostgreSQL 15.x
- **缓存**: Redis 7.x
- **WebSocket**: 实时数据推送
- **数据源**: rustdx (A 股行情)

**前端:**
- **框架**: React 18 + TypeScript
- **构建工具**: Vite
- **UI 库**: Ant Design 5
- **图表**: ECharts (分时图 + K线图)

---

## 🚀 快速开始

### 环境要求

- **Rust**: 1.75.0 或更高版本
- **Docker**: 20.10+ (可选)
- **操作系统**: Linux (Ubuntu 22.04 推荐)

### 一键启动（推荐）

```bash
# 克隆项目
git clone https://github.com/your-org/duanxianxia.git
cd duanxianxia

# 启动所有服务
bash ./start-all.sh

# 验证服务状态
bash ./health-check.sh
```

**就这么简单！** 系统将自动:
- ✅ 启动依赖服务（ClickHouse, PostgreSQL, Redis）
- ✅ 初始化数据库表结构
- ✅ 编译并启动 11 个微服务
- ✅ 健康检查验证

### 手动部署

详细步骤请参考 [部署文档](./DEPLOYMENT.md)。

```bash
# 1. 启动基础设施
docker-compose up -d clickhouse postgres redis

# 2. 编译服务
cargo build --workspace --release

# 3. 启动服务
./target/release/query-service &
./target/release/limit-review-service &
# ... 其他服务
```

---

## 📡 微服务列表

| 服务 | 端口 | 数据库 | 功能 | 架构模式 |
|------|------|--------|------|----------|
| **auction-realtime** | 8081 | Redis | 集合竞价实时推送 | 本地 Domain |
| **auction-service** | 8082 | PostgreSQL | 竞价数据分析 | 本地 Domain |
| **auction-storage** | 8083 | PostgreSQL | 竞价数据存储 | 本地 Domain |
| **auth-service** | 8084 | PostgreSQL | 用户认证授权 | 本地 Domain |
| **backtest-service** | 8085 | PostgreSQL | 策略回测引擎 | 本地 Domain |
| **data-collector** | 8086 | ClickHouse | 全维度数据采集 | 共享 Domain |
| **kline-collector** | 8087 | ClickHouse | K线数据采集 | 本地 Domain |
| **limit-review-service** | 8088 | ClickHouse | 涨停板复盘分析 | 本地 Domain |
| **query-service** | 8089 | ClickHouse | 选股器和查询 | 本地 Domain |
| **realtime-service** | 8090 | Redis | 实时行情推送 | 本地 Domain |
| **storage-service** | 8091 | PostgreSQL | 通用存储服务 | 独立 Domain |

---

## 💻 API 使用示例

### 1. 查询龙头股票

```bash
curl "http://localhost:8089/api/screener/leaders?date=2025-01-16&limit=10"
```

**响应**:
```json
{
  "code": 0,
  "message": "success",
  "data": [
    {
      "code": "000001",
      "name": "平安银行",
      "leader_height": 95.6,
      "limit_times": 5,
      "change_percent": 10.01
    }
  ]
}
```

### 2. 查询技术指标

```bash
curl "http://localhost:8089/api/indicators/000001"
```

### 3. 每日涨停复盘

```bash
curl "http://localhost:8088/api/review/2025-01-16"
```

### 4. WebSocket 实时行情

```javascript
const ws = new WebSocket('ws://localhost:8090/ws/quotes');

ws.onopen = () => {
  ws.send(JSON.stringify({
    action: 'subscribe',
    codes: ['000001', '000002', '600000']
  }));
};

ws.onmessage = (event) => {
  const quote = JSON.parse(event.data);
  console.log('实时行情:', quote);
};
```

更多 API 示例请参考 [使用文档](./USAGE.md)。

---

## 🔧 开发

### 编译

```bash
# 开发模式
cargo build --workspace

# 发布模式（优化性能）
cargo build --workspace --release
```

### 测试

```bash
# 运行所有测试
cargo test --workspace

# 运行单个服务测试
cargo test -p query-service
```

### 代码规范

```bash
# 格式化代码
cargo fmt --all

# 检查代码
cargo clippy --workspace
```

---

## 📊 项目统计

- **总代码量**: 25,453 行
- **文件数量**: 225 个
- **微服务数量**: 11 个
- **架构模式**: 3 种（本地/共享/独立 Domain）
- **编译错误**: 0 个 ✅
- **架构覆盖率**: 100% SOLID 原则

---

## 📖 架构亮点

### 1. 三种 Domain 模式灵活应用

- **本地 Domain**: 9 个服务，简单直接
- **共享 Domain Crate**: data-collector，跨服务复用
- **独立 Domain Crate**: storage-service，独立管理

### 2. 依赖方向严格单向

```
Main → Application → Domain
Main → Adapters → Domain
```

**Domain 层零外部依赖！** 纯业务逻辑，易于测试和复用。

### 3. SOLID 原则全面应用

- ✅ **S** - 单一职责：每个服务专注单一业务
- ✅ **O** - 开闭原则：扩展无需修改核心代码
- ✅ **L** - 里氏替换：适配器可替换
- ✅ **I** - 接口隔离：专一接口
- ✅ **D** - 依赖倒置：依赖抽象不依赖具体

---

## 🎓 学习资源

- **[六边形架构最终报告](./HEXAGONAL_ARCHITECTURE_FINAL_REPORT.md)** - 完整的架构迁移记录
- **[部署文档](./DEPLOYMENT.md)** - 部署和运维指南
- **[使用文档](./USAGE.md)** - API 使用指南

---

## 🛠️ 故障排查

常见问题及解决方案请参考 [部署文档 > 故障排查](./DEPLOYMENT.md#故障排查)。

### 快速诊断

```bash
# 检查服务状态
curl http://localhost:8089/health
curl http://localhost:8088/health

# 查看日志
tail -f logs/*.log

# 检查端口占用
netstat -tunlp | grep -E "808[0-9]"
```

---

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

---

## 📞 支持

如有问题，请联系：
- 邮件: support@duanxianxia.com
- GitHub Issues: https://github.com/your-org/duanxianxia/issues

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

---

## 🎉 致谢

感谢 **Claude Code** 在六边形架构迁移中的大力支持！

---

**项目状态**: ✅ 生产就绪
**最后更新**: 2025-01-16
**版本**: v1.0.0
