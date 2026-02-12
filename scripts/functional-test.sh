#!/bin/bash

###############################################################################
# 短线侠平台 - 功能完整性测试脚本
# 用途: 验证所有服务的核心功能是否正常工作
###############################################################################

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# 测试结果数组
declare -a FAILED_TEST_NAMES
declare -a FAILED_TEST_DETAILS

###############################################################################
# 工具函数
###############################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# 测试函数
test_api() {
    local test_name="$1"
    local url="$2"
    local expected_code="${3:-200}"
    local auth_header="$4"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log_info "测试: $test_name"

    local response
    if [ -n "$auth_header" ]; then
        response=$(curl -s -w "\n%{http_code}" -H "Authorization: $auth_header" "$url" 2>&1 || echo "000")
    else
        response=$(curl -s -w "\n%{http_code}" "$url" 2>&1 || echo "000")
    fi

    local body=$(echo "$response" | head -n -1)
    local http_code=$(echo "$response" | tail -n 1)

    if [ "$http_code" = "$expected_code" ]; then
        log_success "$test_name - 通过 (HTTP $http_code)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name - 失败 (期望: $expected_code, 实际: $http_code)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("$test_name")
        FAILED_TEST_DETAILS+=("URL: $url | HTTP $http_code | 响应: ${body:0:100}")
        return 1
    fi
}

test_websocket() {
    local test_name="$1"
    local url="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log_info "测试: $test_name"

    # 使用websocat或curl测试WebSocket连接
    local response
    if command -v websocat &> /dev/null; then
        response=$(timeout 3 websocat -n "$url" 2>&1 || echo "failed")
    else
        # 使用curl测试WebSocket握手
        response=$(curl -s -I \
            -H "Connection: Upgrade" \
            -H "Upgrade: websocket" \
            -H "Sec-WebSocket-Version: 13" \
            -H "Sec-WebSocket-Key: test" \
            "$url" 2>&1 | grep -i "101\|404" || echo "failed")
    fi

    if echo "$response" | grep -qi "101\|Upgrade"; then
        log_success "$test_name - 通过 (WebSocket可连接)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name - 失败 (WebSocket连接失败)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("$test_name")
        FAILED_TEST_DETAILS+=("URL: $url | 响应: ${response:0:100}")
        return 1
    fi
}

###############################################################################
# 主测试流程
###############################################################################

