---
kind: configuration_system
name: Tauri + SvelteKit 应用配置系统
category: configuration_system
scope:
    - '**'
source_files:
    - osd-bubble/src-tauri/tauri.conf.json
    - osd-bubble/vite.config.js
    - osd-bubble/svelte.config.js
    - osd-bubble/package.json
    - osd-bubble/src-tauri/Cargo.toml
    - osd-bubble/src-tauri/src/lib.rs
    - osd-bubble/src-tauri/src/state_machine.rs
---

该项目采用 Tauri 2 + SvelteKit 架构，配置系统由多层配置文件和运行时持久化机制共同构成：

**1. 构建与打包配置**
- `osd-bubble/src-tauri/tauri.conf.json`：Tauri 核心配置，定义产品名称、版本、标识符、窗口（标题为“设置”，默认隐藏）、安全策略（CSP 为空）以及打包目标（all，包含多平台图标）。构建脚本通过 `beforeDevCommand: npm run dev` 和 `beforeBuildCommand: npm run build` 驱动前端。
- `osd-bubble/vite.config.js`：Vite 开发服务器固定端口 1420，严格端口模式，HMR 通过 `TAURI_DEV_HOST` 环境变量注入；忽略 `src-tauri` 目录监听。
- `osd-bubble/svelte.config.js`：使用 `@sveltejs/adapter-static` 以 SPA 模式运行（因 Tauri 无 Node.js 服务器），fallback 到 `index.html`。
- `osd-bubble/package.json`：声明依赖 `@tauri-apps/plugin-store`、`@tauri-apps/plugin-autostart`、`@tauri-apps/plugin-opener`、`@tauri-apps/plugin-global-shortcut` 等插件。
- `osd-bubble/src-tauri/Cargo.toml`：Rust 侧依赖包括 `tauri-plugin-store`、`tauri-plugin-autostart`、`tauri-plugin-global-shortcut`、`rdev`（全局钩子）、`windows`（Win32 API）等。

**2. 运行时状态与持久化**
- `osd-bubble/src-tauri/src/lib.rs`：通过 `tauri_plugin_store::Builder::new().build()` 启用 store 插件，提供键值对持久化能力。所有用户设置通过 tauri command 暴露给前端（如 `update_settings`、`update_position`、`update_bubble_style`、`update_exclude_apps`、`update_custom_style`、`get_custom_style`、`toggle_enabled`、`update_show_keyboard/mouse/scroll`、`update_opacity`、`apply_preset`、`update_theme`、`reset_to_defaults`）。
- `osd-bubble/src-tauri/src/state_machine.rs`：集中定义应用状态结构体 `StateMachine`，包含气泡显示时长、淡出时长、象限位置、样式、排除应用列表、自定义样式、开关选项（enabled/show_keyboard/show_mouse/show_scroll）、不透明度、主题、缩放比例等字段，并提供 `apply_preset` 和 `reset_to_defaults` 方法。

**3. 启动与快捷键配置**
- 托盘菜单动态生成“暂停/启用”、“设置”、“退出”三个菜单项。
- 注册两个全局快捷键：`Ctrl+Shift+K` 切换启用状态，`Ctrl+Shift+,` 打开设置窗口。
- 自动启动通过 `tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hide"]))` 配置。

**4. 配置约定与约束**
- 所有运行时设置通过内存中的 `STATE: Mutex<Option<StateMachine>>` 共享，未实现跨进程持久化（store 插件已初始化但未见读写调用）。
- 窗口生命周期：主窗口关闭时仅隐藏而非退出（`prevent_close()`），custom-style 窗口允许正常关闭。
- 调试模式下自动打开 DevTools（`#[cfg(debug_assertions)] window.open_devtools();`）。
- 发布模式通过 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 隐藏控制台窗口。