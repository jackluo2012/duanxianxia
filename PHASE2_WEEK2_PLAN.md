# Phase 2 Week 2 实施计划 - 数据挖掘模块

**日期：** 2026-01-01
**状态：** 准备开始
**前置条件：** Phase 2 Week 1 已完成 (21/21 tasks) ✅

---

## 📊 Week 2 目标

实现数据挖掘模块，包括个股挖掘、概念检索和技术指标计算功能。

---

## ✅ 任务分解

### Day 1: 个股挖掘算法 (2026-01-02)

#### Task 1.1: 扩展 query-service 服务
- [ ] 创建 `services/query-service` 目录结构
- [ ] 设置 Cargo.toml 依赖（actix-web, clickhouse, serde等）
- [ ] 创建基础框架（main.rs, lib.rs）
- [ ] 集成 ClickHouse 客户端

#### Task 1.2: 实现个股挖掘算法
- [ ] 创建 `screener.rs` 模块
- [ ] 实现龙头高度计算算法
  - 计算同行业股票数量
  - 计算个股在行业中的排名
  - 龙头高度 = 排名倒数
- [ ] 实现连板天统计
  - 查询历史涨停数据
  - 计算连续涨停天数
  - 记录连板起始和结束日期
- [ ] 实现涨停跌停分析
  - 识别涨停/跌停股票
  - 统计涨停时间和次数
  - 分析涨停原因（竞价强势、板块热点等）

#### Task 1.3: HTTP API 开发
- [ ] `GET /api/screener/leaders` - 龙头高度排行
- [ ] `GET /api/screener/consecutive` - 连板股票列表
- [ ] `GET /api/screener/limit-up` - 涨停股票列表
- [ ] `GET /api/screener/limit-down` - 跌停股票列表

**预期产出：**
- query-service 服务基础框架
- 个股挖掘核心算法（300+行）
- 4个 REST API 端点
- 单元测试覆盖率 > 80%

---

### Day 2: 概念检索功能 (2026-01-03)

#### Task 2.1: 板块数据模型设计
- [ ] 设计 ClickHouse 板块表结构
  ```sql
  CREATE TABLE sector_stocks (
      date Date,
      sector_code String,
      sector_name String,
      stock_code String,
      stock_name String
  ) ENGINE = MergeTree()
  ORDER BY (sector_code, stock_code);
  ```
- [ ] 设计板块表现表
  ```sql
  CREATE TABLE sector_performance (
      date Date,
      sector_code String,
      sector_name String,
      avg_change_percent Float64,
      total_volume Float64,
      total_amount Float64,
      stock_count UInt32
  ) ENGINE = MergeTree()
  ORDER BY (date, avg_change_percent DESC);
  ```

#### Task 2.2: 实现概念检索模块
- [ ] 创建 `sectors.rs` 模块
- [ ] 实现板块-股票关联查询
- [ ] 实现板块表现统计
  - 平均涨跌幅计算
  - 总成交额/成交量计算
  - 板块内股票数量统计
- [ ] 实现资金流向计算
  - 板块资金流入流出
  - 主力资金净流入计算

#### Task 2.3: HTTP API 开发
- [ ] `GET /api/sectors` - 所有板块列表
- [ ] `GET /api/sectors/{code}/stocks` - 板块内股票列表
- [ ] `GET /api/sectors/performance` - 板块表现排行
- [ ] `GET /api/sectors/{code}/flow` - 板块资金流向

**预期产出：**
- 2个 ClickHouse 新表
- 概念检索模块（250+行）
- 4个 REST API 端点
- 板块数据初始化脚本

---

### Day 3: 技术指标计算 (2026-01-04)

#### Task 3.1: 技术指标算法实现
- [ ] 创建 `indicators.rs` 模块
- [ ] 实现移动平均线（MA）算法
  - MA5, MA10, MA20, MA60
  - EMA（指数移动平均）
- [ ] 实现 MACD 算法
  - DIF = EMA12 - EMA26
  - DEA = EMA(DIF, 9)
  - MACD = 2 × (DIF - DEA)
