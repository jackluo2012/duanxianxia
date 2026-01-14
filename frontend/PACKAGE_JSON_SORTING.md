# package.json 依赖项排序说明

## 问题描述

在执行 Task 0 (添加龙头高度页面) 时,`package.json` 中的依赖项顺序发生了变化。

## 原因说明

这是 **npm 的正常自动行为**,不是人为的额外修改。

从 npm v7 开始,当你运行 `npm install <package>` 命令时,npm 会自动按字母顺序重新排序 `package.json` 中的 `dependencies` 和 `devDependencies` 字段。

### 示例

安装前:
```json
"dependencies": {
  "react": "^18.2.0",
  "react-dom": "^18.2.0",
  "antd": "^5.12.0",
  "@ant-design/pro-components": "^2.6.4"
}
```

运行 `npm install @tanstack/react-query` 后,npm 会自动排序为:

```json
"dependencies": {
  "@ant-design/pro-components": "^2.6.4",
  "@tanstack/react-query": "^5.17.0",
  "antd": "^5.12.0",
  "react": "^18.2.0",
  "react-dom": "^18.2.0"
}
```

## 当前项目使用的 npm 版本

```bash
npm --version
# 11.6.1
```

## 建议

1. **接受这个行为**: npm 的自动排序有助于保持依赖项的一致性和可读性
2. **团队规范**: 在项目贡献指南中说明,npm install 会自动排序依赖项
3. **代码审查**: 在审查 package.json 变更时,关注实际添加/删除的依赖,而非顺序变化

## 相关提交

- 提交 SHA: `638d1b0ac4e1a3895c84edccd9559475a2f61549`
- 提交信息: `feat: 添加龙头高度页面路由和菜单项`

## 实际添加的依赖

在 Task 0 中实际添加的新依赖:
- `@tanstack/react-query`: ^5.17.0
- `react-window`: ^1.8.10
- `use-debounce`: ^9.0.4
- `@types/react-window`: ^1.8.8 (devDependency)

这些依赖项的添加符合任务需求,用于实现虚拟滚动和性能优化。