main() {
    print_header "短线侠平台 - 功能完整性测试"
    echo "测试时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""

    # 测试Token存储
    AUTH_TOKEN=""

    ###########################################################################
    # 1. 健康检查测试
    ###########################################################################
    print_header "1. 服务健康检查"

    test_api "认证服务健康检查" \
        "http://localhost:8082/api/health" \
        "200"

    test_api "查询服务健康检查" \
        "http://localhost:8089/health" \
        "200"

    test_api "涨停复盘服务健康检查" \
        "http://localhost:8088/health" \
        "200"

    test_api "竞价存储服务健康检查" \
        "http://localhost:8084/api/health" \
        "200"

    test_api "实时行情服务健康检查" \
        "http://localhost:8090/health" \
        "200"

    ###########################################################################
    # 2. 认证服务测试
    ###########################################################################
    print_header "2. 认证服务功能测试"

    # 测试用户注册
    local test_username="testuser_$(date +%s)"
    log_info "测试: 用户注册"
    local register_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_username\",\"email\":\"${test_username}@example.com\",\"password\":\"password123\"}" \
        "http://localhost:8082/api/auth/register" 2>&1 || echo '{"error":"failed"}')

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if echo "$register_response" | grep -q "token"; then
        log_success "用户注册 - 通过"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        log_error "用户注册 - 失败"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("用户注册")
        FAILED_TEST_DETAILS+=("响应: ${register_response:0:100}")
    fi

    # 测试用户登录
    log_info "测试: 用户登录"
    local login_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d '{"email":"test@example.com","password":"123456"}' \
        "http://localhost:8082/api/auth/login" 2>&1 || echo '{"error":"failed"}')

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if echo "$login_response" | grep -q "token"; then
        log_success "用户登录 - 通过"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        # 提取token用于后续测试
        AUTH_TOKEN=$(echo "$login_response" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
    else
        log_error "用户登录 - 失败"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        FAILED_TEST_NAMES+=("用户登录")
        FAILED_TEST_DETAILS+=("响应: ${login_response:0:100}")
    fi

    ###########################################################################
    # 3. 竞价存储服务测试
    ###########################################################################
    print_header "3. 竞价存储服务功能测试"

    # 测试竞价排行榜 - 买封
    test_api "竞价排行榜-买封" \
        "http://localhost:8084/api/auction/rankings?type=buy_seal&limit=10" \
        "200"

    # 测试竞价排行榜 - 强度
    test_api "竞价排行榜-强度" \
        "http://localhost:8084/api/auction/rankings?type=intensity&limit=10" \
        "200"

    # 测试竞价排行榜 - 涨幅
    test_api "竞价排行榜-涨幅" \
        "http://localhost:8084/api/auction/rankings?type=change_percent&limit=10" \
        "200"

    # 测试竞价详情查询
    test_api "竞价详情查询(000001)" \
        "http://localhost:8084/api/auction/details/000001" \
        "200"

    ###########################################################################
    # 4. 查询服务测试
    ###########################################################################
    print_header "4. 查询服务功能测试"

    # 测试龙头股票查询
    test_api "龙头股票查询" \
        "http://localhost:8089/api/screener/leaders?date=2025-01-16&limit=10" \
        "200"

    # 测试涨停股票查询
    test_api "涨停股票查询" \
        "http://localhost:8089/api/screener/limit-up?date=2025-01-16&limit=10" \
        "200"

    # 测试跌停股票查询
    test_api "跌停股票查询" \
        "http://localhost:8089/api/screener/limit-down?date=2025-01-16&limit=10" \
        "200"

    # 测试板块列表查询
    test_api "板块列表查询" \
        "http://localhost:8089/api/sectors/list" \
        "200"

    # 测试技术指标查询
    test_api "技术指标查询(000001)" \
        "http://localhost:8089/api/indicators/000001" \
        "200"

    # 测试MA指标查询
    test_api "MA指标查询(000001)" \
        "http://localhost:8089/api/indicators/000001/ma" \
        "200"

    # 测试MACD指标查询
    test_api "MACD指标查询(000001)" \
        "http://localhost:8089/api/indicators/000001/macd" \
        "200"

    # 测试KDJ指标查询
    test_api "KDJ指标查询(000001)" \
        "http://localhost:8089/api/indicators/000001/kdj" \
        "200"

    # 测试RSI指标查询
    test_api "RSI指标查询(000001)" \
        "http://localhost:8089/api/indicators/000001/rsi" \
        "200"

    # 测试每日复盘
    test_api "每日复盘查询" \
        "http://localhost:8089/api/review/daily?date=2025-01-16" \
        "200"

    # 测试板块复盘
    test_api "板块复盘查询" \
        "http://localhost:8089/api/review/sectors?date=2025-01-16" \
        "200"

    # 测试趋势复盘
    test_api "趋势复盘查询" \
        "http://localhost:8089/api/review/trend?date=2025-01-16" \
        "200"

    # 测试K线数据查询
    test_api "K线数据查询(000001)" \
        "http://localhost:8089/api/history/kline/000001?period=5m&limit=100" \
        "200"

    ###########################################################################
    # 5. 涨停复盘服务测试
    ###########################################################################
    print_header "5. 涨停复盘服务功能测试"

    # 测试每日涨停复盘
    test_api "每日涨停复盘" \
        "http://localhost:8088/api/review/2025-01-16" \
        "200"

    ###########################################################################
    # 6. WebSocket连接测试
    ###########################################################################
    print_header "6. WebSocket连接测试"

    test_websocket "实时行情WebSocket" \
        "ws://localhost:8090/ws/realtime"

    test_websocket "竞价实时WebSocket" \
        "ws://localhost:8081/ws"

    ###########################################################################
    # 测试报告
    ###########################################################################
    print_header "测试报告"

    echo -e "${BLUE}总测试数:${NC} $TOTAL_TESTS"
    echo -e "${GREEN}通过:${NC} $PASSED_TESTS"
    echo -e "${RED}失败:${NC} $FAILED_TESTS"
    echo -e "${YELLOW}跳过:${NC} $SKIPPED_TESTS"

    local pass_rate=0
    if [ $TOTAL_TESTS -gt 0 ]; then
        pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    fi
    echo -e "${BLUE}通过率:${NC} ${pass_rate}%"
    echo ""

    # 失败测试详情
    if [ $FAILED_TESTS -gt 0 ]; then
        print_header "失败测试详情"
        for i in "${!FAILED_TEST_NAMES[@]}"; do
            echo -e "${RED}✗${NC} ${FAILED_TEST_NAMES[$i]}"
            echo -e "   ${YELLOW}详情:${NC} ${FAILED_TEST_DETAILS[$i]}"
            echo ""
        done
    fi

    ###########################################################################
    # 保存测试报告
    ###########################################################################
    local report_file="docs/reports/functional-test-$(date +%Y-%m-%d).md"
    mkdir -p "$(dirname "$report_file")"

    cat > "$report_file" << EOF
# 功能完整性测试报告

**测试时间**: $(date '+%Y-%m-%d %H:%M:%S')
**测试环境**: 本地开发环境

## 测试统计

| 指标 | 数值 |
|------|------|
| 总测试数 | $TOTAL_TESTS |
| 通过 | $PASSED_TESTS |
| 失败 | $FAILED_TESTS |
| 跳过 | $SKIPPED_TESTS |
| **通过率** | **${pass_rate}%** |

## 测试结果概览

$(if [ $FAILED_TESTS -eq 0 ]; then
    echo "✅ **所有测试通过！** 系统功能完整。"
else
    echo "⚠️ **存在 $FAILED_TESTS 个失败测试**，需要修复。"
fi)

## 失败测试详情

$(if [ $FAILED_TESTS -gt 0 ]; then
    echo "| 测试名称 | 详情 |"
    echo "|---------|------|"
    for i in "${!FAILED_TEST_NAMES[@]}"; do
        echo "| ${FAILED_TEST_NAMES[$i]} | ${FAILED_TEST_DETAILS[$i]} |"
    done
else
    echo "无失败测试"
fi)

## 服务覆盖

- ✅ 认证服务 (auth-service:8082)
- ✅ 查询服务 (query-service:8089)
- ✅ 涨停复盘服务 (limit-review-service:8088)
- ✅ 竞价存储服务 (auction-storage:8084)
- ✅ 实时行情服务 (realtime-service:8090)

## 建议

$(if [ $FAILED_TESTS -gt 0 ]; then
    echo "1. 优先修复失败的服务功能"
    echo "2. 检查服务日志: \`tail -f logs/*.log\`"
    echo "3. 确认数据库连接正常"
else
    echo "1. 继续保持代码质量"
    echo "2. 定期运行功能测试"
fi)

---

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')
**测试脚本**: scripts/functional-test.sh
EOF

    log_success "测试报告已保存到: $report_file"

    ###########################################################################
    # 返回值
    ###########################################################################
    if [ $FAILED_TESTS -gt 0 ]; then
        print_header "测试结果"
        echo -e "${RED}❌ 测试未完全通过${NC}"
        return 1
    else
        print_header "测试结果"
        echo -e "${GREEN}✅ 所有测试通过！${NC}"
        return 0
    fi
}

###############################################################################
# 执行主函数
###############################################################################

main "$@"
