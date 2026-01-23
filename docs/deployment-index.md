# 部署文档导航

**最后更新:** 2026-01-23

---

## 🚀 快速部署

### 新手用户（推荐）

**5分钟快速部署:**
```bash
bash ./start-all.sh
bash ./health-check.sh
```

**详细步骤:** 请阅读 [快速开始指南](../QUICK_START.md)

---

## 📖 完整部署文档

### 主要文档

1. **[Hexagonal 架构文档](./HEXAGONAL_ARCHITECTURE.md)** ⭐⭐⭐ NEW
   - 六边形架构设计
   - Domain-Application-Adapters 分层
   - 端口和适配器模式
   - 性能指标和测试结果

2. **[完整部署指南](./DEPLOYMENT.md)** ⭐⭐⭐
   - 环境要求和检查
   - 三种部署模式（quick/full/update）
   - 详细部署步骤
   - 故障排查
   - 维护操作

3. **[快速开始指南](../QUICK_START.md)** ⭐⭐
   - 5分钟快速部署
   - 常用命令
   - 验证方法

4. **[故障排查指南](../TROUBLESHOOTING.md)** ⭐⭐
   - 常见问题解答
   - 问题诊断步骤
   - 解决方案

---

## 🛠️ 部署脚本

### 脚本列表

| 脚本 | 用途 | 适用场景 |
|------|------|----------|
| `start-all.sh` | 一键启动所有服务 | 新手、日常使用 |
| `stop-all.sh` | 停止所有服务 | 日常使用 |
| `check-env.sh` | 环境检查 | 部署前检查 |
| `health-check.sh` | 健康检查 | 验证服务状态 |
| `deploy.sh` | 多模式部署 | 高级用户 |

### 使用示例

**方式一：简单启动（推荐新手）**
```bash
bash ./start-all.sh
```

**方式二：带检查的部署**
```bash
# 1. 环境检查
bash ./check-env.sh

# 2. 部署
bash ./deploy.sh quick

# 3. 健康检查
bash ./health-check.sh
```

**方式三：多模式部署（高级）**
```bash
# 快速部署（重启服务，保留数据）
bash ./deploy.sh quick

# 完全部署（清理后重新部署）
bash ./deploy.sh full

# 增量更新（更新代码并重新编译）
bash ./deploy.sh update
```

---

## 📊 测试报告

### 部署相关测试

1. **[部署流程测试报告](../DEPLOYMENT_FLOW_TEST.md)** - 2026-01-13
   - 完整部署流程测试
   - 发现的问题和解决方案
   - 测试结果统计

2. **[数据采集测试报告](../DEPLOYMENT_TEST_REPORT.md)** - 2026-01-12
   - 真实数据采集测试
   - 性能指标
   - 数据完整性验证

---

## ⚠️ 重要说明

### Shell 兼容性

**所有部署脚本必须在 bash 环境中运行！**

```bash
# ✅ 正确
bash ./start-all.sh

# ❌ 错误（在 zsh 中可能失败）
./start-all.sh
```

### 数据采集时间

- **交易时段 (09:30-15:00):** 每 3 秒采集一次
- **竞价时段 (09:15-09:25):** 实时采集
- **非交易时段:** 服务休眠（正常行为）

查看日志确认状态：
```bash
tail -f logs/data-collector.log | grep "调度"
```

---

## 🎯 推荐阅读顺序

### 新用户
1. [README.md](../README.md) - 项目概览
2. [快速开始](../QUICK_START.md) - 5分钟部署
3. [完整部署指南](./DEPLOYMENT.md) - 详细步骤
4. [故障排查](../TROUBLESHOOTING.md) - 遇到问题时

### 高级用户
1. [完整部署指南](./DEPLOYMENT.md) - 所有部署选项
2. [系统架构](../ARCHITECTURE.md) - 技术架构
3. [用户指南](../USER_GUIDE.md) - 功能说明

---

## 📝 文档维护

### 文档版本

- **主文档:** `docs/deployment/DEPLOYMENT.md` (609行，最详细)
- **简化版:** `docs/DEPLOYMENT.md` (快速参考)
- **导航页:** 当前文档

### 更新记录

- 2026-01-13: 合并 feature/deployment-system 分支
- 2026-01-13: 添加 bash 环境检查
- 2026-01-13: 创建文档导航

---

**需要帮助？** 请查看 [故障排查指南](../TROUBLESHOOTING.md)
