---
kind: build_system
name: Tauri 2 + SvelteKit 构建与打包系统
category: build_system
scope:
    - '**'
source_files:
    - osd-bubble/package.json
    - osd-bubble/vite.config.js
    - osd-bubble/svelte.config.js
    - osd-bubble/src-tauri/tauri.conf.json
    - osd-bubble/src-tauri/Cargo.toml
    - osd-bubble/src-tauri/build.rs
---

本项目采用 Tauri 2 作为桌面应用框架，结合 SvelteKit（静态适配器）作为前端构建系统，通过 npm scripts 与 Cargo 协同完成开发、构建与打包流程。

**构建工具链**
- 前端：Vite 6 + SvelteKit 2，使用 `@sveltejs/adapter-static` 以 SPA 模式输出静态资源到 `build/` 目录
- 后端：Rust + Tauri 2，Cargo 管理原生依赖，`tauri-build` 在编译时生成 schema
- 版本统一：前端 `package.json` 与后端 `src-tauri/Cargo.toml`、`tauri.conf.json` 三处版本号同步为 `0.1.0`

**开发流程**
- `npm run dev`：启动 Vite 开发服务器（端口 1420），HMR 通过 `TAURI_DEV_HOST` 环境变量配置
- `npm run tauri dev`：Tauri CLI 调用 `beforeDevCommand: npm run dev`，自动加载本地前端并启动 Rust 后端
- `vite.config.js` 中固定端口 1420、严格端口模式、忽略 `src-tauri/**` 的监听，避免 Rust 代码触发前端热重载

**构建与打包流程**
- `npm run build`：Vite 构建静态前端资源至 `build/` 目录
- `npm run tauri build`：Tauri 先执行 `beforeBuildCommand: npm run build`，再编译 Rust 后端并将前端资源打包进可执行文件
- `tauri.conf.json` 中 `bundle.targets = "all"` 表示同时打包所有目标平台（当前主要为 Windows），产物包含多种图标格式（ico、icns、png）
- `src-tauri/build.rs` 仅调用 `tauri_build::build()`，无自定义构建逻辑

**关键约定**
- 前端输出目录固定为 `../build`（相对于 `src-tauri/`），由 `frontendDist` 指定
- 开发服务器端口固定为 1420，HMR 端口 1421，均硬编码于 `vite.config.js`
- Tauri 应用窗口默认隐藏（`visible: false`），仅作为设置面板存在
- CSP 策略设为 `null`，允许内联脚本与样式（适合 Tauri 嵌入静态页面）
- 未使用 Makefile、Dockerfile、CI 配置文件或自动化发布脚本，构建完全依赖 npm scripts 与 Tauri CLI

**约束与限制**
- 项目当前仅针对 Windows 平台开发（依赖 `windows` crate 的 Win32 API）
- 无跨平台交叉编译配置，无 Docker 容器化方案
- 无 CI/CD 流水线，版本管理依赖手动同步 package.json、Cargo.toml 与 tauri.conf.json