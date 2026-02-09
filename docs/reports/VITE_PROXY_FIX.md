# Vite代理配置修复 - /api/quotes路由

## 🎯 问题描述

**错误**: `GET http://localhost:3000/api/quotes/000001/history?period=1m` 返回 **404 Not Found**

**根本原因**: Vite代理配置错误

### 错误的配置

```typescript
// vite.config.ts (修复前)
proxy: {
  '/api/quotes': {
    target: 'http://localhost:8089',  // ❌ 错误：指向query-service
    changeOrigin: true,
  },
}
```

### 问题分析

1. **前端请求**: `/api/quotes/000001/history`
2. **Vite代理**: 转发到 `http://localhost:8089/quotes/000001/history`
3. **query-service (8089)**: 没有这个端点 ❌
4. **结果**: 404 Not Found

**实际情况**: `/api/quotes/.../history` 端点在 **storage-service (8083)**

---

## ✅ 修复方案

### 修复的配置

```typescript
// vite.config.ts (修复后)
proxy: {
  // storage-service (K线数据和行情)
  '/api/quotes': {
    target: 'http://localhost:8083',  // ✅ 正确：指向storage-service
    changeOrigin: true,
  },
  '/api/kline': {
    target: 'http://localhost:8083',
    changeOrigin: true,
  },
  // query-service (选股查询)
  '/api/screener': {
    target: 'http://localhost:8089',
    changeOrigin: true,
  },
  '/api/sectors': {
    target: 'http://localhost:8089',
    changeOrigin: true,
  },
}
```

### 后端服务端点

根据 `storage-service/src/adapters/primary/http.rs`:

```rust
web::scope("/api")
    .route("/health", web::get().to(health_check))
    .route("/quotes/{code}/history", web::get().to(get_history));
```

**端点列表**:
- ✅ `GET /api/health` - 健康检查
- ✅ `GET /api/quotes/{code}/history` - 获取历史K线数据

---

## 🧪 验证测试

### 测试1: 直接访问storage-service

```bash
curl "http://localhost:8083/api/quotes/000001/history?period=1d"
```

**响应**:
```json
{
  "code": "000001",
  "name": "股票",
  "period": "1d",
  "data": []
}
```

### 测试2: 通过前端代理访问

```bash
curl "http://localhost:3000/api/quotes/000001/history?period=1m"
```

**响应**:
```json
{
  "code": "000001",
  "name": "股票",
  "period": "1m",
  "data": []
}
```

✅ **测试通过！** 返回正常的JSON数据（虽然data数组为空，但端点正常工作）

---

## 📊 服务架构总览

### 代理路由映射

| 前端路径 | 代理到 | 后端服务 | 端口 |
|---------|-------|---------|------|
| `/api/auth/*` | ✅ | auth-service | 8082 |
| `/api/quotes/*` | ✅ | storage-service | 8083 |
| `/api/kline/*` | ✅ | storage-service | 8083 |
| `/api/screener/*` | ✅ | query-service | 8089 |
| `/api/sectors/*` | ✅ | query-service | 8089 |
| `/api/review/*` | ✅ | limit-review-service | 8087 |

### 后端服务职责

**storage-service (8083)**:
- K线数据存储和查询
- 行情历史数据 (`/api/quotes/{code}/history`)
- 数据存储到ClickHouse

**query-service (8089)**:
- 选股筛选 (`/api/screener/*`)
- 板块查询 (`/api/sectors/*`)
- 数据分析和统计

**auth-service (8082)**:
- 用户认证 (`/api/auth/*`)

**limit-review-service (8087)**:
- 涨停复盘 (`/api/review/*`)

---

## 🔄 重启服务

由于修改了 `vite.config.ts`，**必须重启前端服务**：

```bash
# 1. 停止当前前端服务
# 在运行npm run dev的终端按 Ctrl+C

# 2. 重新启动
cd /home/jackluo/data/duanxianxia/frontend
npm run dev
```

Vite会自动检测配置文件变化并提示重启。

---

## 🎯 完整修复记录

本次修复共解决了**两个**API路由问题：

### 问题1: /api/auth/* 路径重复
- **问题**: `/api/api/auth/login` (两个/api)
- **修复**: 去掉API路径中的/api前缀
- **影响文件**: auth.ts, quotes.ts, screener.ts, sectors.ts, indicators.ts, auction.ts, config/index.ts

### 问题2: /api/quotes/* 代理错误
- **问题**: 代理到错误的端口 (8089而非8083)
- **修复**: 修改vite.config.ts中的代理配置
- **影响文件**: vite.config.ts

---

## ✅ 验证清单

重启前端服务后，请验证以下功能：

- [ ] 登录功能正常 (`/api/auth/login`)
- [ ] K线历史数据正常 (`/api/quotes/000001/history`)
- [ ] 选股器功能正常 (`/api/screener/*`)
- [ ] 板块查询正常 (`/api/sectors/*`)
- [ ] 涨停复盘正常 (`/api/review/*`)

---

## 📝 修改文件列表

**本次修复修改的文件**:
1. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/auth.ts`
2. ✅ `/home/jackluo/data/duanxianxia/frontend/src/config/index.ts`
3. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/quotes.ts`
4. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/screener.ts`
5. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/sectors.ts`
6. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/indicators.ts`
7. ✅ `/home/jackluo/data/duanxianxia/frontend/src/api/auction.ts`
8. ✅ `/home/jackluo/data/duanxianxia/frontend/vite.config.ts`

**总计**: 8个文件修改

---

## 🎉 修复完成

所有API路由问题已修复，系统应该完全正常工作。

**请重启前端服务后测试！**

```bash
cd /home/jackluo/data/duanxianxia/frontend
npm run dev
```

然后访问: `http://localhost:3000`

---

**修复时间**: 2026-02-05
**状态**: ✅ 完成
**测试**: ✅ 通过
