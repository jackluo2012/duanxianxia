# Backtest Service 部署文档

## 目录

1. [环境要求](#环境要求)
2. [快速开始](#快速开始)
3. [本地开发](#本地开发)
4. [Docker 部署](#docker-部署)
5. [生产环境部署](#生产环境部署)
6. [配置说明](#配置说明)
7. [监控和运维](#监控和运维)
8. [故障排查](#故障排查)

---

## 环境要求

### 软件依赖

- **Rust**: 1.70.0 或更高版本
- **Docker**: 20.10.0 或更高版本
- **Docker Compose**: 2.0.0 或更高版本
- **ClickHouse**: 24.11 或更高版本
- **操作系统**: Linux (推荐 Ubuntu 22.04+)

### 硬件要求

#### 最低配置
- CPU: 2 核
- 内存: 2GB
- 磁盘: 10GB

#### 推荐配置（生产环境）
- CPU: 4 核或以上
- 内存: 8GB 或以上
- 磁盘: SSD 100GB 或以上
- 网络: 1Gbps

---

## 快速开始

### 1. 克隆项目

```bash
git clone <repository-url>
cd backtest-service
```

### 2. 使用 Docker Compose 启动

```bash
# 启动所有服务（ClickHouse + Backtest Service）
make docker-up

# 查看服务状态
docker-compose ps

# 查看日志
make docker-logs
```

### 3. 验证服务

```bash
# 健康检查
curl http://localhost:8086/health

# 查看指标
curl http://localhost:8086/metrics
```

---

## 本地开发

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 安装依赖

```bash
# 安装开发工具
make install-tools

# 安装项目依赖
cargo build
```

### 运行测试

```bash
# 运行所有测试
make test

# 快速测试（不显示详细输出）
make test-quick

# 运行 Clippy 检查
make clippy

# 代码格式化
make format
```

### 启动开发服务器

```bash
# 方式1: 直接运行
make run

# 方式2: 开发模式（自动重新编译）
make dev

# 方式3: CLI 模式
make cli-list
make cli-run-auction
```

---

## Docker 部署

### 1. 构建镜像

```bash
# 使用 Makefile
make docker-build

# 或使用 Docker Compose
docker-compose build
```

### 2. 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 3. 停止服务

```bash
# 停止所有服务
make docker-down

# 或使用 Docker Compose
docker-compose down
```

### 4. 重启服务

```bash
make docker-restart
```

---

## 生产环境部署

### 方案 1: Docker Compose（推荐用于小规模部署）

#### 1.1 创建生产配置文件

```bash
# 创建生产配置目录
mkdir -p config/production

# 复制并修改配置文件
cp config/development.toml config/production.toml
```

编辑 `config/production.toml`：

```toml
[database]
clickhouse_url = "http://clickhouse:8123"
pool_size = 20
query_timeout_secs = 60

[server]
host = "0.0.0.0"
port = 8086
metrics_port = 9091
max_body_size = 50  # MB

[backtest]
max_backtest_days = 90
default_commission_rate = 0.0003
min_initial_capital = 10000.0
max_concurrent_tasks = 10

[logging]
level = "info"
log_to_file = true
log_file = "/var/log/backtest-service/app.log"
```

#### 1.2 创建生产环境 Docker Compose 文件

创建 `docker-compose.prod.yml`：

```yaml
version: '3.8'

services:
  clickhouse:
    image: clickhouse/clickhouse-server:24.11
    container_name: backtest-clickhouse
    ports:
      - "8123:8123"
      - "9000:9000"
    environment:
      CLICKHOUSE_DB: backtest
      CLICKHOUSE_USER: default
      CLICKHOUSE_PASSWORD: ${CLICKHOUSE_PASSWORD:-changeme}
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./config/clickhouse/config.xml:/etc/clickhouse-server/config.d/custom.xml:ro
    restart: unless-stopped
    ulimits:
      nofile:
        soft: 262144
        hard: 262144

  backtest-service:
    image: backtest-service:latest
    container_name: backtest-service
    ports:
      - "8086:8086"
      - "9091:9091"
    environment:
      CLICKHOUSE_URL: http://clickhouse:8123
      RUST_LOG: info
    volumes:
      - ./config/production:/app/config:ro
      - ./logs:/var/log/backtest-service
    depends_on:
      - clickhouse
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8086/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  clickhouse_data:
    driver: local
```

#### 1.3 启动生产环境

```bash
# 启动生产环境
docker-compose -f docker-compose.prod.yml up -d

# 查看状态
docker-compose -f docker-compose.prod.yml ps

# 查看日志
docker-compose -f docker-compose.prod.yml logs -f backtest-service
```

### 方案 2: Kubernetes（推荐用于大规模部署）

#### 2.1 创建命名空间

```bash
kubectl create namespace backtest
```

#### 2.2 创建 ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: backtest-config
  namespace: backtest
data:
  development.toml: |
    [database]
    clickhouse_url = "http://clickhouse-service:8123"
    pool_size = 20
    query_timeout_secs = 60

    [server]
    host = "0.0.0.0"
    port = 8086
    metrics_port = 9091
    max_body_size = 50

    [backtest]
    max_backtest_days = 90
    default_commission_rate = 0.0003
    min_initial_capital = 10000.0
    max_concurrent_tasks = 10

    [logging]
    level = "info"
    log_to_file = true
    log_file = "/var/log/backtest-service/app.log"
```

应用 ConfigMap：

```bash
kubectl apply -f configmap.yaml
```

#### 2.3 创建 Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backtest-service
  namespace: backtest
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backtest-service
  template:
    metadata:
      labels:
        app: backtest-service
    spec:
      containers:
      - name: backtest-service
        image: backtest-service:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8086
          name: http
        - containerPort: 9091
          name: metrics
        env:
        - name: CLICKHOUSE_URL
          value: "http://clickhouse-service:8123"
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: logs
          mountPath: /var/log/backtest-service
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8086
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8086
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: backtest-config
      - name: logs
        emptyDir: {}
---
apiVersion: v1
kind: Service
metadata:
  name: backtest-service
  namespace: backtest
spec:
  selector:
    app: backtest-service
  ports:
  - name: http
    port: 8086
    targetPort: 8086
  - name: metrics
    port: 9091
    targetPort: 9091
  type: LoadBalancer
```

应用 Deployment：

```bash
kubectl apply -f deployment.yaml
```

#### 2.4 创建 Ingress（可选）

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: backtest-ingress
  namespace: backtest
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - backtest.example.com
    secretName: backtest-tls
  rules:
  - host: backtest.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: backtest-service
            port:
              number: 8086
```

应用 Ingress：

```bash
kubectl apply -f ingress.yaml
```

---

## 配置说明

### 环境变量

| 变量名 | 说明 | 默认值 | 必填 |
|--------|------|--------|------|
| `CLICKHOUSE_URL` | ClickHouse 连接地址 | `http://localhost:8123` | 否 |
| `RUST_LOG` | 日志级别 | `info` | 否 |
| `BIND_ADDRESS` | 服务监听地址 | `0.0.0.0:8086` | 否 |

### 配置文件

配置文件路径：`config/development.toml`

#### 数据库配置

```toml
[database]
clickhouse_url = "http://localhost:8123"
pool_size = 10                    # 连接池大小
query_timeout_secs = 30           # 查询超时（秒）
```

#### 服务配置

```toml
[server]
host = "0.0.0.0"                  # 监听地址
port = 8086                       # 监听端口
metrics_port = 9091               # Prometheus 端口
max_body_size = 10                # 请求体最大大小（MB）
```

#### 回测配置

```toml
[backtest]
max_backtest_days = 90            # 最大回测天数
default_commission_rate = 0.0003  # 默认手续费率
min_initial_capital = 10000.0     # 最小初始资金
max_concurrent_tasks = 5          # 最大并发任务数
```

#### 日志配置

```toml
[logging]
level = "info"                    # 日志级别
log_to_file = false               # 是否输出到文件
log_file = "logs/app.log"         # 日志文件路径
```

---

## 监控和运维

### 健康检查

```bash
# HTTP 健康检查
curl http://localhost:8086/health

# 返回示例
{
  "status": "ok",
  "service": "backtest-service"
}
```

### Prometheus 指标

#### 服务端口指标

```bash
curl http://localhost:8086/metrics
```

#### Prometheus 端口指标

```bash
curl http://localhost:9091/metrics
```

#### 主要指标

| 指标名 | 类型 | 说明 |
|--------|------|------|
| `backtest_started_total` | Counter | 回测启动总数 |
| `backtest_completed_total` | Counter | 回测完成总数 |
| `backtest_failed_total` | Counter | 回测失败总数 |
| `backtest_duration_seconds` | Histogram | 回测执行时间 |
| `http_requests_total` | Counter | HTTP 请求总数 |
| `http_request_duration_seconds` | Histogram | HTTP 请求延迟 |
| `queue_pending_tasks` | Gauge | 待处理任务数 |
| `queue_running_tasks` | Gauge | 运行中任务数 |
| `queue_completed_tasks` | Gauge | 已完成任务数 |

### 日志查看

#### Docker 环境

```bash
# 查看实时日志
docker-compose logs -f backtest-service

# 查看最近 100 行日志
docker-compose logs --tail=100 backtest-service

# 查看特定时间的日志
docker-compose logs --since=2024-01-01T00:00:00 backtest-service
```

#### Kubernetes 环境

```bash
# 查看 Pod 日志
kubectl logs -f deployment/backtest-service -n backtest

# 查看所有 Pod 日志
kubectl logs -f -l app=backtest-service -n backtest
```

### 性能监控

#### CPU 和内存监控

```bash
# Docker 容器资源使用
docker stats backtest-service

# Kubernetes Pod 资源使用
kubectl top pod -n backtest
```

#### ClickHouse 监控

```bash
# 进入 ClickHouse 容器
docker exec -it backtest-clickhouse clickhouse-client

# 查看查询统计
SELECT * FROM system.metrics WHERE metric LIKE '%Query%';

# 查看表大小
SELECT
    database,
    table,
    formatReadableSize(sum(bytes)) as size
FROM system.parts
WHERE active
GROUP BY database, table
ORDER BY size DESC;
```

---

## 故障排查

### 常见问题

#### 1. 服务无法启动

**症状**：执行 `make run` 后服务立即退出

**排查步骤**：

```bash
# 检查 ClickHouse 是否运行
curl http://localhost:8123/ping

# 查看 RUST_LOG 环境变量开启详细日志
RUST_LOG=debug cargo run

# 检查端口占用
lsof -i :8086
```

**解决方案**：

```bash
# 启动 ClickHouse
docker-compose up -d clickhouse

# 或使用 systemd 启动本地 ClickHouse
sudo systemctl start clickhouse-server
```

#### 2. 回测任务失败

**症状**：API 返回 500 错误

**排查步骤**：

```bash
# 查看服务日志
docker-compose logs backtest-service | tail -100

# 检查 ClickHouse 数据
docker exec -it backtest-clickhouse clickhouse-client \
  "SELECT count() FROM stock_auction_data"
```

**常见原因**：

- ClickHouse 数据不存在
- 日期范围超出数据范围
- 参数验证失败（资金太小、天数太多等）

#### 3. 内存不足

**症状**：OOM (Out of Memory) 错误

**排查步骤**：

```bash
# 检查容器内存限制
docker inspect backtest-service | grep -i memory

# 查看 ClickHouse 内存使用
docker exec backtest-clickhouse clickhouse-client \
  "SELECT formatReadableSize(sum(bytes)) FROM system.parts"
```

**解决方案**：

```yaml
# 增加内存限制（docker-compose.yml）
services:
  backtest-service:
    deploy:
      resources:
        limits:
          memory: 4G
```

#### 4. 配置热重载不生效

**症状**：修改配置文件后没有自动重新加载

**排查步骤**：

```bash
# 检查配置文件路径
ls -la config/development.toml

# 查看服务日志中的配置加载信息
docker-compose logs backtest-service | grep "配置"
```

**解决方案**：

```bash
# 确保配置文件路径正确
# 确保服务有权限读取配置文件

# 手动触发重新加载（如果自动重载失败）
docker-compose restart backtest-service
```

### 性能优化建议

#### 1. ClickHouse 优化

```sql
-- 创建物化视图加速查询
CREATE MATERIALIZED VIEW stock_auction_data_mv
ENGINE = SummingMergeTree()
ORDER BY (date, stock_code)
AS SELECT
    date,
    stock_code,
    sum(auction_amount) as total_auction_amount,
    avg(strength_score) as avg_strength_score
FROM stock_auction_data
GROUP BY date, stock_code;

-- 设置分区策略
ALTER TABLE stock_auction_data
MODIFY PARTITION BY toYYYYMM(date);
```

#### 2. 连接池优化

```toml
[database]
pool_size = 20              # 增加连接池大小
query_timeout_secs = 60     # 增加超时时间
```

#### 3. 并发控制

```toml
[backtest]
max_concurrent_tasks = 10   # 根据硬件调整并发数
```

### 备份和恢复

#### ClickHouse 备份

```bash
# 创建备份
docker exec backtest-clickhouse clickhouse-backup \
  create backup_$(date +%Y%m%d)

# 恢复备份
docker exec backtest-clickhouse clickhouse-backup \
  restore backup_20240101
```

#### 配置文件备份

```bash
# 备份配置
tar -czf config-backup-$(date +%Y%m%d).tar.gz config/

# 恢复配置
tar -xzf config-backup-20240101.tar.gz
```

---

## 安全加固

### 1. 使用 HTTPS

```yaml
# docker-compose.yml 添加 Nginx 反向代理
nginx:
  image: nginx:alpine
  ports:
    - "443:443"
  volumes:
    - ./nginx.conf:/etc/nginx/nginx.conf:ro
    - ./ssl:/etc/nginx/ssl:ro
  depends_on:
    - backtest-service
```

### 2. 限制访问

```yaml
# 使用防火墙规则
ufw allow from 10.0.0.0/8 to any port 8086
ufw deny from any to any port 8086
```

### 3. ClickHouse 安全

```xml
<!-- /etc/clickhouse-server/config.d/security.xml -->
<clickhouse>
  <users>
    <default>
      <password>strong_password_here</password>
      <networks>
        <ip>::/0</ip>
      </networks>
      <profile>default</profile>
      <quota>default</quota>
    </default>
  </users>
</clickhouse>
```

---

## 附录

### A. Makefile 命令参考

| 命令 | 说明 |
|------|------|
| `make help` | 显示帮助信息 |
| `make build` | 构建项目 |
| `make test` | 运行测试 |
| `make run` | 运行服务 |
| `make clean` | 清理构建 |
| `make docker-build` | 构建 Docker 镜像 |
| `make docker-up` | 启动 Docker Compose |
| `make docker-down` | 停止 Docker Compose |
| `make docker-logs` | 查看 Docker 日志 |
| `make metrics-view` | 查看 Prometheus 指标 |
| `make config-view` | 查看当前配置 |
| `make migrate-list` | 列出所有迁移 |

### B. API 端点参考

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /health | 健康检查 |
| GET | /metrics | Prometheus 指标 |
| POST | /api/backtest/run | 启动回测 |
| GET | /api/backtest/{id} | 获取回测结果 |
| GET | /api/backtest/strategies | 获取策略列表 |
| GET | /api/backtest/history | 获取回测历史 |

### C. 支持和联系

- 文档: `docs/`
- Issues: GitHub Issues
- 邮件: support@example.com

---

**最后更新**: 2026-01-12
**版本**: 1.0.0
