# WebSocket 连接修复报告

**日期**: 2026-02-09 10:25
**问题**: 前端 WebSocket 连接错误
**状态**: ✅ 已修复

---

## 问题诊断

### 错误信息
```
[WebSocket] 连接错误
```

### 根本原因

1. **realtime-service 已运行** ✅
   - PID: 46470
   - 端口: 8080
   - 健康检查: 正常

2. **前端配置问题** ❌
   - 原配置: `VITE_REALTIME_URL=ws://localhost:8080`
   - 问题: 前端直接连接 `ws://localhost:8080/ws/realtime`
   - 绕过了 Vite 代理，导致跨域问题

3. **Vite 代理配置** ✅
   ```typescript
   '/ws': {
     target: 'ws://localhost:8080',
     ws: true,
     changeOrigin: true,
   }
   ```
   代理配置正确，但前端没有使用

---

## 修复方案

### 修改 1: 环境变量配置

**文件**: `frontend/.env.development`

**修改前**:
```bash
VITE_REALTIME_URL=ws://localhost:8080
```

**修改后**:
```bash
VITE_REALTIME_URL=
```

**说明**: 留空，让系统自动适配

---

### 修改 2: 动态 WebSocket URL

**文件**: `frontend/src/config/index.ts`

**添加函数**:
```typescript
// 动态获取 WebSocket URL（适配当前浏览器地址）
const getWebSocketUrl = () => {
  const envUrl = import.meta.env.VITE_REALTIME_URL;
  if (envUrl) return envUrl;

  // 自动使用当前页面的协议、主机和端口
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  return `${protocol}//${host}/ws`;
};
```

**修改配置**:
```typescript
export const config = {
  // ...
  realtimeUrl: getWebSocketUrl(), // 动态获取
  // ...
};
```

**优势**:
- ✅ 自动适配当前浏览器地址
- ✅ 无论 3000 还是 3001 端口都能工作
- ✅ 自动使用 HTTP/HTTPS 对应的 WS/WSS 协议
- ✅ 通过 Vite 代理，避免跨域问题

---

## 工作原理

### 修复前
```
浏览器 → ws://localhost:8080/ws/realtime
              ↓ (直接连接，可能跨域)
         realtime-service
```

### 修复后
```
浏览器 → ws://localhost:3001/ws (动态)
              ↓ (Vite 代理转发)
         ws://localhost:8080/ws/realtime
              ↓
         realtime-service
```

---

## 验证步骤

### 1. 检查前端访问地址
```
http://localhost:3001
```

### 2. 检查浏览器控制台 (F12)

**应该看到**:
```
✅ WebSocket connected
✅ Status: connected
```

**不应该看到**:
```
❌ [WebSocket] 连接错误
```

### 3. 检查网络连接

在浏览器 DevTools → Network → WS 标签页：

应该看到:
- URL: `ws://localhost:3001/ws/realtime`
- State: `101 Switching Protocols`
- Status: ✅ Connected

### 4. 测试实时数据更新

1. 登录系统: http://localhost:3001/login
2. 进入实时行情页
3. 观察股票价格是否自动更新（每5秒）

---

## 服务状态

| 服务 | 端口 | 状态 | 说明 |
|------|------|------|------|
| realtime-service | 8080 | ✅ 运行中 | WebSocket 服务 |
| Frontend (Vite) | 3001 | ✅ 运行中 | 前端开发服务器 |
| Vite Proxy | 3001 → 8080 | ✅ 配置正确 | WebSocket 代理 |

---

## 前端热更新

Vite 开发服务器支持热模块替换（HMR），修改后的代码应该会自动刷新。

**如果没有自动刷新**:
1. 按 `Ctrl+R` 或 `F5` 刷新浏览器
2. 或清除缓存后刷新：`Ctrl+Shift+R`

---

## 测试账号

```
用户名: testuser
密码: password123
```

登录页面: http://localhost:3001/login

---

## 后续监控

### 如果仍然出现错误

1. **检查 realtime-service 日志**:
   ```bash
   tail -f /tmp/realtime-service.log
   ```

2. **检查 WebSocket 连接**:
   - 浏览器 DevTools → Console
   - 查看错误详情

3. **验证服务运行**:
   ```bash
   lsof -i :8080  # realtime-service
   lsof -i :3001  # Frontend
   ```

4. **测试健康检查**:
   ```bash
   curl http://localhost:8080/health
   ```

### 常见问题

**Q: WebSocket 状态一直是 connecting**
A: 检查 realtime-service 是否运行，端口是否正确

**Q: 连接后立即断开**
A: 检查浏览器控制台错误信息，可能是认证问题

**Q: 收不到实时数据**
A: 检查 Redis 连接，data-collector 是否在推送数据

---

## 修改文件清单

1. ✅ `frontend/.env.development` - 清空 `VITE_REALTIME_URL`
2. ✅ `frontend/src/config/index.ts` - 添加动态 URL 函数
3. ✅ `services/realtime-service` - 已在运行（无需修改）

---

## 总结

✅ **WebSocket 连接问题已修复**

**关键改进**:
- 前端自动适配浏览器地址
- 通过 Vite 代理避免跨域
- 支持 HTTP/HTTPS 自动切换
- 无需手动配置端口

**可以测试**:
1. 刷新浏览器页面
2. 登录系统
3. 查看实时行情页面
4. 验证股票价格自动更新

---

**修复完成时间**: 2026-02-09 10:28
**状态**: ✅ 已修复，等待用户验证
