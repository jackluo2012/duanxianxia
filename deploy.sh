#!/bin/bash

# 短线侠系统 - 多模式部署脚本
# 用途: 支持 quick/full/update 三种部署模式

# 检查是否在 bash 环境中运行
if [ -z "$BASH_VERSION" ]; then
    echo "❌ 错误: 此脚本需要 bash 环境"
    echo "请使用以下命令运行: bash $0"
    exit 1
fi

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志文件
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="logs/deploy-${TIMESTAMP}.log"
BACKUP_DIR="backup/config-${TIMESTAMP}"

# 创建必要目录
mkdir -p logs
mkdir -p backup
mkdir -p "$BACKUP_DIR"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} [$] $*" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "$LOG_FILE"
}

# 显示帮助信息
show_help() {
    cat << EOF
短线侠系统 - 部署脚本

用法: $0 [模式]

模式:
  quick    快速部署 (默认) - 重启服务,保留数据
  full     完全部署 - 清理后重新部署
  update   增量更新 - 更新代码并重新编译

选项:
  -h, --help     显示此帮助信息
  --no-check     跳过环境检查
  --no-backup    跳过备份(仅 quick 模式)

示例:
  $0              # 快速部署
  $0 quick        # 快速部署
  $0 full         # 完全部署
  $0 update       # 增量更新

EOF
}

# 解析命令行参数
DEPLOY_MODE="quick"
SKIP_CHECK=false
SKIP_BACKUP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        quick|full|update)
            DEPLOY_MODE="$1"
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        --no-check)
            SKIP_CHECK=true
            shift
            ;;
        --no-backup)
            SKIP_BACKUP=true
            shift
            ;;
        *)
            echo "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# ==================== 部署开始 ====================

echo ""
echo "🚀 短线侠系统 - 部署脚本"
echo "========================================"
log "开始部署 (模式: ${DEPLOY_MODE})"
echo ""

# 1. 环境检查
if [ "$SKIP_CHECK" = false ]; then
    log_info "步骤 1: 环境检查..."
    if ! ./check-env.sh; then
        log_error "环境检查失败,部署中止"
        exit 1
    fi
    echo ""
else
    log_warn "跳过环境检查"
fi

