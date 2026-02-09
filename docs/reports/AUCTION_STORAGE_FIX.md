# auction-storage 服务路由修复报告

## 🎯 问题发现

**问题**: `/api/auction/rankings` 返回404 Not Found

**根本原因**: 路由配置中有重复的 `/api` 前缀

```rust
// 当前配置
web::scope("/api")
    .service(rankings)  // #[get("/api/auction/rankings")]

// 实际路径 = /api + /api/auction/rankings = /api/api/auction/rankings ❌
```

---

## ✅ 修复方案

### 方案1: 修改路由为相对路径（推荐）

将所有路由定义从绝对路径改为相对路径：

**修改前**:
```rust
#[get("/api/auction/rankings")]
#[get("/api/auction/details/{code}")]
#[post("/api/auction/alerts")]
#[get("/api/auction/alerts")]
#[get("/api/auction/alerts/history")]
#[post("/api/auction/watchlist")]
#[get("/api/auction/watchlist")]
#[get("/api/auction/watchlist/{code}/check")]
```

**修改后**:
```rust
#[get("/auction/rankings")]
#[get("/auction/details/{code}")]
#[post("/auction/alerts")]
#[get("/auction/alerts")]
#[get("/auction/alerts/history")]
#[post("/auction/watchlist")]
#[get("/auction/watchlist")]
#[get("/auction/watchlist/{code}/check")]
```

### 方案2: 移除 /api scope（不推荐）

如果不想修改路由定义，可以移除 scope：

```rust
// 修改前
cfg.service(
    web::scope("/api")
        .service(...)
);

// 修改后
cfg.service(...)  // 直接在根路径下注册
```

但这会导致其他端点（如 `/health`）也需要修改路径。

---

## 🔧 修复步骤

### 步骤1: 修改源代码

文件: `/home/jackluo/data/duanxianxia/services/auction-storage/src/adapters/primary/http.rs`

需要修改的行:
- 第57行: `#[get("/api/auction/rankings")]` → `#[get("/auction/rankings")]` ✅ 已完成
- 第106行: `#[get("/api/auction/details/{code}")]` → `#[get("/auction/details/{code}")]`
- 第148行: `#[post("/api/auction/alerts")]` → `#[post("/auction/alerts")]`
- 第174行: `#[get("/api/auction/alerts")]` → `#[get("/auction/alerts")]`
- 第201行: `#[get("/api/auction/alerts/history")]` → `#[get("/auction/alerts/history")]`
- 第244行: `#[post("/api/auction/watchlist")]` → `#[post("/auction/watchlist")]`
- 第298行: `#[get("/api/auction/watchlist")]` → `#[get("/auction/watchlist")]`
- 第312行: `#[get("/api/auction/watchlist/{code}/check")]` → `#[get("/auction/watchlist/{code}/check")]`

### 步骤2: 重新编译

```bash
cd /home/jackluo/data/duanxianxia
cargo build --release -p auction-storage
```

### 步骤3: 重启服务

```bash
# 杀掉旧进程
killall -9 auction-storage

# 启动新版本
nohup ./target/release/auction-storage > logs/auction-storage.log 2>&1 &
```

### 步骤4: 验证

```bash
# 测试health端点
curl http://localhost:8084/api/health

# 测试rankings端点
curl "http://localhost:8084/api/auction/rankings?type=change&limit=5"

# 通过前端代理测试
curl "http://localhost:3000/api/auction/rankings?type=change&limit=5"
```

---

## 🧪 当前状态

**已完成的修复**:
- ✅ 启动 auction-storage 服务（8084端口）
- ✅ 修改 Vite 代理配置（`/api/auction` → `http://localhost:8084`）
- ✅ 修改第57行路由定义

**待完成的修复**:
- ⏳ 修改其余7个路由定义
- ⏳ 重新编译服务
- ⏳ 重启并验证

---

## 📝 临时解决方案

如果需要立即让前端工作，可以暂时修改前端API路径：

**文件**: `/home/jackluo/data/duanxianxia/frontend/src/api/auction.ts`

```typescript
// 临时修改 - 添加双重 /api 前缀
export async function fetchAuctionRankings(...) {
  return request.get(
    `/api/api/auction/rankings?type=${rankingType}&limit=${limit}`
  );
}
```

但这只是临时方案，不推荐使用。

---

## 🎯 预期结果

修复后，所有竞价API端点应该正常工作：

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/auction/rankings` | GET | 竞价排行榜 |
| `/api/auction/details/{code}` | GET | 竞价详情 |
| `/api/auction/alerts` | POST | 创建告警规则 |
| `/api/auction/alerts` | GET | 获取告警规则 |
| `/api/auction/alerts/history` | GET | 告警历史 |
| `/api/auction/watchlist` | POST | 添加到自选 |
| `/api/auction/watchlist` | GET | 获取自选列表 |
| `/api/auction/watchlist/{code}/check` | GET | 检查是否自选 |

---

## 📊 服务架构

```
前端 (localhost:3000)
  ↓ /api/auction/*
Vite 代理
  ↓ http://localhost:8084/api/auction/*
auction-storage (8084端口)
  ↓
实际路由: /auction/rankings (在 /api scope下)
  ↓
完整路径: /api + /auction/rankings = /api/auction/rankings ✅
```

---

**状态**: 🟡 部分修复完成（1/8路由）
**下一步**: 完成剩余路由修改并重新编译
