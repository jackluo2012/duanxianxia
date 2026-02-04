#!/bin/bash

# 部署脚本 - 用于快速部署前端应用

set -e

echo "======================================"
echo "  短线侠前端部署脚本"
echo "======================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查环境
check_env() {
    echo -e "${YELLOW}检查环境...${NC}"

    # 检查Node.js
    if ! command -v node &> /dev/null; then
        echo -e "${RED}错误: Node.js未安装${NC}"
        exit 1
    fi

    # 检查npm
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}错误: npm未安装${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Node.js: $(node -v)${NC}"
    echo -e "${GREEN}✓ npm: $(npm -v)${NC}"
    echo ""
}

# 安装依赖
install_deps() {
    echo -e "${YELLOW}安装依赖...${NC}"
    npm ci
    echo -e "${GREEN}✓ 依赖安装完成${NC}"
    echo ""
}

# 运行测试
run_tests() {
    if [ "$SKIP_TESTS" != "true" ]; then
        echo -e "${YELLOW}运行测试...${NC}"
        npm run test -- --run
        echo -e "${GREEN}✓ 测试通过${NC}"
        echo ""
    else
        echo -e "${YELLOW}跳过测试${NC}"
        echo ""
    fi
}

# 构建应用
build_app() {
    echo -e "${YELLOW}构建应用...${NC}"
    npm run build
    echo -e "${GREEN}✓ 构建完成${NC}"
    echo ""
}

# 部署到生产
deploy_prod() {
    echo -e "${YELLOW}部署到生产环境...${NC}"

    # 这里可以根据实际部署方式进行配置
    # 例如：rsync到服务器、上传到CDN等

    echo -e "${GREEN}✓ 部署完成${NC}"
    echo ""
}

# 清理
cleanup() {
    echo -e "${YELLOW}清理临时文件...${NC}"
    rm -rf .temp
    echo -e "${GREEN}✓ 清理完成${NC}"
    echo ""
}

# 主流程
main() {
    # 解析参数
    SKIP_TESTS=false
    DEPLOY=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --deploy)
                DEPLOY=true
                shift
                ;;
            -h|--help)
                echo "用法: $0 [选项]"
                echo ""
                echo "选项:"
                echo "  --skip-tests    跳过测试"
                echo "  --deploy       部署到生产环境"
                echo "  -h, --help     显示帮助"
                exit 0
                ;;
            *)
                echo "未知选项: $1"
                echo "使用 -h 或 --help 查看帮助"
                exit 1
                ;;
        esac
    done

    # 执行流程
    check_env
    install_deps
    run_tests
    build_app

    if [ "$DEPLOY" = true ]; then
        deploy_prod
    fi

    cleanup

    echo -e "${GREEN}======================================"
    echo "  部署成功！"
    echo "======================================${NC}"
}

# 运行主流程
main "$@"
