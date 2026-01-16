# 六边形架构迁移与项目清理方案

**创建日期**: 2026-01-15
**项目**: 短线侠 - A股实时行情分析平台
**目标**: 全部服务迁移到六边形架构 + 项目文档清理

---

## 📋 执行摘要

本方案旨在将短线侠系统的所有服务迁移到统一的六边形架构,并清理项目中的冗余文档和文件,实现代码库的精简和架构的现代化。

**核心目标:**
1. ✅ 所有服务采用六边形架构(复杂服务独立domain,简单服务简化重构)
2. ✅ 只保留最新文档,清理所有过时、重复的文档
3. ✅ 统一部署方式和脚本
4. ✅ 建立可复用的服务开发模板

**预期成果:**
- 清晰的项目结构
- 统一的架构风格
- 完善的文档体系
- 生产就绪的代码质量

**总时间估算**: 8-10天

---

## 🎯 第一部分:六边形架构迁移策略

### 1.1 服务分类与处理方式

#### **复杂服务(需要独立 domain crate)**

这些服务有复杂的业务逻辑,需要完整的DDD建模:

1. **storage-service** (数据存储服务)
   - 领域模型: `Query`, `DataBatch`, `StorageMetrics`
   - 值对象: `TimeRange`, `QueryCriteria`
   - 领域服务: `BatchWriter`, `QueryOptimizer`
   - 迁移时间: 2天

2. **auction-storage** (竞价数据存储服务)
   - 领域模型: `AuctionQuote`, `AlertRule`, `WatchlistItem`
   - 值对象: `AuctionTime`, `AlertCondition`, `StrengthScore`
   - 领域服务: `AlertEvaluator`, `RankingCalculator`
   - 迁移时间: 2天

3. **auction-service** (竞价数据采集服务)
   - 领域模型: `CollectingTask`, `MarketStatus`, `StrengthScorer`
   - 值对象: `CollectingInterval`, `MarketStatus`
   - 领域服务: `StrengthScorer`, `SealedAmountCalculator`
   - 迁移时间: 1.5天

4. **backtest-service** (回测服务)
   - 领域模型: `Backtest`, `Strategy`, `Trade`, `PerformanceMetrics`
   - 值对象: `DateRange`, `PositionSize`
   - 领域服务: `BacktestExecutor`, `PerformanceCalculator`
   - 迁移时间: 1.5天

#### **简单服务(不需要独立 domain)**

这些服务主要是技术适配器,业务逻辑简单:

5. **realtime-service** - WebSocket广播,无复杂业务逻辑 (0.5天)
6. **auth-service** - 标准JWT认证,使用现有库 (0.5天)
7. **auction-realtime** - WebSocket广播 (0.5天)

#### **已完成的六边形架构服务**

8. **data-collector** - ✅ 已完成六边形架构重构 (0.5天优化对齐)

### 1.2 架构模板设计

#### 完整服务模板(复杂服务)

```
service-name/
├── Cargo.toml
├── src/
│   ├── main.rs                    # 入口点
│   ├── service.rs                 # 服务封装
│   ├── config.rs                  # 配置管理
│   ├── application/               # 应用层
│   │   ├── mod.rs
│   │   ├── orchestrator.rs        # 编排器
│   │   └── use_cases/             # 用例
│   │       ├── mod.rs
│   │       ├── query_use_case.rs
│   │       └── command_use_case.rs
│   ├── adapters/                  # 适配器层
│   │   ├── mod.rs
│   │   ├── primary/               # 主适配器(驱动)
│   │   │   ├── mod.rs
│   │   │   └── http_controller.rs
│   │   └── secondary/             # 次适配器(被驱动)
│   │       ├── mod.rs
│   │       ├── database.rs
│   │       └── message_queue.rs
│   └── types.rs                   # 共享类型
├── domain/                        # 领域层(独立的crate)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entities/              # 实体
│       │   ├── mod.rs
│       │   └── *.rs
│       ├── value_objects/         # 值对象
│       │   ├── mod.rs
│       │   └── *.rs
│       ├── services/              # 领域服务
│       │   ├── mod.rs
│       │   └── *.rs
│       └── ports/                 # 端口定义
│           ├── mod.rs
│           ├── primary/           # 主端口
│           │   ├── mod.rs
│           │   └── *.rs
│           └── secondary/         # 次端口
│               ├── mod.rs
│               └── *.rs
└── tests/
    ├── unit_test.rs
    └── integration_test.rs
```

