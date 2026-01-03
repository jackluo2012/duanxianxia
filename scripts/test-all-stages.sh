#!/bin/bash

# 完整测试套件
# 测试所有三个阶段的功能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# 统计变量
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 记录测试结果
record_test() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ $1 -eq 0 ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        print_success "$2 - 通过"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        print_error "$2 - 失败"
    fi
}

# 获取脚本目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

print_header "短线侠 - 完整测试套件"
print_info "项目路径: $PROJECT_ROOT"
print_info "开始时间: $(date '+%Y-%m-%d %H:%M:%S')"

# ============================================================================
# Phase 1: 基础设施测试
# ============================================================================
print_header "Phase 1: 基础设施测试"

print_info "检查 Docker 容器状态..."
if docker ps | grep -q "redis"; then
    print_success "Redis 容器运行中"
    record_test 0 "Redis容器检查"
else
    print_error "Redis 容器未运行"
    record_test 1 "Redis容器检查"
fi

if docker ps | grep -q "clickhouse"; then
    print_success "ClickHouse 容器运行中"
    record_test 0 "ClickHouse容器检查"
else
    print_error "ClickHouse 容器未运行"
    record_test 1 "ClickHouse容器检查"
fi

if docker ps | grep -q "postgres"; then
    print_success "PostgreSQL 容器运行中"
    record_test 0 "PostgreSQL容器检查"
else
    print_error "PostgreSQL 容器未运行"
    record_test 1 "PostgreSQL容器检查"
fi

print_info "测试数据库连接..."
# ClickHouse连接测试
if docker exec $(docker ps -q -f name=clickhouse) clickhouse-client --query "SELECT 1" &>/dev/null; then
    print_success "ClickHouse 数据库连接成功"
    record_test 0 "ClickHouse连接测试"
else
    print_error "ClickHouse 数据库连接失败"
    record_test 1 "ClickHouse连接测试"
fi

# Redis连接测试
if docker exec $(docker ps -q -f name=redis) redis-cli PING &>/dev/null; then
    print_success "Redis 连接成功"
    record_test 0 "Redis连接测试"
else
    print_error "Redis 连接失败"
    record_test 1 "Redis连接测试"
fi

# ============================================================================
# Phase 2: 编译测试
# ============================================================================
print_header "Phase 2: 编译测试"

print_info "编译 data-collector..."
if cargo build --release -p data-collector &>/dev/null; then
    print_success "data-collector 编译成功"
    record_test 0 "data-collector编译"
else
    print_error "data-collector 编译失败"
    record_test 1 "data-collector编译"
fi

print_info "编译 storage-service..."
if cargo build --release -p storage-service &>/dev/null; then
    print_success "storage-service 编译成功"
    record_test 0 "storage-service编译"
else
    print_error "storage-service 编译失败"
    record_test 1 "storage-service编译"
fi

print_info "编译 realtime-service..."
if cargo build --release -p realtime-service &>/dev/null; then
    print_success "realtime-service 编译成功"
    record_test 0 "realtime-service编译"
else
    print_error "realtime-service 编译失败"
    record_test 1 "realtime-service编译"
fi

print_info "编译 auction-storage..."
if cargo build --release -p auction-storage &>/dev/null; then
    print_success "auction-storage 编译成功"
    record_test 0 "auction-storage编译"
else
    print_error "auction-storage 编译失败"
    record_test 1 "auction-storage编译"
fi

# ============================================================================
# Phase 3: 单元测试
# ============================================================================
print_header "Phase 3: 单元测试"

print_info "运行 data-collector 单元测试..."
if cargo test -p data-collector --lib &>/dev/null; then
    print_success "data-collector 单元测试通过"
    record_test 0 "data-collector单元测试"
else
    print_error "data-collector 单元测试失败"
    record_test 1 "data-collector单元测试"
fi

print_info "运行 quality_monitor 单元测试..."
if cargo test -p data-collector --lib quality_monitor &>/dev/null; then
    print_success "quality_monitor 单元测试通过"
    record_test 0 "quality_monitor单元测试"
else
    print_error "quality_monitor 单元测试失败"
    record_test 1 "quality_monitor单元测试"
fi

print_info "运行 auction-storage 单元测试..."
if cargo test -p auction-storage --lib &>/dev/null; then
    print_success "auction-storage 单元测试通过"
    record_test 0 "auction-storage单元测试"
else
    print_error "auction-storage 单元测试失败"
    record_test 1 "auction-storage单元测试"
fi

# ============================================================================
# Phase 4: 集成测试
# ============================================================================
print_header "Phase 4: 集成测试"

print_info "测试 Redis Stream 功能..."
REDIS_CONTAINER=$(docker ps -q -f name=redis)
if [ -n "$REDIS_CONTAINER" ]; then
    # 添加测试数据
    docker exec $REDIS_CONTAINER redis-cli XADD stock_quotes "*" code "000001" name "测试股票" price "10.50" &>/dev/null
    # 读取数据
    COUNT=$(docker exec $REDIS_CONTAINER redis-cli XLEN stock_quotes 2>/dev/null)
    if [ "$COUNT" -gt 0 ]; then
        print_success "Redis Stream 功能正常 (当前条数: $COUNT)"
        record_test 0 "Redis Stream测试"
    else
        print_error "Redis Stream 功能异常"
        record_test 1 "Redis Stream测试"
    fi
else
    print_error "Redis 容器未找到"
    record_test 1 "Redis Stream测试"
