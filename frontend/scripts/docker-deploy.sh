#!/bin/bash

# Docker部署脚本 - 使用Docker Compose部署

set -e

echo "======================================"
echo "  Docker部署脚本"
echo "======================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 检查Docker
check_docker() {
    echo -e "${YELLOW}检查Docker环境...${NC}"

    if ! command -v docker &> /dev/null; then
        echo "错误: Docker未安装"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        echo "错误: docker-compose未安装"
        exit 1
    fi

    echo -e "${GREEN}✓ Docker: $(docker --version)${NC}"
    echo -e "${GREEN}✓ docker-compose: $(docker-compose --version)${NC}"
    echo ""
}

# 构建镜像
build_image() {
    echo -e "${YELLOW}构建Docker镜像...${NC}"
    docker-compose build
    echo -e "${GREEN}✓ 镜像构建完成${NC}"
    echo ""
}

# 启动容器
start_containers() {
    echo -e "${YELLOW}启动容器...${NC}"
    docker-compose up -d
    echo -e "${GREEN}✓ 容器启动完成${NC}"
    echo ""

    echo "容器状态:"
    docker-compose ps
    echo ""
}

# 查看日志
view_logs() {
    echo "查看日志 (Ctrl+C退出)..."
    docker-compose logs -f
}

# 主流程
main() {
    check_docker

    # 解析参数
    ACTION="start"

    while [[ $# -gt 0 ]]; do
        case $1 in
            build)
                ACTION="build"
                shift
                ;;
            logs)
                ACTION="logs"
                shift
                ;;
            stop)
                ACTION="stop"
                shift
                ;;
            restart)
                ACTION="restart"
                shift
                ;;
            -h|--help)
                echo "用法: $0 [选项]"
                echo ""
                echo "选项:"
                echo "  (无)           构建并启动容器"
                echo "  build          仅构建镜像"
                echo "  logs           查看日志"
                echo "  stop           停止容器"
                echo "  restart        重启容器"
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

    # 执行操作
    case $ACTION in
        build)
            build_image
            ;;
        start)
            build_image
            start_containers
            ;;
        logs)
            view_logs
            ;;
        stop)
            echo "停止容器..."
            docker-compose down
            echo -e "${GREEN}✓ 容器已停止${NC}"
            ;;
        restart)
            echo "重启容器..."
            docker-compose restart
            echo -e "${GREEN}✓ 容器已重启${NC}"
            ;;
    esac

    echo -e "${GREEN}======================================"
    echo "  操作完成！"
    echo "======================================${NC}"
}

main "$@"