# 2. 备份配置
if [ "$DEPLOY_MODE" != "full" ] && [ "$SKIP_BACKUP" = false ]; then
    log_info "步骤 2: 备份配置..."
    BACKUP_SUCCESS=true
    for env_file in services/*/.env; do
        if [ -f "$env_file" ]; then
            SERVICE=$(basename $(dirname "$env_file"))
            if cp "$env_file" "$BACKUP_DIR/$SERVICE.env"; then
                log "  已备份: $SERVICE/.env"
            else
                log_warn "  备份失败: $SERVICE/.env"
                BACKUP_SUCCESS=false
            fi
        fi
    done

    if [ "$BACKUP_SUCCESS" = true ]; then
        log_info "配置备份完成: $BACKUP_DIR"
    else
        log_warn "部分配置备份失败,继续部署"
    fi
    echo ""
fi

# 3. 执行部署
case $DEPLOY_MODE in
    quick)
        # ==================== 快速部署模式 ====================
        log_info "步骤 3: 快速部署 - 重启服务"
        echo ""

        # 停止服务
        log_info "停止现有服务..."
        if [ -f "./stop-all.sh" ]; then
            ./stop-all.sh 2>&1 | tee -a "$LOG_FILE" || true
        fi
        sleep 2

        # 清理端口占用
        log_info "清理端口占用..."
        for port in 8080 8082 8083 8084 8085 8087 8088 8089 6379; do
            PID=$(lsof -ti:$port 2>/dev/null || true)
            if [ -n "$PID" ]; then
                log_info "  - 停止端口 $port 的进程 $PID"
                kill $PID 2>/dev/null || true
                sleep 1
                kill -9 $PID 2>/dev/null || true
            fi
        done
        sleep 2

        # 重新编译并启动
        log_info "重新编译并启动服务..."
        ./start-all.sh 2>&1 | tee -a "$LOG_FILE"

        log_info "快速部署完成"
        ;;

    full)
        # ==================== 完全部署模式 ====================
        log_warn "⚠️  完全重置模式：将删除所有数据，像重新部署一套新系统"
        sleep 3
        log_info "步骤 3: 完全部署 - 清理并重新部署"
        echo ""

        # 1. 停止所有服务
        log_info "[1/6] 停止所有服务..."
        if [ -f "./stop-all.sh" ]; then
            ./stop-all.sh 2>&1 | tee -a "$LOG_FILE" || true
        fi
        sleep 2

        # 2. 清理端口占用
        log_info "[2/6] 清理端口占用..."
        for port in 8080 8082 8083 8084 8085 8087 8088 8089 6379 5433; do
            PID=$(lsof -ti:$port 2>/dev/null || true)
            if [ -n "$PID" ]; then
                log_info "  - 停止端口 $port 的进程 $PID"
                kill $PID 2>/dev/null || true
                sleep 1
                kill -9 $PID 2>/dev/null || true
            fi
        done

        # 3. 清理 Docker 容器、网络和数据卷（完全删除数据）
        log_info "[3/6] 清理 Docker 资源（包括数据卷）..."
        log_warn "  - 将删除所有数据库数据！"
        docker-compose down -v 2>&1 | tee -a "$LOG_FILE" || true
        sleep 2

        # 删除所有相关容器
        docker ps -a --filter "name=duanxianxia" --format "{{.Names}}" | xargs -r docker rm -f 2>/dev/null || true

        # 删除所有相关网络
        docker network ls --filter "name=duanxianxia" --format "{{.Name}}" | xargs -r docker network rm 2>/dev/null || true

        # 删除所有相关数据卷
        docker volume ls --filter "name=duanxianxia" --format "{{.Name}}" | xargs -r docker volume rm -f 2>/dev/null || true
        docker volume rm -f duanxianxia_clickhouse_data duanxianxia_redis_data duanxianxia_postgres_data 2>/dev/null || true
        log_info "  ✅ Docker 资源已完全清理"

        # 4. 清理编译产物
        log_info "[4/6] 清理编译产物..."
        for service_dir in services/*/; do
            if [ -d "$service_dir/target" ]; then
                log "  清理 $service_dir/target"
                rm -rf "$service_dir/target"
            fi
        done
        log_info "  ✅ 编译产物已清理"

        # 5. 清理日志和 PID 文件
        log_info "[5/6] 清理日志和 PID 文件..."
        if [ -d "logs" ]; then
            find logs -name "*.pid" -type f -delete 2>/dev/null || true
            # 保留部分日志用于调试，只清理 PID 文件
            log_info "  ✅ PID 文件已清理"
        fi

        # 6. 启动数据库和服务
        log_info "[6/6] 启动全新系统..."
        log_info "  - 启动数据库服务..."
        docker-compose up -d redis clickhouse postgres 2>&1 | tee -a "$LOG_FILE"
        sleep 10

        log_info "  - 初始化数据库..."
        if [ -f "db/init.sql" ]; then
            docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/init.sql 2>&1 | tee -a "$LOG_FILE" || true
        fi
        if [ -f "db/auction.sql" ]; then
            docker exec -i $(docker ps -q -f name=clickhouse) clickhouse-client --multiquery < db/auction.sql 2>&1 | tee -a "$LOG_FILE" || true
        fi

        log_info "  - 启动所有服务..."
        ./start-all.sh 2>&1 | tee -a "$LOG_FILE"

        log_info "✅ 完全重置部署完成（全新系统）"
        ;;

    update)
        # ==================== 增量更新模式 ====================
        log_info "步骤 3: 增量更新 - 更新代码并重新编译"
        echo ""

        # 拉取最新代码
        log_info "拉取最新代码..."
        if git pull 2>&1 | tee -a "$LOG_FILE"; then
            log_info "代码更新成功"
        else
            log_warn "代码更新失败或无新代码"
        fi

        # 停止服务
        log_info "停止现有服务..."
        if [ -f "./stop-all.sh" ]; then
            ./stop-all.sh 2>&1 | tee -a "$LOG_FILE" || true
        fi
        sleep 2

        # 清理端口占用
        log_info "清理端口占用..."
        for port in 8080 8082 8083 8084 8085 8087 8088 8089 6379; do
            PID=$(lsof -ti:$port 2>/dev/null || true)
            if [ -n "$PID" ]; then
                log_info "  - 停止端口 $port 的进程 $PID"
                kill $PID 2>/dev/null || true
                sleep 1
                kill -9 $PID 2>/dev/null || true
            fi
        done
        sleep 2

        # 重新编译
        log_info "重新编译服务..."
        cd services/data-collector && cargo build --bin data-collector 2>&1 | tee -a "../../$LOG_FILE" && cd ../..
        cd services/storage-service && cargo build 2>&1 | tee -a "../../$LOG_FILE" && cd ../..
        cd services/realtime-service && cargo build 2>&1 | tee -a "../../$LOG_FILE" && cd ../..
        cd services/auth-service && cargo build 2>&1 | tee -a "../../$LOG_FILE" && cd ../..

        # 启动服务
        log_info "启动服务..."
        ./start-all.sh 2>&1 | tee -a "$LOG_FILE"

        log_info "增量更新完成"
        ;;
esac

# 4. 等待服务启动
echo ""
log_info "等待服务启动..."
sleep 5

# 5. 健康检查
if [ -f "./health-check.sh" ]; then
    log_info "步骤 4: 健康检查..."
    chmod +x ./health-check.sh
    if ./health-check.sh 2>&1 | tee -a "$LOG_FILE"; then
        log_info "✅ 健康检查通过"
    else
        log_warn "⚠️  健康检查发现问题,请查看日志"
    fi
else
    log_warn "health-check.sh 不存在,跳过健康检查"
fi

# ==================== 部署完成 ====================

echo ""
echo "========================================"
log_info "✅ 部署完成!"
echo ""
echo "📋 部署信息:"
echo "  模式: ${DEPLOY_MODE}"
echo "  时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "  日志: ${LOG_FILE}"
echo "  备份: ${BACKUP_DIR}"
echo ""
echo "📊 服务状态:"
docker-compose ps 2>/dev/null | tail -n +3 || echo "  (Docker 服务未运行)"
echo ""
echo "📋 查看日志:"
echo "  tail -f logs/data-collector.log"
echo "  tail -f logs/storage-service.log"
echo "  tail -f logs/realtime-service.log"
echo "  tail -f logs/auth-service.log"
echo ""
echo "✅ 部署成功!"
