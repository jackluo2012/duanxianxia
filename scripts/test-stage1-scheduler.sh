#!/bin/bash
set -e

echo "=== 阶段1集成测试：交易日历和调度器 ==="

# 1. 编译检查
echo "1. 编译检查..."
cargo check -p trading-calendar
cargo check -p data-collector

# 2. 单元测试
echo "2. 运行单元测试..."
cargo test -p trading-calendar
cargo test -p data-collector scheduler_test

# 3. 验证配置文件
echo "3. 验证配置文件..."
if [ ! -f "config/trading-days.json" ]; then
    echo "错误: config/trading-days.json 不存在"
    exit 1
fi

# 验证JSON格式
python3 -m json.tool config/trading-days.json > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ trading-days.json 格式正确"
else
    echo "错误: trading-days.json JSON格式错误"
    exit 1
fi

# 4. 检查环境变量配置
echo "4. 检查环境变量配置..."
if [ ! -f "services/data-collector/.env.example" ]; then
    echo "错误: .env.example 不存在"
    exit 1
fi

echo "✅ .env.example 已创建"

echo ""
echo "================================"
echo "✅ 阶段1测试通过！"
echo "================================"
echo ""
echo "阶段1包含的功能："
echo "  ✅ 交易日历共享库 (trading-calendar)"
echo "  ✅ 智能调度器 (scheduler)"
echo "  ✅ 主采集循环集成"
echo "  ✅ 环境变量配置"
echo ""
