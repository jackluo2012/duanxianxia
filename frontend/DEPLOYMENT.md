# 部署指南

本文档介绍如何部署短线侠前端应用。

## 目录

- [Docker部署（推荐）](#docker部署推荐)
- [传统部署](#传统部署)
- [环境配置](#环境配置)
- [生产环境优化](#生产环境优化)

---

## Docker部署（推荐）

### 前置要求

- Docker 20.10+
- Docker Compose 2.0+

### 快速启动

```bash
# 1. 构建并启动容器
./scripts/docker-deploy.sh

# 或者使用docker-compose
docker-compose up -d

# 2. 查看日志
docker-compose logs -f

# 3. 停止容器
docker-compose down
```

### 访问应用

- **本地访问**: http://localhost:3000
- **容器IP**: http://<container-ip>:3000

### Docker镜像信息

- **镜像名称**: frontend-frontend:latest
- **镜像大小**: ~83MB
- **基础镜像**: nginx:alpine
- **构建方式**: 多阶段构建（Node.js + Nginx）

### 高级用法

```bash
# 仅构建镜像
docker-compose build

# 查看容器状态
docker-compose ps

# 重启容器
docker-compose restart

# 查看资源使用
docker stats frontend-frontend
```

---

## 传统部署

### 前置要求

- Node.js 18+
- npm 9+

### 部署步骤

```bash
# 1. 安装依赖并构建
./scripts/deploy.sh

# 或者手动执行
npm ci
npm run build

# 2. 使用Web服务器托管dist目录
# 例如：nginx、apache、caddy等
```

### Nginx配置示例

```nginx
server {
    listen 80;
    server_name your-domain.com;

    root /path/to/dist;
    index index.html;

    # SPA路由支持
    location / {
        try_files $uri $uri/ /index.html;
    }

    # 静态资源缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # API代理
    location /api/ {
        proxy_pass http://backend:8089;
        proxy_set_header Host $host;
    }
}
```

---

## 环境配置

### 环境变量

复制环境变量模板并根据需要修改：

```bash
cp .env.example .env
```

### 主要配置项

```bash
# API服务地址
VITE_API_BASE_URL=http://localhost:8089
VITE_STORAGE_URL=http://localhost:8083
VITE_REALTIME_URL=ws://localhost:8090

# 功能开关
VITE_ENABLE_MOCK=false
VITE_ENABLE_WS=true

# 应用配置
VITE_REQUEST_TIMEOUT=10000
VITE_WS_RECONNECT_INTERVAL=3000
VITE_WS_HEARTBEAT_INTERVAL=30000

# 图表配置
VITE_CHART_SAMPLING_KLINE=1000
VITE_CHART_SAMPLING_MINUTE=500

# 刷新配置
VITE_AUTO_REFRESH_INTERVAL=30000
```

---

## 生产环境优化

### 1. 代码分割

已配置vendor chunk分割：

- `react-vendor`: React核心库
- `antd-vendor`: Ant Design组件库
- `charts-vendor`: ECharts图表库
- `query-vendor`: React Query

### 2. 静态资源缓存

- **CSS/JS文件**: 文件名哈希，永久缓存
- **图片字体**: 1年缓存
- **HTML文件**: 不缓存

### 3. Gzip压缩

Nginx自动压缩以下类型：

- text/html
- text/css
- application/json
- application/javascript
- image/svg+xml

### 4. 安全头

已配置的安全响应头：

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
```

### 5. 性能监控

#### 构建产物大小

```
dist/index.html                            0.79 kB │ gzip:   0.39 kB
dist/assets/index-v8LMYla8.css             0.17 kB │ gzip:   0.17 kB
dist/assets/query-vendor-CIcX5YZi.js      34.51 kB │ gzip:  10.08 kB
dist/assets/index-CuxemeYy.js            140.82 kB │ gzip:  44.37 kB
dist/assets/react-vendor-DyOhWI-L.js     158.52 kB │ gzip:  51.62 kB
dist/assets/antd-vendor-CngTDhP1.js    1,027.73 kB │ gzip: 311.90 kB
dist/assets/charts-vendor-D1pkpC7f.js  1,041.33 kB │ gzip: 338.34 kB
```

---

## 常见问题

### Q: Docker构建失败？

**A**: 检查Docker版本和网络连接：

```bash
docker --version
docker-compose --version
ping registry-1.docker.io
```

### Q: 端口冲突？

**A**: 修改docker-compose.yml中的端口映射：

```yaml
ports:
  - "3000:80"  # 改为 "8080:80"
```

### Q: API请求失败？

**A**: 检查环境变量配置和网络连接：

```bash
# 检查后端服务是否运行
curl http://localhost:8089/health

# 检查nginx代理配置
docker-compose exec frontend cat /etc/nginx/conf.d/default.conf
```

### Q: WebSocket连接失败？

**A**: 确保WebSocket代理配置正确：

```nginx
location /ws {
    proxy_pass http://backend:8090;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

---

## 监控和日志

### 查看应用日志

```bash
# Docker日志
docker-compose logs -f frontend

# Nginx访问日志
docker-compose exec frontend tail -f /var/log/nginx/access.log

# Nginx错误日志
docker-compose exec frontend tail -f /var/log/nginx/error.log
```

### 健康检查

```bash
# 检查容器状态
docker-compose ps

# 检查应用响应
curl http://localhost:3000
```

---

## 更新部署

### Docker更新

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 重新构建镜像
docker-compose build

# 3. 重启容器
docker-compose up -d

# 4. 清理旧镜像
docker image prune -f
```

### 传统部署更新

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 重新构建
npm ci
npm run build

# 3. 重启Web服务器
sudo systemctl reload nginx
```

---

## 备份和恢复

### 备份

```bash
# 备份构建产物
tar -czf frontend-backup-$(date +%Y%m%d).tar.gz dist/

# 备份Docker镜像
docker save frontend-frontend:latest | gzip > frontend-image-$(date +%Y%m%d).tar.gz
```

### 恢复

```bash
# 恢复构建产物
tar -xzf frontend-backup-YYYYMMDD.tar.gz

# 恢复Docker镜像
gunzip -c frontend-image-YYYYMMDD.tar.gz | docker load
```

---

## 技术支持

如有问题，请查看：

- [项目README](./README.md)
- [前端开发总结](./docs/FRONTEND_COMPLETION_SUMMARY.md)
- [最终完成报告](./docs/FINAL_COMPLETION_REPORT.md)

---

**最后更新**: 2026-02-04