- [ ] 实现 KDJ 算法
  - RSV = (收盘价 - 最低价) / (最高价 - 最低价) × 100
  - K = 2/3 × 前一日K + 1/3 × RSV
  - D = 2/3 × 前一日D + 1/3 × K
  - J = 3K - 2D
- [ ] 实现 RSI 算法
  - 相对强弱指标计算
  - RSI6, RSI12, RSI24

#### Task 3.2: ClickHouse 物化视图
- [ ] 创建技术指标表
  ```sql
  CREATE TABLE stock_indicators (
      date Date,
      code String,
      ma5 Float64,
      ma10 Float64,
      ma20 Float64,
      ma60 Float64,
      macd Float64,
      dif Float64,
      dea Float64,
      kdj_k Float64,
      kdj_d Float64,
      kdj_j Float64,
      rsi6 Float64,
      rsi12 Float64,
      rsi24 Float64
  ) ENGINE = MergeTree()
  ORDER BY (code, date);
  ```
- [ ] 创建定时更新任务
  - 每日收盘后计算所有股票技术指标
  - 增量更新当日指标

#### Task 3.3: HTTP API 开发
- [ ] `GET /api/indicators/{code}` - 获取股票技术指标
- [ ] `GET /api/indicators/{code}/history` - 历史指标数据
- [ ] `POST /api/indicators/calculate` - 触发指标计算

**预期产出：**
- 技术指标算法模块（400+行）
- ClickHouse 技术指标表
- 定时计算任务
- 3个 REST API 端点
- 单元测试覆盖率 > 85%

---

### Day 4: 前端挖掘页面 (2026-01-05)

#### Task 4.1: 个股挖掘页面
- [ ] 创建 `frontend/src/pages/ScreenerDashboard.tsx`
- [ ] 实现 Tab 布局
  - Tab 1: 龙头高度排行
  - Tab 2: 连板股票列表
  - Tab 3: 涨停股票列表
  - Tab 4: 跌停股票列表
- [ ] 实现数据表格展示
  - Ant Design Table 组件
  - 排序和筛选功能
  - 分页加载
- [ ] 实现图表可视化
  - 龙头高度分布柱状图
  - 连板天数分布图

#### Task 4.2: 概念板块页面
- [ ] 创建 `frontend/src/pages/SectorDashboard.tsx`
- [ ] 实现板块列表
  - 板块名称、涨跌幅、成交额
  - 排序和筛选
- [ ] 实现板块详情
  - 板块内股票列表
  - 板块资金流向图
  - 板块走势图

#### Task 4.3: 技术指标页面
- [ ] 创建 `frontend/src/pages/IndicatorDashboard.tsx`
- [ ] 实现指标查询表单
  - 股票代码选择器
  - 指标类型选择（MA/MACD/KDJ/RSI）
  - 日期范围选择
- [ ] 实现 ECharts 可视化
  - MA 指标：价格 + MA 线条
  - MACD：DIF、DEA、MACD 柱状图
  - KDJ：K、D、J 三线图
  - RSI：RSI 曲线图

#### Task 4.4: API 客户端开发
- [ ] 创建 `frontend/src/api/screener.ts`
- [ ] 创建 `frontend/src/api/sectors.ts`
- [ ] 创建 `frontend/src/api/indicators.ts`
- [ ] 集成到路由配置

**预期产出：**
- 3个前端页面（每个150+行）
- 3个 API 客户端模块
- ECharts 图表集成
- 路由配置更新

---

### Day 5: 测试和优化 (2026-01-06)

#### Task 5.1: 单元测试
- [ ] query-service 单元测试
  - 挖掘算法测试
  - 板块统计测试
  - 技术指标计算测试
- [ ] 前端组件测试
  - 页面渲染测试
  - API 交互测试

#### Task 5.2: 集成测试
- [ ] 创建 `test-screener-api.sh`
  - 龙头排行 API 测试
  - 连板股票 API 测试
  - 涨停跌停 API 测试