#### 简化服务模板(简单服务)

```
service-name/
├── Cargo.toml
├── src/
│   ├── main.rs                    # 入口点
│   ├── service.rs                 # 服务封装
│   ├── config.rs                  # 配置管理
│   ├── adapters/                  # 适配器层
│   │   ├── mod.rs
│   │   ├── primary/               # 主适配器
│   │   │   ├── mod.rs
│   │   │   └── *.rs
│   │   └── secondary/             # 次适配器
│   │       ├── mod.rs
│   │       └── *.rs
│   └── types.rs                   # 共享类型
└── tests/
    └── integration_test.rs
```

---

## 🗂️ 第二部分:文档清理策略

### 2.1 根目录文档清理

#### 保留的核心文档

✅ **README.md** - 项目主文档(需要更新架构说明)
✅ **CHANGELOG.md** - 变更日志(保留历史记录)

#### 删除的临时报告文档

❌ **DAY5_SUMMARY.md** - Day 5临时总结
❌ **DEPLOYMENT_DOCUMENTATION_TEST.md** - 部署文档测试报告
❌ **DEPLOYMENT_INTEGRATION_SUMMARY.md** - 部署集成总结
❌ **DEPLOYMENT_MERGE_COMPLETE_REPORT.md** - 部署合并完成报告
❌ **FRONTEND_BACKEND_INTEGRATION_COMPLETE.md** - 前后端集成完成报告
❌ **PHASE2_WEEK2_DAY1_SUMMARY.md** - Phase 2 Day 1总结
❌ **PHASE2_WEEK2_PLAN.md** - Phase 2 Week 2计划
❌ **PHASE2_WEEK2_PROGRESS.md** - Phase 2 Week 2进度
❌ **REAL_DB_INTEGRATION_REPORT.md** - 真实数据库集成报告
❌ **STARTUP_SCRIPTS_UPDATED.md** - 启动脚本更新报告
❌ **TASK_0_ENVIRONMENT_VERIFICATION.md** - 环境验证报告

**清理命令:**
```bash
# 创建备份
mkdir -p /tmp/duanxianxia-backup-$(date +%Y%m%d)
cp *.md /tmp/duanxianxia-backup-$(date +%Y%m%d)/

# 删除临时文档
rm -f DAY5_SUMMARY.md
rm -f DEPLOYMENT_DOCUMENTATION_TEST.md
rm -f DEPLOYMENT_INTEGRATION_SUMMARY.md
rm -f DEPLOYMENT_MERGE_COMPLETE_REPORT.md
rm -f FRONTEND_BACKEND_INTEGRATION_COMPLETE.md
rm -f PHASE2_WEEK2_DAY1_SUMMARY.md
rm -f PHASE2_WEEK2_PLAN.md
rm -f PHASE2_WEEK2_PROGRESS.md
rm -f REAL_DB_INTEGRATION_REPORT.md
rm -f STARTUP_SCRIPTS_UPDATED.md
rm -f TASK_0_ENVIRONMENT_VERIFICATION.md
```

### 2.2 docs/ 目录清理

#### 保留的核心文档

