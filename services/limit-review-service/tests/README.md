# 涨停复盘系统 - 测试文档

**创建日期:** 2026-01-13
**版本:** v1.0-beta

---

## 📋 测试概述

本系统包含3个层次的测试:
1. **单元测试** - 测试独立函数和模块
2. **集成测试** - 测试模块间交互
3. **性能测试** - 验证性能指标

---

## 🧪 单元测试

### 运行所有单元测试

```bash
cd services/limit-review-service
cargo test
```

### 运行特定测试

```bash
# 只测试涨停识别器
cargo test limit_detector

# 只测试连板计算器
cargo test consecutive_calculator

# 运行特定测试用例
cargo test test_calculate_limit_price_normal_stock

# 显示输出
cargo test -- --nocapture

# 显示详细日志
RUST_LOG=debug cargo test
```

### 单元测试覆盖

#### LimitDetector (涨停识别器)

| 测试用例 | 测试内容 | 状态 |
|---------|---------|------|
| `test_calculate_limit_price_normal_stock` | 普通股票10%涨停价 | ✅ |
| `test_calculate_limit_price_gem_stock` | 创业板20%涨停价 | ✅ |
| `test_calculate_limit_price_star_stock` | 科创板20%涨停价 | ✅ |
| `test_calculate_limit_price_st_stock` | ST股票5%涨停价 | ✅ |
| `test_is_limit_up_true` | 判断涨停(正确) | ✅ |
| `test_is_limit_up_false` | 判断涨停(错误) | ✅ |
| `test_is_limit_up_with_tolerance` | 1分钱误差容忍 | ✅ |
| `test_classify_straight_board` | 一字板分类 | ✅ |
| `test_classify_t_shape_board` | T字板分类 | ✅ |
| `test_classify_natural_board` | 换手板分类 | ✅ |
| `test_classify_broken_board` | 炸板分类 | ✅ |
| `test_count_open_times_zero` | 开板次数=0 | ✅ |
| `test_count_open_times_once` | 开板次数=1 | ✅ |
| `test_count_open_times_multiple` | 开板次数>1 | ✅ |
| `test_detect_seal_timings_first_seal` | 首次封板时间 | ✅ |
| `test_detect_seal_timings_final_seal` | 最终封板时间 | ✅ |
| `test_detect_seal_timings_broken_time` | 炸板时间 | ✅ |
| `test_calculate_sealed_amount` | 封单金额计算 | ✅ |
| `test_limit_type_display_name` | 板类型显示名称 | ✅ |
| `test_limit_type_from_str` | 板类型字符串解析 | ✅ |
| `test_limit_type_as_str` | 板类型转字符串 | ✅ |

**总计:** 22个测试用例

#### ConsecutiveCalculator (连板计算器)

| 测试用例 | 测试内容 | 状态 |
|---------|---------|------|
| `test_calculate_consecutive_from_history_no_limit` | 未涨停连板数 | ✅ |
| `test_calculate_consecutive_from_history_first_board` | 首板连板数 | ✅ |
| `test_calculate_consecutive_from_history_2_consecutive` | 2连板 | ✅ |
| `test_calculate_consecutive_from_history_5_consecutive` | 5连板 | ✅ |
| `test_calculate_consecutive_from_history_broken` | 断板后重新开始 | ✅ |
| `test_is_new_high_from_history_true` | 判断新高(是) | ✅ |
| `test_is_new_high_from_history_false` | 判断新高(否) | ✅ |
| `test_is_new_high_from_history_equal` | 等于历史最高 | ✅ |
| `test_is_new_high_from_history_no_history` | 无历史数据 | ✅ |
| `test_prev_trading_day_weekday` | 前一交易日(工作日) | ✅ |
| `test_prev_trading_day_monday` | 前一交易日(周一) | ✅ |
| `test_prev_trading_day_saturday` | 前一交易日(周六) | ✅ |
| `test_calculate_consecutive_max_30_days` | 最多30日连板 | ✅ |
| `test_calculate_consecutive_empty_history` | 空历史记录 | ✅ |
| `test_calculate_consecutive_different_stock` | 不同股票独立计算 | ✅ |

**总计:** 15个测试用例

---

## 🔗 集成测试

### 前置条件

集成测试需要ClickHouse数据库运行:

```bash
# 启动ClickHouse
docker-compose up -d clickhouse

# 验证运行
docker ps | grep clickhouse
```

### 初始化测试数据

