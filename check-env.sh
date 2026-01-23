#!/bin/bash

# 短线侠 - 环境检查脚本
# 用途: 部署前检查环境是否满足要求

# 检查是否在 bash 环境中运行
if [ -z "$BASH_VERSION" ]; then
    echo "❌ 错误: 此脚本需要 bash 环境"
    echo "请使用以下命令运行: bash $0"
    exit 1
fi

# 不使用 set -e,因为某些检查命令可能返回非零退出码

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查计数器
CHECKS_PASSED=0
CHECKS_FAILED=0
WARNINGS=0

echo "🔍 短线侠系统 - 环境检查"
echo "========================================"
echo ""

# 辅助函数
pass() {
    echo -e "${GREEN}✅ $1${NC}"
    ((CHECKS_PASSED++))
}

fail() {
    echo -e "${RED}❌ $1${NC}"
    ((CHECKS_FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARNINGS++))
}

# ==================== 必检项 ====================

echo "📋 必检项检查:"
echo ""

# 1. Docker 检查
echo -n "检查 Docker..."
if command -v docker &> /dev/null; then
    if docker info &> /dev/null; then
        DOCKER_VERSION=$(docker --version | grep -oP '\d+\.\d+\.\d+' | head -1)
        pass "Docker ${DOCKER_VERSION} 已运行"
    else
        fail "Docker 未运行,请先启动 Docker"
    fi
else
    fail "Docker 未安装,请先安装 Docker"
fi

# 2. Docker Compose 检查
echo -n "检查 Docker Compose..."
if command -v docker-compose &> /dev/null; then
    COMPOSE_VERSION=$(docker-compose --version | grep -oP '\d+\.\d+\.\d+' | head -1)
    pass "Docker Compose ${COMPOSE_VERSION} 已安装"
else
    fail "Docker Compose 未安装,请先安装"
fi

# 3. Rust 工具链检查
echo -n "检查 Rust 工具链..."
if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | grep -oP '\d+\.\d+\.\d+' | head -1)
    pass "Rust ${RUST_VERSION} 工具链完整"
else
    fail "Rust 工具链不完整,请先安装"
fi

# 4. 端口占用检查
echo ""
echo "检查端口占用..."
PORTS_TO_CHECK=(8080 8082 8083 8084 8085 6379 5433 8123 9000)
PORTS_OCCUPIED=()

for port in "${PORTS_TO_CHECK[@]}"; do
    if lsof -ti:$port &> /dev/null; then
        PORTS_OCCUPIED+=($port)
    fi
done

if [ ${#PORTS_OCCUPIED[@]} -eq 0 ]; then
    pass "所有端口未被占用"
    ((CHECKS_PASSED++))
else
    warn "以下端口被占用: ${PORTS_OCCUPIED[*]}"
    echo "   deploy.sh 会自动清理这些端口"
    ((WARNINGS++))
fi

# 5. 磁盘空间检查
echo -n "检查磁盘空间..."
AVAILABLE_DISK=$(df -BG . | tail -1 | awk '{print $4}' | tr -d 'G')
if [ "$AVAILABLE_DISK" -ge 5 ]; then
    pass "磁盘可用空间: ${AVAILABLE_DISK}GB"
else
    fail "磁盘可用空间不足: ${AVAILABLE_DISK}GB (需要至少 5GB)"
fi

# 6. 内存空间检查
echo -n "检查内存空间..."
if command -v free &> /dev/null; then
    AVAILABLE_MEM=$(free -m | awk '/^Mem:/{print $7}')
    AVAILABLE_MEM_GB=$((AVAILABLE_MEM / 1024))
    if [ "$AVAILABLE_MEM_GB" -ge 2 ]; then
        pass "可用内存: ${AVAILABLE_MEM_GB}GB"
    else
        fail "可用内存不足: ${AVAILABLE_MEM_GB}GB (需要至少 2GB)"
    fi
else
    warn "无法检查内存空间 (free 命令不可用)"
fi

# 7. 文件权限检查
echo -n "检查文件权限..."
if [ -w "logs" ] 2>/dev/null || [ ! -d "logs" ]; then
    if [ ! -d "logs" ]; then
        mkdir -p logs 2>/dev/null || true
    fi
    if [ -w "logs" ] 2>/dev/null || [ ! -d "logs" ]; then
        pass "文件权限正常"
    else
        fail "无法写入 logs 目录,请检查权限"
    fi
else
    pass "文件权限正常"
fi

# ==================== 警告项 ====================

echo ""
echo "⚠️  警告项检查:"
echo ""

# 1. Git 状态检查
echo -n "检查 Git 状态..."
if git status --porcelain 2>/dev/null | grep -q .; then
    CHANGED_FILES=$(git status --porcelain 2>/dev/null | wc -l)
    warn "检测到 ${CHANGED_FILES} 个未提交的变更"
    git status --short 2>/dev/null | head -5 | sed 's/^/   /'
    if [ $CHANGED_FILES -gt 5 ]; then
        echo "   ..."
    fi
else
    pass "工作目录干净"
    ((CHECKS_PASSED++))
fi

# 2. 环境变量文件检查
echo -n "检查环境变量文件..."
MISSING_ENV=()
for service in services/*/; do
    if [ -f "$service/.env.example" ] && [ ! -f "$service/.env" ]; then
        MISSINg_ENV+=($(basename $service))
    fi
done

if [ ${#MISSING_ENV[@]} -eq 0 ]; then
    pass "环境变量文件完整"
    ((CHECKS_PASSED++))
else
    warn "以下服务缺少 .env 文件: ${MISSING_ENV[*]}"
    echo "   将从 .env.example 复制"
fi

# 3. 网络连接检查
echo -n "检查网络连接..."
if command -v curl &> /dev/null; then
    if curl -s --head https://crates.io | head -n 1 | grep -q "200"; then
        pass "可访问 crates.io"
        ((CHECKS_PASSED++))
    else
        warn "无法访问 crates.io,可能影响依赖下载"
    fi
else
    warn "curl 不可用,无法检查网络连接"
fi

# ==================== 汇总 ====================

echo ""
echo "========================================"
echo "📊 检查结果汇总:"
echo "  通过: ${CHECKS_PASSED}"
echo "  失败: ${CHECKS_FAILED}"
echo "  警告: ${WARNINGS}"
echo ""

# 判断是否可以继续
if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 环境检查通过!${NC}"
    echo "可以继续部署。"
    exit 0
else
    echo -e "${RED}❌ 环境检查失败!${NC}"
    echo "请解决上述问题后再部署。"
    exit 1
fi