✅ **QUICK_START.md** - 快速开始指南
✅ **USER_GUIDE.md** - 用户使用指南
✅ **TROUBLESHOOTING.md** - 故障排查指南
✅ **deployment/** - 部署文档目录
  - `DEPLOYMENT.md` - 完整部署指南
  - `deployment-index.md` - 部署文档索引
✅ **deployment-index.md** - 部署文档索引(根级导航)
✅ **reports/** - 报告目录
  - `2026-01-09-backtest-service-completion.md` - 保留最新完成报告

#### 删除的过时文档

❌ **DEPLOYMENT.old.md** - 旧版部署文档(已被新文档替代)
❌ **PERFORMANCE.md** - 性能文档(内容过时,需要重写)
❌ **INDEX.md** - 重复索引(README已包含)
❌ **FINAL_STATUS.md** - 临时状态文档

#### 归档的历史报告

移动到 `docs/reports/archive/`:

📦 **HEXAGONAL_ARCHITECTURE_COMPLETION_REPORT.md** - 六边形架构完成报告
📦 **HEXAGONAL_REFACTORING_FINAL_REPORT.md** - 重构最终报告
📦 **PHASE2_COMPLETION_REPORT.md** - Phase 2完成报告
📦 **PHASE3_COMPLETION_REPORT.md** - Phase 3完成报告
📦 **PHASE4_EXECUTION_PLAN.md** - Phase 4执行计划
📦 **PHASE4_WEEK1_REPORT.md** - Phase 4 Week 1报告
📦 **CLICKHOUSE_25_DEPLOYMENT_REPORT.md** - ClickHouse部署报告
📦 **END_TO_END_TEST_REPORT.md** - 端到端测试报告
📦 **TEST_REPORT.md** - 测试报告
📦 **STAGE1_COMPLETION_REPORT.md** - Stage 1完成报告

**重组命令:**
```bash
# 创建归档目录
mkdir -p docs/reports/archive

# 移动历史报告
mv docs/HEXAGONAL_*_REPORT.md docs/reports/archive/
mv docs/PHASE*_COMPLETION_REPORT.md docs/reports/archive/
mv docs/PHASE4_EXECUTION_PLAN.md docs/reports/archive/
mv docs/CLICKHOUSE_25_DEPLOYMENT_REPORT.md docs/reports/archive/
mv docs/END_TO_END_TEST_REPORT.md docs/reports/archive/
mv docs/TEST_REPORT.md docs/reports/archive/
mv docs/STAGE1_COMPLETION_REPORT.md docs/reports/archive/

# 删除过时文档
rm docs/DEPLOYMENT.old.md
rm docs/PERFORMANCE.md
rm docs/INDEX.md
rm docs/FINAL_STATUS.md
```

### 2.3 新增核心文档

#### 1. 重写 **docs/ARCHITECTURE.md**

**内容结构:**
```markdown
# 系统架构文档

## 六边形架构设计
- 架构原则
- 依赖倒置
- 端口和适配器模式

## 架构层次
- Domain Layer(领域层)
- Application Layer(应用层)
- Adapter Layer(适配器层)

## 服务边界和职责
- 各服务的领域边界
- 服务间通信方式
- 数据流向

## 领域模型设计
- 核心实体说明
- 值对象设计
- 领域服务

## 技术选型
- 后端技术栈
- 数据库选型
- 消息队列
```

#### 2. 新建 **docs/HEXAGONAL_GUIDE.md**

**内容结构:**
```markdown
# 六边形架构开发指南

## 架构原则
- SOLID原则应用
- DDD实践
- 依赖注入模式

## 服务开发步骤
1. 领域建模
2. 定义端口
3. 实现应用层
4. 实现适配器
5. 编写测试

## 代码模板
- domain crate模板
- service结构模板
- 测试模板

## 最佳实践
- 错误处理规范
- 日志记录规范
- 测试覆盖率要求
- 代码审查清单

## 常见问题
- Q&A
- 示例代码
```

#### 3. 新建 **docs/DEPLOYMENT_GUIDE.md**

**内容结构:**
```markdown
# 部署指南

## 环境准备
- 系统要求
- 依赖安装
- 配置检查

## 部署方式
- 方式一: 一键启动(推荐新手)
- 方式二: 分步部署(推荐开发者)
- 方式三: 开发模式

## 配置说明
- 环境变量
- docker-compose.yml
- 服务配置

## 健康检查
- 检查脚本使用
- 监控指标
- 告警配置

## 故障排查
- 常见问题
- 日志分析
- 性能调优

## 生产部署
- 蓝绿部署
- 灰度发布
- 回滚策略
```

---

## 🚀 第三部分:部署脚本统一

### 3.1 脚本整合

#### 保留的核心脚本

✅ **start-all.sh** - 一键启动所有服务
✅ **stop-all.sh** - 停止所有服务
✅ **health-check.sh** - 健康检查
✅ **check-env.sh** - 环境检查
✅ **deploy.sh** - 多模式部署

#### 删除的冗余脚本

❌ **reset-all.sh** - 功能重复,已集成到 deploy.sh
❌ **scripts/start_hexagonal.sh** - 功能合并到 start-all.sh
❌ **scripts/stop_hexagonal.sh** - 功能合并到 stop-all.sh
❌ **scripts/monitor_hexagonal.sh** - 功能合并到 health-check.sh

**清理命令:**
```bash
rm -f reset-all.sh
rm -f scripts/start_hexagonal.sh
rm -f scripts/stop_hexagonal.sh
rm -f scripts/monitor_hexagonal.sh
```

### 3.2 脚本更新要点

#### start-all.sh 更新

**改进点:**
1. 支持六边形架构服务启动
2. 按依赖顺序启动服务
3. 彩色输出和进度显示
4. 自动等待数据库就绪
5. 自动初始化数据库
6. 完成后自动执行健康检查

#### health-check.sh 更新

**改进点:**
1. 检查所有服务的HTTP健康端点
2. 检查进程状态
3. 彩色输出检查结果
4. 失败时显示详细错误信息
5. 支持单个服务检查

---

## 📅 第四部分:详细实施步骤

### 阶段一:项目清理(1天)

**目标:** 清理冗余文档和文件,为重构扫清障碍

#### 任务清单

- [ ] 1.1 备份当前项目状态
- [ ] 1.2 清理根目录临时文档
- [ ] 1.3 重组docs/目录,归档历史报告
- [ ] 1.4 删除过时文档
- [ ] 1.5 清理冗余脚本
- [ ] 1.6 验证项目仍可正常运行

**预期成果:**
- 根目录只保留 README.md 和 CHANGELOG.md
- docs/ 目录结构清晰
- 历史报告归档到 docs/reports/archive/
- 部署脚本统一为5个核心脚本

---

### 阶段二:六边形架构模板创建(0.5天)

**目标:** 创建可复用的服务开发模板

#### 任务清单

- [ ] 2.1 创建服务模板目录 `templates/hexagonal-service/`
- [ ] 2.2 创建完整服务模板(含domain)
- [ ] 2.3 创建简化服务模板(不含domain)
- [ ] 2.4 编写 `docs/HEXAGONAL_GUIDE.md` 开发指南
- [ ] 2.5 创建代码示例和最佳实践文档

**预期成果:**
- 开发者可以基于模板快速创建新服务
- 统一的代码结构和开发规范
- 完善的开发指南

---

### 阶段三:核心服务迁移(5-7天)

#### 3.1 storage-service 迁移(2天)

**Day 1: 领域层创建**

任务:
- [ ] 创建 `services/storage-service/domain/` crate
- [ ] 定义实体: QueryRequest, DataBatch, StorageMetrics
- [ ] 定义值对象: TimeRange, QueryCriteria, BatchConfig
- [ ] 定义领域服务: BatchWriter, QueryOptimizer, MetricsCollector
- [ ] 定义端口: StorageService(Primary), QuoteRepository(Secondary)
- [ ] 编写领域层单元测试

**Day 2: 应用层和适配器层**

任务:
- [ ] 创建应用层: StorageOrchestrator, QueryUseCase, BatchWriteUseCase
- [ ] 实现适配器: HttpController, ClickHouseRepository, RedisAdapter
- [ ] 重构 main.rs 使用六边形架构
- [ ] 创建 service.rs 服务封装
- [ ] 编写集成测试
- [ ] 性能测试对比

**验收标准:**
- ✅ 编译0错误0警告
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试100%通过
- ✅ API响应时间 < 100ms
- ✅ 数据写入成功率100%

---

#### 3.2 auction-storage 迁移(2天)

**Day 1: 领域层创建**

任务:
- [ ] 创建 `services/auction-storage/domain/` crate
- [ ] 定义实体: AuctionQuote, AlertRule, WatchlistItem, AlertEvent
- [ ] 定义值对象: AuctionTime, AlertCondition, StrengthScore, SealedAmount
- [ ] 定义领域服务: AlertEvaluator, RankingCalculator, StrengthScorer
- [ ] 定义端口: AuctionStorageService, AuctionRepository, AlertRepository
- [ ] 编写单元测试

**Day 2: 应用层和适配器层**

任务:
- [ ] 创建应用层和用例
- [ ] 实现适配器
- [ ] 重构入口点
- [ ] 编写集成测试

**验收标准:**
- ✅ 编译0错误0警告
- ✅ 单元测试覆盖率 > 80%
- ✅ 告警功能测试通过
- ✅ 排行榜性能 < 200ms

---

#### 3.3 auction-service 迁移(1.5天)

任务:
- [ ] 创建domain crate
- [ ] 定义领域模型
- [ ] 实现应用层和适配器
- [ ] 与auction-storage集成测试

**验收标准:**
- ✅ 竞价数据采集成功率100%
- ✅ 强度评分算法准确

---

#### 3.4 backtest-service 迁移(1.5天)

任务:
- [ ] 创建domain crate
- [ ] 定义回测领域模型
- [ ] 实现回测引擎
- [ ] 性能测试

**验收标准:**
- ✅ 回测逻辑正确
- ✅ 性能指标准确

---

### 阶段四:简单服务处理(1-2天)

#### 4.1 realtime-service 简化重构(0.5天)

任务:
- [ ] 清理代码结构
- [ ] 统一错误处理
- [ ] 优化连接管理
- [ ] 补充健康检查
- [ ] 添加集成测试

---

#### 4.2 auth-service 简化重构(0.5天)

任务:
- [ ] 保留现有JWT架构
- [ ] 统一错误处理
- [ ] 优化配置管理
- [ ] 补充API文档
- [ ] 添加单元测试

---

#### 4.3 auction-realtime 简化重构(0.5天)

任务:
- [ ] 类似realtime-service重构
- [ ] 集成测试

---

#### 4.4 data-collector 验证和优化(0.5天)

任务:
- [ ] 对齐新服务模板结构
- [ ] 统一错误处理模式
- [ ] 补充集成测试
- [ ] 性能验证

---

### 阶段五:部署文档和脚本更新(0.5天)

任务:
- [ ] 创建 `docs/DEPLOYMENT_GUIDE.md`
- [ ] 重写 `docs/ARCHITECTURE.md`
- [ ] 更新 `start-all.sh` 支持六边形架构
- [ ] 更新 `health-check.sh` 检查所有服务
- [ ] 创建集成测试脚本 `test-all-services.sh`

---

### 阶段六:最终验证和文档完善(0.5天)

任务:
- [ ] 运行完整集成测试
- [ ] 性能基准测试
- [ ] 更新 README.md 架构说明
- [ ] 创建迁移完成报告
- [ ] 验证部署流程
- [ ] 代码审查和优化

---

## 🎯 第五部分:风险控制和质量保证

### 5.1 风险识别和应对

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| 数据丢失 | 高 | 低 | 完整备份,灰度发布 |
| 性能下降 | 中 | 低 | 性能基准测试,对比验证 |
| 服务中断 | 中 | 中 | 蓝绿部署,快速回滚 |
| 集成问题 | 中 | 中 | 充分集成测试,Mock验证 |
| 编译错误 | 低 | 中 | 增量编译,及时修复 |

### 5.2 回滚策略

**如果迁移出现问题:**

```bash
# 1. 快速回滚到上一个稳定版本
git checkout main
git checkout -b rollback-$(date +%Y%m%d)
git revert <migration-commits>

# 2. 重新部署
bash deploy.sh quick

# 3. 验证服务
bash health-check.sh
```

### 5.3 质量门禁

**每个服务迁移完成后必须通过:**

- ✅ **编译检查**: 0错误, 0警告
- ✅ **单元测试**: 覆盖率 > 80%
- ✅ **集成测试**: 100%通过
- ✅ **性能测试**: 不低于迁移前基准
- ✅ **代码审查**: 架构合规性检查
- ✅ **文档更新**: API文档和架构文档更新

---

## 📊 第六部分:时间线和里程碑

### 总时间估算: 8-10天

```
Week 1:
├── Day 1: 项目清理 + 模板创建
├── Day 2-3: storage-service迁移
├── Day 4-5: auction-storage迁移
└── Day 6: auction-service迁移

Week 2:
├── Day 1: backtest-service迁移
├── Day 2: 简单服务重构
├── Day 3: 部署脚本和文档更新
├── Day 4: 集成测试和验证
└── Day 5: 文档完善和总结
```

### 关键里程碑

- ✅ **Milestone 1 (Day 1)**: 项目清理完成,模板就绪
- ✅ **Milestone 2 (Day 3)**: 第一个复杂服务(storage-service)迁移完成
- ✅ **Milestone 3 (Day 6)**: 所有复杂服务迁移完成
- ✅ **Milestone 4 (Day 8)**: 所有服务迁移完成
- ✅ **Milestone 5 (Day 10)**: 测试通过,文档更新,上线就绪

---

## 🎯 第七部分:预期成果

### 7.1 清晰的项目结构

**根目录:**
```
duanxianxia/
├── README.md                    # 项目主文档
├── CHANGELOG.md                 # 变更日志
├── Cargo.toml                   # Workspace配置
├── docker-compose.yml           # Docker编排
├── start-all.sh                 # 一键启动
├── stop-all.sh                  # 一键停止
├── health-check.sh              # 健康检查
├── check-env.sh                 # 环境检查
└── deploy.sh                    # 部署脚本
```

**docs/ 目录:**
```
docs/
├── QUICK_START.md               # 快速开始
├── USER_GUIDE.md                # 用户指南
├── TROUBLESHOOTING.md           # 故障排查
├── ARCHITECTURE.md              # 架构文档(重写)
├── HEXAGONAL_GUIDE.md           # 开发指南(新建)
├── DEPLOYMENT_GUIDE.md          # 部署指南(新建)
├── deployment-index.md          # 部署索引
├── deployment/                  # 部署详细文档
│   └── DEPLOYMENT.md
└── reports/                     # 报告目录
    ├── archive/                 # 历史报告归档
    │   ├── HEXAGONAL_*.md
    │   ├── PHASE*.md
    │   └── ...
    └── 2026-01-09-*.md          # 最新报告
```

### 7.2 统一的架构风格

**所有服务采用:**
- ✅ 六边形架构设计
- ✅ 清晰的层次划分(Domain/Application/Adapter)
- ✅ 依赖倒置原则
- ✅ 高度可测试的代码
- ✅ 统一的错误处理
- ✅ 结构化日志

### 7.3 完善的文档体系

- ✅ **架构设计文档**: ARCHITECTURE.md
- ✅ **开发指南**: HEXAGONAL_GUIDE.md
- ✅ **部署手册**: DEPLOYMENT_GUIDE.md
- ✅ **快速开始**: QUICK_START.md
- ✅ **用户手册**: USER_GUIDE.md
- ✅ **故障排查**: TROUBLESHOOTING.md

### 7.4 生产就绪

- ✅ **代码质量**: 0编译警告,高测试覆盖率
- ✅ **性能**: 保持优秀(< 100ms响应时间)
- ✅ **监控**: 健全的日志和监控
- ✅ **部署**: 快速一键部署
- ✅ **文档**: 完善的运维文档

---

## 📝 第八部分:后续维护建议

### 8.1 短期(1-2周)

1. **生产环境切换**
   - 监控新架构服务运行状态
   - 收集性能指标
   - 快速修复发现的问题

2. **持续优化**
   - 根据监控数据优化配置
   - 优化资源使用
   - 改进错误处理

### 8.2 中期(1-2个月)

1. **功能扩展**
   - 实现完整的事件发布机制
   - 添加更多数据源支持
   - 实现CQRS模式

2. **性能优化**
   - 批量处理优化
   - 连接池优化
   - 缓存策略优化

### 8.3 长期(3-6个月)

1. **架构演进**
   - 引入事件溯源
   - 实现读写分离
   - 微服务拆分

2. **智能化**
   - 机器学习预测
   - 智能调度
   - 自动扩缩容

---

## 📞 附录

### A. 参考文档

- **六边形架构**: docs/plans/HEXAGONAL_REFACTORING_GUIDE.md
- **现有架构**: docs/HEXAGONAL_REFACTORING_FINAL_REPORT.md
- **服务文档**: 各服务目录下的 README.md

### B. 联系方式

如有问题,请参考:
- 故障排查: docs/TROUBLESHOOTING.md
- 开发指南: docs/HEXAGONAL_GUIDE.md
- 部署指南: docs/DEPLOYMENT_GUIDE.md

---

**文档状态**: ✅ 已完成,待审批
**下一步**: 开始实施阶段一(项目清理)