```bash
# 创建表结构
docker exec -i $(docker ps -q -f name=clickhouse) \
  clickhouse-client < db/limit_review_schema.sql

# 插入测试数据
docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query="
INSERT INTO duanxianxia.stock_realtime_quotes FORMAT CSVWithNames
" < tests/test_data.csv
```

### 运行集成测试

```bash
# 运行所有集成测试
cargo test --test integration_test

# 运行特定集成测试
cargo test --test integration_test test_end_to_end_daily_review

# 运行被忽略的测试
cargo test -- --ignored
```

### 集成测试覆盖

| 测试用例 | 测试内容 | 需求 |
|---------|---------|------|
| `test_end_to_end_daily_review` | 端到端复盘生成 | ClickHouse |
| `test_clickhouse_connection` | 数据库连接 | ClickHouse |
| `test_load_day_quotes` | 数据加载 | ClickHouse |
| `test_api_get_daily_review` | API端点 | 服务运行 |
| `test_performance_1000_stocks` | 性能测试 | ClickHouse + 数据 |

**注意:** 集成测试默认使用`#[ignore]`标记,需要显式启用。

---

## ⚡ 性能测试

### 性能指标

| 指标 | 目标值 | 测试方法 |
|------|--------|---------|
| 单日1000只股票处理时间 | < 10秒 | `test_performance_1000_stocks` |
| API响应时间(P95) | < 100ms | 压力测试 |
| 内存占用 | < 500MB | 监控工具 |
| 并发处理能力 | 50股票并行 | 并发测试 |

### 运行性能测试

```bash
# Release模式运行
cargo test --release --test integration_test test_performance

# 使用 Criterion 基准测试
cargo bench
```

---

## 📊 测试覆盖率

### 生成覆盖率报告

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open html/index.html
```

### 目标覆盖率

| 模块 | 目标覆盖率 | 当前状态 |
|------|-----------|---------|
| `limit_detector` | 90% | 待测试 |
| `consecutive_calculator` | 85% | 待测试 |
| `data_loader` | 70% | 待测试 |
| `review_generator` | 75% | 待测试 |
| **总体** | **80%** | **待测试** |

---

## 🐛 调试测试

### 启用日志输出

```bash
# 显示标准输出
cargo test -- --nocapture

# 启用调试日志
RUST_LOG=debug cargo test -- --nocapture

# 启用跟踪日志
RUST_LOG=trace cargo test -- --nocapture
```

### 单步调试

```bash
# 使用 lldb
cargo test -- --nocapture
lldb target/debug/deps/xxx-xxx

# 使用 gdb
cargo test -- --nocapture
gdb target/debug/deps/xxx-xxx
```

---

## ✅ 测试检查清单

在提交代码前,请确保:

- [ ] 所有单元测试通过 (`cargo test`)
- [ ] 代码编译无警告 (`cargo clippy`)
- [ ] 代码格式化 (`cargo fmt --check`)
- [ ] 集成测试在本地通过
- [ ] 性能测试满足指标
- [ ] 文档更新

---

## 📝 添加新测试

### 测试命名规范

```rust
#[test]
fn test_<功能>_<场景>_<期望>() {
    // 示例: test_calculate_limit_price_normal_stock
}
```

### 测试结构

```rust
#[tokio::test] // 异步测试
async fn test_async_function() {
    // Arrange (准备)
    let input = create_test_input();

    // Act (执行)
    let result = function_under_test(input).await.unwrap();

    // Assert (断言)
    assert_eq!(result, expected);
}
```

### Mock外部依赖

```rust
use mockall::mock;

mock! {
    pub DataLoader {}

    impl DataLoader {
        pub async fn load_day_quotes(&self, date: Date) -> Result<Vec<StockQuote>>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockDataLoader::new();
    mock.expect_load_day_quotes()
        .returning(|| Ok(vec![]));

    // 测试代码
}
```

---

## 🔗 相关文档

- [技术方案](../docs/plans/2026-01-13-limit-review-system-design.md)
- [实施指南](../docs/plans/2026-01-13-limit-review-implementation-guide.md)
- [问题修复报告](../docs/plans/2026-01-13-limit-review-bugfix-report.md)

---

## 📞 支持

**测试问题:**
- 查看测试日志
- 检查ClickHouse连接
- 验证测试数据

**性能问题:**
- 使用Release模式
- 检查资源占用
- 分析火焰图

---

**版本:** v1.0-beta
**状态:** 单元测试已完成,集成测试待验证
