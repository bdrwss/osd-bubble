# 贡献指南 (Contributing Guide)

感谢你关注并愿意为 **OSD Bubble** 项目做出贡献！无论是提交 Bug 报告、提出功能建议，还是贡献代码，我们都非常欢迎。

---

## 🛠️ 开发流程

### 1. Fork 并克隆仓库
```bash
git clone https://github.com/<your-username>/osd-bubble.git
cd osd-bubble/osd-bubble
```

### 2. 创建特性分支
```bash
git checkout -b feat/your-feature-name
# 或修复分支
git checkout -b fix/your-bug-fix
```

### 3. 安装依赖与本地开发
```bash
npm install
npm run tauri dev
```

### 4. 运行全套测试验证
在提交 PR 之前，请务必保证本地测试 100% 通过：
```bash
# 前端类型检查与组件测试
npm run check
npm test

# Rust 后端单元测试
cargo test --manifest-path src-tauri/Cargo.toml
```

### 5. 提交规范 (Commit Convention)
请使用简洁明了的英文 Commit 动词前缀：
- `feat:` 新功能
- `fix:` 缺陷修复
- `docs:` 文档更新
- `style:` 样式或排版调整
- `refactor:` 代码重构
- `test:` 自动化测试
- `chore:` 构建或工程依赖变更

### 6. 发起 Pull Request
在 GitHub 上向 `master` / `main` 分支提交 PR，并详细描述变更内容与验证方式。
