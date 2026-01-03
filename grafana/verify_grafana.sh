#!/bin/bash

# Grafana验证脚本
# 验证Grafana安装、插件和数据源连接

set -e

GRAFANA_URL="http://127.0.0.1:3002"
ADMIN_USER="admin"
ADMIN_PASSWORD="grafana_admin_2026"

echo "=========================================="
echo "Grafana验证脚本"
echo "=========================================="
echo ""

# Step 1: 检查Web界面可访问性
echo "Step 1: 检查Web界面可访问性..."
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" ${GRAFANA_URL})
if [ "$HTTP_STATUS" == "200" ] || [ "$HTTP_STATUS" == "302" ]; then
    echo "✓ Web界面可访问 (HTTP $HTTP_STATUS)"
else
    echo "✗ Web界面不可访问 (HTTP $HTTP_STATUS)"
    exit 1
fi
echo ""

# Step 2: 检查容器健康状态
echo "Step 2: 检查容器健康状态..."
HEALTH_CHECK=$(docker exec grafana curl -s http://localhost:3000/api/health)
if echo "$HEALTH_CHECK" | grep -q '"database": "ok"'; then
    echo "✓ Grafana容器健康"
    echo "  详情: $HEALTH_CHECK"
else
    echo "✗ Grafana容器不健康"
    exit 1
fi
echo ""

# Step 3: 检查ClickHouse插件安装
echo "Step 3: 检查ClickHouse插件安装..."
PLUGINS=$(docker exec grafana grafana-cli plugins ls 2>/dev/null || echo "")
if echo "$PLUGINS" | grep -q "clickhouse"; then
    echo "✓ ClickHouse插件已安装"
    echo "$PLUGINS" | grep clickhouse
else
    echo "✗ ClickHouse插件未安装"
    exit 1
fi
echo ""

# Step 4: 检查数据源配置文件
echo "Step 4: 检查数据源配置文件..."
if [ -f "/Users/jackluo/Data/duanxianxia/grafana/provisioning/datasources/clickhouse.yml" ]; then
    echo "✓ ClickHouse数据源配置文件存在"
    echo "  配置内容:"
    cat /Users/jackluo/Data/duanxianxia/grafana/provisioning/datasources/clickhouse.yml | sed 's/^/    /'
else
    echo "✗ 数据源配置文件不存在"
    exit 1
fi
echo ""

# Step 5: 检查ClickHouse连接
echo "Step 5: 检查ClickHouse连接..."
CLICKHOUSE_CONTAINER=$(docker ps --filter "name=clickhouse" --format "{{.Names}}" | head -1)
if [ -z "$CLICKHOUSE_CONTAINER" ]; then
    echo "✗ ClickHouse容器未运行"
    exit 1
fi
CLICKHOUSE_STATUS=$(curl -s http://localhost:8123/ping 2>/dev/null || echo "")
if [ "$CLICKHOUSE_STATUS" == "Ok." ]; then
    echo "✓ ClickHouse服务可访问 (容器: ${CLICKHOUSE_CONTAINER})"
else
    echo "✗ ClickHouse服务不可访问"
    exit 1
fi
echo ""

# Step 6: 测试数据库查询
echo "Step 6: 测试数据库查询..."
DB_CHECK=$(curl -s "http://localhost:8123/?query=SELECT%20name%20FROM%20system.databases%20WHERE%20name=%27duanxianxia%27" 2>/dev/null || echo "")
if echo "$DB_CHECK" | grep -q "duanxianxia"; then
    echo "✓ 数据库 'duanxianxia' 存在"
else
    echo "⚠ 数据库 'duanxianxia' 可能不存在"
fi
echo ""

# Step 7: 检查Grafana日志中的错误
echo "Step 7: 检查Grafana日志中的错误..."
ERRORS=$(docker logs grafana 2>&1 | grep -i "error" | grep -v "provisioning" | tail -5 || echo "")
if [ -z "$ERRORS" ]; then
    echo "✓ 没有发现严重错误"
else
    echo "⚠ 发现一些错误信息:"
    echo "$ERRORS" | sed 's/^/    /'
fi
echo ""

echo "=========================================="
echo "验证完成"
echo "=========================================="
echo ""
echo "重要信息:"
echo "  Grafana URL: $GRAFANA_URL"
echo "  用户名: $ADMIN_USER"
echo "  密码: $ADMIN_PASSWORD"
echo ""
echo "下一步:"
echo "  1. 在浏览器中访问: $GRAFANA_URL"
echo "  2. 使用上述凭据登录"
echo "  3. 导航到 Configuration → Data sources"
echo "  4. 点击 ClickHouse 数据源的 'Test' 按钮"
echo "  5. 验证连接成功"
echo ""
