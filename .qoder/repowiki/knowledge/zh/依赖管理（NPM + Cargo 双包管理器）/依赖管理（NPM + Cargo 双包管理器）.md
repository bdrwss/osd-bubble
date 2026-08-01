---
kind: dependency_management
name: 依赖管理（NPM + Cargo 双包管理器）
category: dependency_management
scope:
    - '**'
source_files:
    - osd-bubble/package.json
    - osd-bubble/package-lock.json
    - osd-bubble/src-tauri/Cargo.toml
    - osd-bubble/src-tauri/Cargo.lock
---

本项目采用前后端分离的双语言栈，依赖管理由两套独立的包管理系统共同完成：前端使用 npm（Node.js），后端 Rust 核心使用 Cargo。两套系统各自维护声明式清单与锁定文件，未使用私有仓库或 vendoring 策略。

**1. 前端依赖（npm）**
- 声明文件：`osd-bubble/package.json`，定义运行时依赖 `@tauri-apps/api`、`@tauri-apps/plugin-autostart`、`@tauri-apps/plugin-opener`、`@tauri-apps/plugin-store`，以及开发依赖 SvelteKit、Vite、TypeScript 等。
- 锁定文件：`osd-bubble/package-lock.json`（lockfileVersion 3），通过 npm 生成并随代码提交，确保构建可重现。
- 版本策略：运行时依赖普遍使用 `^` 语义化版本前缀（如 `^2`、`^2.5.1`、`^2.4.4`），允许次/补丁版本自动升级；开发依赖中 TypeScript 使用 `~5.6.2` 精确到小版本。
- 包源：默认指向 `https://registry.npmjs.org`，未发现 `.npmrc` 或私有 registry 配置。
- 脚本命令：`dev`、`build`、`preview`、`check`、`tauri` 等统一通过 `npm run` 调用。

**2. 后端依赖（Cargo/Rust）**
- 声明文件：`osd-bubble/src-tauri/Cargo.toml`，核心依赖包括 `tauri 2`（启用 tray-icon、devtools 特性）、`serde`、`serde_json`、`rdev`、`tiny-skia`、`windows 0.62.2`、`rusttype`、`lazy_static`，以及多个 `tauri-plugin-*`（store、autostart、global-shortcut）。
- 锁定文件：`osd-bubble/src-tauri/Cargo.lock`（version 4），由 Cargo 自动生成，记录所有依赖的精确版本与 checksum，来源均为 `registry+https://github.com/rust-lang/crates.io-index`。
- 版本策略：主要依赖使用宽松版本号（如 `"2"`、`"1"`、`"0.5.3"`），部分平台相关 crate 显式指定 feature 集合。
- 包源：默认 crates.io，未发现 `Cargo.config` 或私有源配置。

**3. 架构与约定**
- 前后端依赖解耦：前端仅通过 `@tauri-apps/*` API 与 Rust 后端通信，不直接依赖 Rust crate。
- 锁定文件纳入版本控制：`package-lock.json` 与 `Cargo.lock` 均随代码提交，保证团队与 CI 环境一致。
- 无 vendoring：未使用 `node_modules` 快照或 `cargo vendor`，依赖从远程注册表下载。
- 无私有仓库：未发现 `.npmrc`、`config.toml` 中的认证或镜像配置。

**4. 约束与规范**
- 前端依赖必须通过 `package.json` 声明，禁止在源码中硬编码路径。
- Rust 依赖必须通过 `Cargo.toml` 声明，并通过 `features` 按需启用能力。
- 锁定文件变更需随依赖更新一并提交，以确保可重现构建。
- 未发现自动化依赖更新工具（如 Dependabot、Renovate）的配置。