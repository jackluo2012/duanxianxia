.PHONY: help build test run clean docker-build docker-up docker-down docker-logs integration cli metrics-view config-view config-validate migrate-list

help: ## 显示帮助信息
	@echo "Backtest Service Makefile"
	@echo ""
	@echo "核心命令:"
	@echo "  make build          - 构建项目"
	@echo "  make test           - 运行测试"
	@echo "  make run            - 运行服务"
	@echo "  make clean          - 清理构建文件"
	@echo ""
	@echo "Docker 命令:"
	@echo "  make docker-build   - 构建 Docker 镜像"
	@echo "  make docker-up      - 启动 Docker Compose"
	@echo "  make docker-down    - 停止 Docker Compose"
	@echo "  make docker-logs    - 查看 Docker 日志"
	@echo ""
	@echo "CLI 命令:"
	@echo "  make cli-list       - 列出可用策略"
	@echo "  make cli-run-auction - 运行竞价龙头策略"
	@echo "  make cli-run-seal   - 运行竞价封单策略"
	@echo "  make cli            - CLI 帮助"
	@echo ""
	@echo "指标命令:"
	@echo "  make metrics-help   - 显示指标信息"
	@echo "  make metrics-view   - 查看 Prometheus 指标"
	@echo ""
	@echo "配置命令:"
	@echo "  make config-view    - 查看当前配置"
	@echo "  make config-validate - 验证配置文件"
	@echo ""
	@echo "数据库命令:"
	@echo "  make migrate-list   - 列出所有迁移"
	@echo "  make migrate-create NAME=xxx - 创建新迁移"
	@echo ""
	@echo "开发命令:"
	@echo "  make integration    - 运行集成测试"
	@echo "  make format         - 格式化代码"
	@echo "  make clippy         - 运行 Clippy"
	@echo "  make dev            - 开发模式"

build: ## 构建项目
	cargo build --release

test: ## 运行所有测试
	cargo test --lib --verbose

test-quick: ## 快速测试 (不带输出)
	cargo test --lib

run: ## 运行服务
	cargo run

clean: ## 清理构建文件
	cargo clean
	rm -rf docker/*.log

docker-build: ## 构建 Docker 镜像
	docker-compose build

docker-up: ## 启动服务 (ClickHouse + Backtest Service)
	docker-compose up -d
	@echo "等待服务启动..."
	@sleep 5
	@docker-compose ps

docker-down: ## 停止服务
	docker-compose down

docker-restart: docker-down docker-up

docker-logs: ## 查看服务日志
	docker-compose logs -f backtest-service

docker-logs-clickhouse: ## 查看 ClickHouse 日志
	docker-compose logs -f clickhouse

integration: ## 运行集成测试 (需要服务先启动)
	@echo "请确保服务已启动 (make docker-up 或 make run)"
	@chmod +x tests/integration_test.sh
	@bash tests/integration_test.sh

format: ## 格式化代码
	cargo fmt --all

clippy: ## 运行 Clippy 检查
	cargo clippy --all-targets --all-features

check: test clippy ## 完整检查 (测试 + Clippy)

install-tools: ## 安装开发工具
	cargo install cargo-watch
	cargo install cargo-audit

dev: ## 开发模式 (自动重新编译)
	cargo watch -x run

dev-test: ## 开发模式 (自动重新测试)
	cargo watch -x test

# CLI 命令
cli-list: ## 列出可用策略
	cargo run -- list

cli-run-auction: ## 运行竞价龙头策略回测
	cargo run -- run auction-leader \
		--start-date 2025-10-01 \
		--end-date 2025-10-31 \
		--capital 100000 \
		--strength-score 80 \
		--seal-amount 1000 \
		--holding-days 1 \
		--output table

cli-run-seal: ## 运行竞价封单策略回测
	cargo run -- run auction-seal \
		--start-date 2025-10-01 \
		--end-date 2025-10-31 \
		--capital 100000 \
		--holding-days 3 \
		--output table

cli: ## CLI 帮助
	cargo run -- --help

# 指标命令
metrics-help: ## 显示指标端点信息
	@echo "Prometheus 指标端点:"
	@echo "  服务端口 8086: http://localhost:8086/metrics"
	@echo "  Prometheus 端口 9091: http://localhost:9091/metrics"
	@echo ""
	@echo "可用指标:"
	@echo "  - backtest_started_total: 回测启动计数"
	@echo "  - backtest_completed_total: 回测完成计数"
	@echo "  - backtest_failed_total: 回测失败计数"
	@echo "  - backtest_duration_seconds: 回测执行时间"
	@echo "  - http_requests_total: HTTP 请求计数"
	@echo "  - http_request_duration_seconds: HTTP 请求延迟"
	@echo "  - queue_*_tasks: 队列状态"
	@echo "  - backtest_*: 资金和交易指标"

metrics-view: ## 查看 Prometheus 指标 (需要服务运行)
	@curl -s http://localhost:8086/metrics || echo "服务未运行"

# 配置命令
config-view: ## 查看当前配置
	@cat config/development.toml 2>/dev/null || echo "配置文件不存在"

config-validate: ## 验证配置文件格式
	@echo "验证配置文件..."
	@if [ -f config/development.toml ]; then \
		echo "✅ TOML 配置文件存在"; \
	else \
		echo "❌ 配置文件不存在"; \
	fi

# 数据库迁移命令
migrate-list: ## 列出所有迁移文件
	@ls -la migrations/ 2>/dev/null || echo "迁移目录不存在"

migrate-create: ## 创建新的迁移文件 (需要提供 NAME 参数)
	@if [ -z "$(NAME)" ]; then \
		echo "用法: make migrate-create NAME=create_users_table"; \
		exit 1; \
	fi
	@VERSION=$$(ls migrations/ 2>/dev/null | wc -l | tr -d ' '); \
	VERSION=$$(printf "%03d" $$((VERSION + 1))); \
	FILE="migrations/$${VERSION}_$(NAME).sql"; \
	echo "-- Migration: $(NAME)" > $$FILE; \
	echo "-- Created: $$(date)" >> $$FILE; \
	echo "" >> $$FILE; \
	echo "-- 在此编写 SQL 语句" >> $$FILE; \
	echo "✅ 创建迁移文件: $$FILE"
