#!/bin/bash
# ===================================================================
# 涨停复盘系统 - 快速测试脚本
# ===================================================================

set -e

echo "🧪 涨停复盘系统 - 快速测试"
echo "================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 切换到服务目录
cd services/limit-review-service

echo "📂 当前目录: $(pwd)"
echo ""

# 1. 检查代码格式
echo "1️⃣  检查代码格式..."
if cargo fmt --check > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 代码格式正确${NC}"
else
    echo -e "${YELLOW}⚠️  代码格式不标准,运行 'cargo fmt' 修复${NC}"
    cargo fmt
fi
echo ""

# 2. 运行 Clippy
echo "2️⃣  运行 Clippy 检查..."
if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Clippy 检查通过${NC}"
else
    echo -e "${YELLOW}⚠️  Clippy 发现警告,请查看输出${NC}"
    cargo clippy --all-targets --all-features -- -D warnings
fi
echo ""

# 3. 编译检查
echo "3️⃣  编译检查..."
if cargo check > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 编译成功${NC}"
else
    echo -e "${RED}❌ 编译失败${NC}"
    cargo check
    exit 1
fi
echo ""

# 4. 运行单元测试
echo "4️⃣  运行单元测试..."
if cargo test --lib > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 单元测试通过${NC}"
else
    echo -e "${YELLOW}⚠️  单元测试失败,运行详细测试...${NC}"
    cargo test --lib -- --nocapture
fi
echo ""

# 5. 测试统计
echo "5️⃣  测试统计..."
TEST_COUNT=$(cargo test --lib --no-run 2>&1 | grep -o "[0-9]* tests" | head -1 || echo "未知")
echo -e "测试数量: ${GREEN}$TEST_COUNT${NC}"
echo ""

# 6. 检查ClickHouse
echo "6️⃣  检查ClickHouse状态..."
if docker ps | grep -q clickhouse; then
    echo -e "${GREEN}✅ ClickHouse 正在运行${NC}"
    echo ""
    echo "   📝 运行集成测试:"
    echo "   cargo test --test integration_test"
else
    echo -e "${YELLOW}⚠️  ClickHouse 未运行${NC}"
    echo ""
    echo "   📝 启动ClickHouse:"
    echo "   docker-compose up -d clickhouse"
fi
echo ""

# 7. 生成文档
echo "7️⃣  生成文档..."
cargo doc --no-deps > /dev/null 2>&1
echo -e "${GREEN}✅ 文档已生成${NC}"
echo ""

# 总结
echo "================================"
echo -e "${GREEN}✅ 快速测试完成!${NC}"
echo ""
echo "📊 测试文件:"
echo "   - src/limit_detector_tests.rs (22个测试)"
echo "   - src/consecutive_calculator_tests.rs (15个测试)"
echo "   - tests/integration_test.rs (5个集成测试)"
echo ""
echo "📝 文档:"
echo "   - tests/README.md (测试文档)"
echo "   - docs/plans/2026-01-13-limit-review-testing-summary.md"
echo ""
echo "🚀 下一步:"
echo "   1. 运行单元测试: cargo test"
echo "   2. 查看测试文档: cat tests/README.md"
echo "   3. 运行集成测试: 需要先启动ClickHouse"