fi

print_info "测试 ClickHouse 写入功能..."
CH_CONTAINER=$(docker ps -q -f name=clickhouse)
if [ -n "$CH_CONTAINER" ]; then
    # 查询数据
    RESULT=$(docker exec $CH_CONTAINER clickhouse-client --query "SELECT count() FROM stock_quotes WHERE datetime >= today()" 2>/dev/null)
    if [ -n "$RESULT" ]; then
        print_success "ClickHouse 查询功能正常 (今日数据: $RESULT 条)"
        record_test 0 "ClickHouse查询测试"
    else
        print_error "ClickHouse 查询功能异常"
        record_test 1 "ClickHouse查询测试"
    fi
else
    print_error "ClickHouse 容器未找到"
    record_test 1 "ClickHouse查询测试"
fi

# ============================================================================
# Phase 5: API 测试
# ============================================================================
print_header "Phase 5: API 测试"

print_info "检查 storage-service API..."
if curl -s http://localhost:8083/health &>/dev/null; then
    print_success "storage-service API 响应正常"
    record_test 0 "storage-service API测试"
else
    print_warning "storage-service 未运行或无法访问"
    record_test 1 "storage-service API测试"
fi

print_info "检查 auction-storage API..."
if curl -s http://localhost:8084/health &>/dev/null; then
    print_success "auction-storage API 响应正常"
    record_test 0 "auction-storage API测试"
else
    print_warning "auction-storage 未运行或无法访问"
    record_test 1 "auction-storage API测试"
fi

print_info "测试 K线数据查询API..."
if curl -s "http://localhost:8083/api/kline/000001?period=5m&limit=10" &>/dev/null; then
    print_success "K线数据查询API 响应正常"
    record_test 0 "K线查询API测试"
else
    print_warning "K线数据查询API 未运行或无法访问"
    record_test 1 "K线查询API测试"
fi

# ============================================================================
# Phase 6: 数据质量监控测试
# ============================================================================
print_header "Phase 6: 数据质量监控测试"

print_info "检查质量监控表..."
CH_CONTAINER=$(docker ps -q -f name=clickhouse)
if [ -n "$CH_CONTAINER" ]; then
    # 检查表是否存在
    TABLE_EXISTS=$(docker exec $CH_CONTAINER clickhouse-client --query "EXISTS TABLE data_quality_metrics" 2>/dev/null)
    if [ "$TABLE_EXISTS" = "1" ]; then
        print_success "data_quality_metrics 表存在"
        record_test 0 "质量监控表检查"
    else
        print_warning "data_quality_metrics 表不存在"
        record_test 1 "质量监控表检查"
    fi

    TABLE_EXISTS=$(docker exec $CH_CONTAINER clickhouse-client --query "EXISTS TABLE abnormal_data_log" 2>/dev/null)
    if [ "$TABLE_EXISTS" = "1" ]; then
        print_success "abnormal_data_log 表存在"
        record_test 0 "异常日志表检查"
    else
        print_warning "abnormal_data_log 表不存在"
        record_test 1 "异常日志表检查"
    fi

    TABLE_EXISTS=$(docker exec $CH_CONTAINER clickhouse-client --query "EXISTS TABLE data_repair_log" 2>/dev/null)
    if [ "$TABLE_EXISTS" = "1" ]; then
        print_success "data_repair_log 表存在"
        record_test 0 "修复日志表检查"
    else
        print_warning "data_repair_log 表不存在"
        record_test 1 "修复日志表检查"
    fi
else
    print_error "ClickHouse 容器未找到"
    record_test 1 "质量监控表检查"
    record_test 1 "异常日志表检查"
    record_test 1 "修复日志表检查"
fi

# ============================================================================
# Phase 7: 性能测试
# ============================================================================
print_header "Phase 7: 性能测试"

print_info "测试 API 响应时间..."
if command -v curl &>/dev/null; then
    START_TIME=$(date +%s%N)
    curl -s http://localhost:8083/health &>/dev/null
    END_TIME=$(date +%s%N)
    RESPONSE_TIME=$(( (END_TIME - START_TIME) / 1000000 ))

    if [ $RESPONSE_TIME -lt 200 ]; then
        print_success "API 响应时间: ${RESPONSE_TIME}ms (< 200ms)"
        record_test 0 "API响应时间测试"
    else
        print_warning "API 响应时间: ${RESPONSE_TIME}ms (目标: < 200ms)"
        record_test 1 "API响应时间测试"
    fi
else
    print_warning "curl 命令不可用"
    record_test 1 "API响应时间测试"
fi

# ============================================================================
# 测试总结
# ============================================================================
print_header "测试总结"

SUCCESS_RATE=0
if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(( PASSED_TESTS * 100 / TOTAL_TESTS ))
fi

echo "总测试数: $TOTAL_TESTS"
echo -e "通过: ${GREEN}$PASSED_TESTS${NC}"
echo -e "失败: ${RED}$FAILED_TESTS${NC}"
echo "通过率: $SUCCESS_RATE%"

if [ $SUCCESS_RATE -ge 90 ]; then
    print_success "测试通过率优秀 (>= 90%)"
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  所有测试完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
elif [ $SUCCESS_RATE -ge 70 ]; then
    print_warning "测试通过率一般 (>= 70%，< 90%)"
    exit 0
else
    print_error "测试通过率过低 (< 70%)"
    exit 1
fi
