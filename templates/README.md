# 六边形架构服务模板

本目录包含两种六边形架构服务模板,用于快速创建新服务。

---

## 📁 模板类型

### 1. hexagonal-service-full (完整服务模板)

**适用场景:**
- ✅ 有复杂业务逻辑的服务
- ✅ 需要完整领域建模
- ✅ 多个实体和值对象
- ✅ 复杂的业务规则

**包含内容:**
```
hexagonal-service-full/
├── domain/                    # 独立的domain crate
│   ├── Cargo.toml
│   └── src/
│       ├── entities/          # 实体
│       ├── value_objects/     # 值对象
│       ├── services/          # 领域服务
│       └── ports/             # 端口定义
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    ├── service.rs
    ├── application/           # 应用层
    │   └── use_cases/
    └── adapters/              # 适配器层
        ├── primary/           # 主适配器(HTTP)
        └── secondary/         # 次适配器(Database)
```

**示例服务:**
- storage-service
- auction-storage
- auction-service
- backtest-service

---

### 2. hexagonal-service-simple (简化服务模板)

**适用场景:**
- ✅ 主要是技术适配器
- ✅ 业务逻辑简单
- ✅ 无需领域建模
- ✅ 单一功能服务

**包含内容:**
```
hexagonal-service-simple/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    ├── service.rs
    └── adapters/              # 只有适配器层
        ├── primary/           # 主适配器
        └── secondary/         # 次适配器
```

**示例服务:**
- realtime-service
- auth-service
- auction-realtime

---

## 🚀 使用模板

### 步骤1: 复制模板

```bash
# 完整服务(有复杂业务逻辑)
cp -r templates/hexagonal-service-full services/my-new-service

# 简化服务(简单功能)
cp -r templates/hexagonal-service-simple services/my-simple-service
```

### 步骤2: 替换占位符

```bash
cd services/my-new-service

# 替换服务名称
find . -type f \( -name "*.rs" -o -name "*.toml" \) -exec sed -i 's/{{service_name}}/my-new-service/g' {} +
find . -type f \( -name "*.rs" -o -name "*.toml" \) -exec sed -i 's/{{ServiceName}}/MyNewService/g' {} +
```

### 步骤3: 更新Cargo.toml

```toml
[package]
name = "my-new-service"
version = "0.1.0"
edition = "2021"

[dependencies]
# domain依赖(仅完整模板需要)
my-new-service-domain = { path = "domain" }

# 其他依赖...
```

### 步骤4: 自定义业务逻辑

1. **Domain层** (完整模板)
   - 修改 `domain/src/entities/` - 添加实体
   - 修改 `domain/src/value_objects/` - 添加值对象
   - 修改 `domain/src/services/` - 实现领域服务
   - 修改 `domain/src/ports/` - 定义端口

2. **Application层**
   - 修改 `src/application/use_cases/` - 添加用例

3. **Adapter层**
   - 修改 `src/adapters/primary/` - HTTP控制器
   - 修改 `src/adapters/secondary/` - 数据库/消息队列

### 步骤5: 编译和测试

```bash
# 编译
cargo build

# 运行
cargo run

# 测试
cargo test
```

---

## 📖 详细文档

请参阅完整的六边形架构开发指南:

**[docs/HEXAGONAL_GUIDE.md](../docs/HEXAGONAL_GUIDE.md)**

该指南包含:
- ✅ 架构原则和SOLID实践
- ✅ 详细的层次说明
- ✅ 完整的开发步骤
- ✅ 代码示例和最佳实践
- ✅ 常见问题解答

---

## 🎯 开发流程

### 完整服务开发流程

```
1. 领域建模 (1-2小时)
   ├─ 识别实体
   ├─ 定义值对象
   ├─ 设计领域服务
   └─ 定义端口接口

2. 实现domain层 (2-4小时)
   ├─ 实现实体行为
   ├─ 实现值对象验证
   ├─ 实现领域服务逻辑
   └─ 编写单元测试

3. 实现application层 (1-2小时)
   ├─ 创建用例
   ├─ 实现编排器
   └─ 集成测试

4. 实现adapter层 (2-4小时)
   ├─ HTTP控制器
   ├─ 数据库适配器
   ├─ Redis/MQ适配器
   └─ 集成测试

5. 组装和测试 (1-2小时)
   ├─ 主入口组装
   ├─ 端到端测试
   └─ 性能测试
```

### 简化服务开发流程

```
1. 设计适配器 (1小时)
   ├─ 主适配器接口
   └─ 次适配器接口

2. 实现适配器 (2-3小时)
   ├─ HTTP/WebSocket
   ├─ Redis/Database
   └─ 集成测试

3. 组装和测试 (1小时)
   ├─ 主入口组装
   └─ 端到端测试
```

---

## ✅ 质量检查清单

在提交代码前,确保:

**Domain层:**
- [ ] 实体包含业务行为
- [ ] 值对象不可变
- [ ] 领域服务无状态
- [ ] 单元测试覆盖率 > 90%

**Application层:**
- [ ] 用例职责单一
- [ ] 编排器协调正确
- [ ] 错误处理完善
- [ ] 集成测试覆盖 > 80%

**Adapter层:**
- [ ] 适配器只负责技术实现
- [ ] 不包含业务逻辑
- [ ] 错误转换为DomainError
- [ ] 集成测试覆盖 > 70%

**总体:**
- [ ] 编译0错误0警告
- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] 代码审查通过

---

## 🆘 需要帮助?

**查看文档:**
- 架构指南: [docs/HEXAGONAL_GUIDE.md](../docs/HEXAGONAL_GUIDE.md)
- 部署指南: [docs/DEPLOYMENT_GUIDE.md](../docs/DEPLOYMENT_GUIDE.md)
- 故障排查: [docs/TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)

**查看示例:**
- data-collector: 已实现的六边形架构服务
- services/目录下其他服务

---

**模板版本**: 2.0
**最后更新**: 2026-01-15
**维护者**: 开发团队
