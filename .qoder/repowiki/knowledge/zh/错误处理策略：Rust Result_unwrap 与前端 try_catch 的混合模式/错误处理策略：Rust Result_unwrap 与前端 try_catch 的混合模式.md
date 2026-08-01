---
kind: error_handling
name: 错误处理策略：Rust Result/unwrap 与前端 try/catch 的混合模式
category: error_handling
scope:
    - '**'
source_files:
    - osd-bubble/src-tauri/src/lib.rs
    - osd-bubble/src-tauri/src/overlay.rs
    - osd-bubble/src-tauri/src/hook.rs
    - osd-bubble/src/routes/+page.svelte
---

该项目的错误处理采用 Rust 后端与 SvelteKit 前端协作的模式，但未建立统一的错误类型体系或全局异常处理机制。

**后端（Rust/Tauri）：**
- 大部分命令函数直接操作 `Mutex` 锁并调用 `.unwrap()` 获取状态，未对锁失败进行优雅降级，一旦互斥锁被永久持有会导致后续调用全部崩溃。
- 异步命令如 `open_custom_style_window`、`close_custom_style_window` 返回 `Result<(), String>`，通过 `map_err` 将错误转为字符串后由 Tauri 框架向上抛出，前端通过 `.catch()` 捕获。
- 关键初始化路径使用 `expect("未能接收到窗口句柄")` 在通道接收失败时直接 panic，属于不可恢复的错误场景。
- 全局钩子监听线程中 `rdev::listen` 的 Err 分支仅打印日志 `println!("监听失败: {:?}", error)`，不中断主流程。
- 所有 `#[tauri::command]` 函数均无显式错误传播，内部状态访问失败时通过静默忽略或 unwrap 崩溃两种极端方式处理。

**前端（SvelteKit）：**
- 所有通过 `invoke()` 调用的后端命令均包裹 `try/catch` 或 `.catch()`，错误统一通过 `console.error` 输出，部分场景辅以 `alert()` 提示用户。
- 版本获取等可选操作在 catch 分支提供默认值回退（如 `appVersion = "0.1.0"`），体现容错设计。
- 自定义样式窗口打开失败时同时记录错误日志和弹窗告警，形成双重反馈。

**架构约束：**
- 项目未定义自定义错误类型（无 `thiserror`/`anyhow` 依赖），错误以字符串形式在 Tauri IPC 层传递。
- 未使用 `panic!` 作为业务逻辑控制流，仅在初始化阶段用于致命错误。
- 无全局错误中间件或拦截器，错误处理分散在各调用点，缺乏一致性。
- 前端与后端的错误语义不对等：后端用 `Result<String>` 表示可恢复错误，前端用通用 `Error` 对象，缺少结构化错误码。