- [ ] 创建 `test-sectors-api.sh`
  - 板块列表 API 测试
  - 板块股票查询测试
  - 板块表现统计测试
- [ ] 创建 `test-indicators-api.sh`
  - 技术指标计算测试
  - 历史数据查询测试

#### Task 5.3: 性能优化
- [ ] ClickHouse 查询优化
  - 添加必要索引
  - 优化 JOIN 查询
  - 实现查询结果缓存
- [ ] 前端性能优化
  - 实现虚拟滚动（react-window）
  - 数据分页加载
  - 图表数据采样
- [ ] API 响应优化
  - 启用 Redis 缓存
  - 批量查询接口
  - 响应数据压缩

#### Task 5.4: 文档更新
- [ ] 更新主 README
  - 新增挖掘模块说明
  - 新增板块检索说明
  - 新增技术指标说明
  - 更新 API 端点文档
- [ ] 创建 `DAY6_SUMMARY.md`
- [ ] 创建 `docs/data-mining-architecture.md`
  - 挖掘算法说明
  - 数据流转图
  - 性能优化方案

**预期产出：**
- 单元测试覆盖率 > 80%
- 集成测试通过率 > 90%
- 性能基准测试报告
- 完整文档更新

---

## 📈 成功标准

### 功能完整性
- ✅ 个股挖掘功能：龙头高度、连板、涨跌停
- ✅ 概念检索功能：板块列表、板块股票、板块表现
- ✅ 技术指标功能：MA、MACD、KDJ、RSI
- ✅ 前端页面：3个新页面，完整展示

### 性能指标
- ✅ API 响应时间 < 200ms (P95)
- ✅ 前端页面加载 < 2秒
- ✅ 技术指标计算 < 5秒（全市场）

### 数据准确性
- ✅ 挖掘算法验证正确
- ✅ 技术指标与主流软件对比一致
- ✅ 板块统计数据准确

---

## 🛠️ 技术架构

### 新增服务
```
query-service  - 查询和挖掘服务
├── src/
│   ├── main.rs           - 服务入口
│   ├── screener.rs       - 个股挖掘模块
│   ├── sectors.rs        - 概念板块模块
│   ├── indicators.rs     - 技术指标模块
│   └── api.rs            - REST API
```

### 数据库扩展
```sql
-- 板块股票关联表
CREATE TABLE sector_stocks (...);

-- 板块表现表
CREATE TABLE sector_performance (...);

-- 技术指标表
CREATE TABLE stock_indicators (...);
```

### 前端页面
```
frontend/src/pages/
├── ScreenerDashboard.tsx   - 个股挖掘页面
├── SectorDashboard.tsx     - 概念板块页面
└── IndicatorDashboard.tsx  - 技术指标页面

frontend/src/api/
├── screener.ts            - 挖掘 API 客户端
├── sectors.ts             - 板块 API 客户端
└── indicators.ts          - 指标 API 客户端
```

---

## ⏱️ 预估工时

| 任务 | 预估工时 | 优先级 |
|------|----------|--------|
| Day 1: 个股挖掘 | 6小时 | P0 |
| Day 2: 概念检索 | 6小时 | P0 |
| Day 3: 技术指标 | 7小时 | P1 |
| Day 4: 前端页面 | 8小时 | P0 |
| Day 5: 测试优化 | 6小时 | P1 |
| **总计** | **33小时** | - |

---

## 🚀 实施步骤

1. **开始 Day 1 任务**：创建 query-service 服务框架
2. **持续开发**：按计划逐天完成模块
3. **每日提交**：每天结束后 git commit
4. **测试驱动**：先写测试，再实现功能
5. **文档同步**：代码与文档同步更新

---

**创建日期：** 2026-01-01
**预计开始日期：** 2026-01-02
**预计完成日期：** 2026-01-06
**Phase 2 总进度：** Week 1 ✅ | Week 2 ⏳ | Week 3 ⏳ | Week 4 ⏳
