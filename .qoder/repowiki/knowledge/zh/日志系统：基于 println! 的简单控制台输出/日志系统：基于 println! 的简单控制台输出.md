---
kind: logging_system
name: 日志系统：基于 println! 的简单控制台输出
category: logging_system
scope:
    - '**'
source_files:
    - osd-bubble/src-tauri/Cargo.toml
    - osd-bubble/src-tauri/src/lib.rs
    - osd-bubble/package.json
---

该仓库未实现结构化的日志系统。Rust 后端（Tauri + SvelteKit）和前端均使用最基础的 `println!` / `console.log` 进行调试输出，没有任何日志框架、日志级别管理或结构化字段。

**Rust 后端**
- 所有日志通过 `println!` 直接输出到标准输出，如 `lib.rs` 中的 `[open_custom_style_window] 函数被调用`、`正在创建原生透明窗口...`、`监听失败: {:?}` 等。
- `Cargo.toml` 中未引入任何日志 crate（无 `tracing`、`log`、`env_logger`、`slog`、`fern`、`log4rs` 等依赖）。
- 没有统一的日志初始化代码、日志文件输出、日志级别配置或结构化字段。

**前端（SvelteKit）**
- `package.json` 中未包含任何日志相关依赖。
- 未发现 `console.log`/`console.debug`/`console.error` 等结构化前端日志的使用模式。

**结论**
该项目处于早期开发阶段，日志输出完全依赖 `println!` 调试打印，不具备生产级日志能力（无级别控制、无持久化、无结构化格式、无集中收集）。