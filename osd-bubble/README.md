# 按键 OSD 可视化工具

一个用于显示键盘、鼠标和滚轮操作的桌面覆盖工具（OSD Bubble），可在屏幕上可视化显示用户的按键和鼠标操作记录。

## 平台限制

- **仅限 Windows**：本项目依赖 Windows Win32 API，目前仅支持在 Windows 系统上运行。
- 最低要求：Windows 10 (64 位)

## 平台边界声明

本项目的平台支持范围已通过以下配置文件明确声明：

- `tauri.conf.json` 的 `bundle.targets` 设置为 `["nsis"]`
- `Cargo.toml` 的 `windows` 依赖已移至 `[target.'cfg(windows)'.dependencies]` 条件段
- Tauri 的 `devtools` 功能通过 `debug-devtools` feature 控制（仅在 debug 构建时启用）

## 快速开始

### 开发运行

```bash
npm run tauri dev
```

启动 Tauri 开发环境，自动运行前端开发和 Rust 后端构建。

### 构建发布版本

```bash
npm run build
```

编译前端资源（`vite build`）并打包成发布版本。

### 代码检查

```bash
npm run check
```

执行 TypeScript 和 Svelte 语法检查，确保代码质量。

## 技术架构

本项目采用 Tauri + SvelteKit + TypeScript 构建，前端使用 Svelte 5，后端使用 Rust。

### 核心模块

#### 🎯 `src-tauri/src/hook.rs` —— 事件钩子模块

负责监听和操作系统的底层输入事件：
- 键盘按键按下/释放（包括组合键 Ctrl、Alt、Shift、Win）
- 鼠标左键/右键/中键点击
- 滚轮滚动事件
- 连击检测与合并（500ms 内的重复按键自动合并）
- 5 秒超时重置机制，防止修饰键被系统吞没

#### ⚙️ `src-tauri/src/state_machine.rs` —— 状态机模块

管理气泡显示的生命周期和行为逻辑：
- 三种状态：Idle（空闲）、Visible（可见）、FadingOut（渐隐）
- 可配置显示时长（默认 1200ms）和渐隐时长（默认 280ms）
- 事件分类过滤：支持单独控制键盘、鼠标、滚轮的显示开关
- 预设配置：课堂模式、录屏模式、直播模式
- 自定义样式管理：颜色、透明度、边框、圆角、阴影等

#### 🖼️ `src-tauri/src/overlay.rs` —— 叠加层模块

负责创建和管理屏幕上的透明窗口：
- 使用 Windows Win32 API 创建 Layered Window（分层窗口）
- 支持 Per-Monitor DPI V2，保证高分屏不模糊
- 实现渐变透明效果（Alpha 合成）
- 智能位置调整：根据屏幕工作区自动选择左右上下方位
- 持续置顶（Topmost），防止被其他窗口覆盖
- 透传点击事件（Transparent），不影响用户交互

#### 🎨 `src-tauri/src/renderer.rs` —— 渲染器模块

负责绘制气泡的视觉内容（待实现）：
- 基于像素图的离屏渲染
- 多种主题样式：默认、3D 按键、复古终端等
- 自定义样式支持
- RGBA 到 BGRA 的颜色空间转换

## 模块职责地图

| 模块 | 职责 | 关键技术 |
|------|------|----------|
| **hook** | 输入事件捕获与解析 | rdev crate, 连击检测 |
| **state_machine** | 生命周期管理与配置 | 状态机模式，时间片调度 |
| **overlay** | 全屏透明窗口绘制 | Win32 API, Layered Window, GDI |
| **renderer** | 视觉内容生成 | 像素操作，颜色空间转换 |

## 规划文档

详细的项目设计文档请参考父目录中的以下规范：

- 📄 [功能概念定义文档](../按键 OSD 可视化工具 - 功能概念定义文档.md) - 核心功能定义与范围说明
- 🔍 [竞品调研报告](../按键 OSD 可视化工具 - 竞品调研报告.md) - 市场分析与技术调研
- 🛠️ [技术选型对比分析](../按键 OSD 可视化工具 - 技术选型对比分析.md) - 技术方案评估与决策
- 🎨 [交互规格说明书](../按键 OSD 可视化工具 - 交互规格说明书.md) - UI/UX 设计规范与交互细节

## 开发环境设置

### 前置要求

- Node.js >= 18.0.0（仅 Windows 平台）
- Rust 1.75.0+（通过 rust-toolchain.toml 自动管理）
- Windows 10 (64 位) 或更高版本

### 安装依赖

```bash
npm install
```

### 开发运行

```bash
# 默认 debug 构建（启用 devtools）
npm run tauri dev

# 如需禁用 devtools，可修改 Cargo.toml 中的 features 配置
```

### 构建发布版本

#### Release 构建（生产环境）

```bash
npm run tauri build
```

生产构建默认不包含 devtools，以获得更小的打包体积和更好的性能。

#### Windows-only 特性说明

由于项目使用 `cargo-tauri` 进行构建，以下配置确保仅在 Windows 平台上编译：

- `windows` crate 通过 `[target.'cfg(windows)'.dependencies]` 声明
- Tauri 打包目标限制为 NSIS (`bundle.targets = ["nsis"]`)
- macOS 和 Linux 平台将无法完成构建（缺少必要依赖）

## 代码检查

```bash
npm run check
```

执行 TypeScript 和 Svelte 语法检查，确保代码质量。

## 测试

### 运行前端测试

```bash
npm test
```

### 运行 Rust 后端测试

```bash
npm run test:rust
```

### 运行所有测试

```bash
npm run check; npm run test:rust
```

## 许可证

MIT License
