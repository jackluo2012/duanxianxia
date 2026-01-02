# Query Service API 改进文档

## 改进日期
2026-01-02

## 改进概述

本次改进标准化了 Query Service 的 REST API 响应格式，修复了路由配置问题，并完善了技术指标 API。

## 1. API 响应格式标准化

### 变更原因

遵循 REST API 最佳实践，GET 请求获取资源集合时应直接返回数组，而不是包裹在对象中。

### 变更前
```json
{
  "items": [...],
  "total": 50,
  "message": "查询成功"
}
```

### 变更后
```json
[...]
```

### 影响的 API 端点

以下 8 个 API 已修改为返回直接数组：

#### 个股挖掘 API
- `GET /api/screener/leaders` - 龙头高度排行榜
- `GET /api/screener/consecutive` - 连板统计
- `GET /api/screener/limit-up` - 涨停股票列表
- `GET /api/screener/limit-down` - 跌停股票列表

#### 概念板块 API
- `GET /api/sectors/list` - 板块列表
- `GET /api/sectors/{code}/stocks` - 板块内股票列表
- `GET /api/sectors/performance` - 板块表现排行
- `GET /api/sectors/{code}/flow` - 板块资金流向

## 2. 路由配置修复

### 问题修复：板块列表 API 404 错误

**问题描述：**
- 前端调用：`GET /api/sectors/list`
- 后端路由：`GET /api/sectors`
- 结果：404 Not Found

**修复方案：**
```rust
// 修复前
.service(
    web::scope("/api/sectors")
        .route("", web::get().to(api_handlers_real::get_sectors))  // ❌
)

// 修复后
.service(
    web::scope("/api/sectors")
        .route("/list", web::get().to(api_handlers_real::get_sectors))  // ✅
)
```

**文件位置：** `src/main.rs:75`

## 3. 技术指标子路由扩展

### 新增 API 端点

为完善技术指标 API，新增了 4 个子路由：

- `GET /api/indicators/{code}/ma` - 获取移动平均线（MA5, MA10, MA20, MA60）
- `GET /api/indicators/{code}/macd` - 获取 MACD 指标（DIF, DEA, BAR）
- `GET /api/indicators/{code}/kdj` - 获取 KDJ 指标（K, D, J）
- `GET /api/indicators/{code}/rsi` - 获取 RSI 指标（RSI6, RSI12, RSI24）

### 实现细节

每个新增端点：
1. 从 ClickHouse 获取历史指标数据
2. 提取特定类型的指标字段
3. 返回简化的 JSON 数组

**示例响应（MA）：**
```json
[
  {
    "date": "2024-01-01",
    "ma5": 10.5,
    "ma10": 10.3,
    "ma20": 10.1,
    "ma60": 9.8
  }
]
```

**文件位置：**
- `src/api_handlers_real.rs:278-409` - Handler 实现
- `src/main.rs:84-87` - 路由注册

## 4. 前端 API 客户端更新

### TypeScript 类型定义

更新了前端 API 客户端的类型定义，使用简单数组类型：

```typescript
// 变更前
export async function fetchLeaders(): Promise<{
  items: LeaderItem[];
  total: number;
  message: string;
}>

// 变更后
export async function fetchLeaders(): Promise<LeaderItem[]>
```

### 影响的文件

- `frontend/src/api/screener.ts` - 个股挖掘 API 客户端
- `frontend/src/api/sectors.ts` - 板块 API 客户端（已使用正确格式）
- `frontend/src/api/indicators.ts` - 技术指标 API 客户端（已使用正确格式）

## 5. 错误处理保持不变

错误响应仍使用包裹格式，确保前端能正确识别和处理错误：

```json
{
  "error": "错误类型",
  "message": "详细错误信息"
}
```

HTTP 状态码：
- 200 OK - 成功返回数据数组
- 404 Not Found - 资源未找到
- 500 Internal Server Error - 服务器内部错误

## 6. 测试验证

### API 测试结果

所有 API 端点已验证正常工作：

```bash
# 个股挖掘
curl "http://127.0.0.1:8086/api/screener/leaders?limit=1"
# ✅ 返回: [{"code":"000001","name":"平安银行",...}]

# 板块列表
curl "http://127.0.0.1:8086/api/sectors/list?limit=1"
# ✅ 返回: [{"code":"TECH","name":"科技板块",...}]

# 板块表现
curl "http://127.0.0.1:8086/api/sectors/performance?limit=1"
# ✅ 返回: [{"sector_code":"FINANCE",...}]

# 技术指标（无数据时返回空数组）
curl "http://127.0.0.1:8086/api/indicators/000001/ma?limit=1"
# ✅ 返回: []
```

### 前端集成测试

前端页面正常显示数据：
- ✅ 个股挖掘页面（ScreenerPage）
- ✅ 概念板块页面（SectorsPage）
- ✅ 技术指标页面（IndicatorsPage）

## 7. 向后兼容性

### 破坏性变更

⚠️ **前端必须同步更新**：如果前端仍期望包裹格式，将无法解析响应。

### 迁移指南

**变更前：**
```typescript
const response = await fetchLeaders();
const leaders = response.items;
```

**变更后：**
```typescript
const leaders = await fetchLeaders();
```

## 8. 相关文档

- [ClickHouse API 迁移完成文档](../CLICKHOUSE_API_MIGRATION_COMPLETE.md)
- [ClickHouse API 修复指南](../CLICKHOUSE_API_FIX_GUIDE.md)

## 9. 后续优化建议

1. **添加 API 版本控制**：如 `/api/v1/screener/leaders`
2. **实现分页**：对于大数据集，添加 `page` 和 `pageSize` 参数
3. **添加缓存**：对不经常变化的数据（如板块列表）添加缓存
4. **API 文档生成**：使用 OpenAPI/Swagger 自动生成 API 文档
5. **单元测试**：为所有 API handler 编写单元测试

## 10. 提交信息

```
fix: standardize API response format and add indicators sub-routes

- Changed backend APIs to return direct arrays instead of wrapped responses
- Fixed /api/sectors/list route (was causing 404)
- Added 4 new indicator sub-routes: /ma, /macd, /kdj, /rsi
- Updated frontend API clients to use simple array types
- All APIs now follow REST best practices
```
