# 验证脚本使用说明

## 新增的验证命令

### `npm run test:rust`
运行 Rust 后端单元测试。

```bash
npm run test:rust
```

**结果**: 在 `src-tauri` 目录下执行 `cargo test`，测试状态机、按键解析等功能。

### `npm run test:all`
同时运行前端类型检查和 Rust 单元测试。

```bash
npm run test:all
```

**结果**: 
- 先运行 `svelte-check` 检查前端 TypeScript/Svelte 类型
- 然后运行 `cargo test` 测试 Rust 代码

## 测试结果

> 最近更新：2026-08-08（阶段 0 收尾验证）

### Rust 测试 (✅ 46/46 通过)

执行 `cargo test` 共 46 个用例，分布在以下模块：

#### easing.rs - 6 个测试
- ✅ 四个缓动函数（linear / ease_out_cubic / ease_in_out_quad / ease_out_back）的边界值、输入钳制、单调性与过冲特性

#### state_machine.rs - 24 个测试
- ✅ 初始状态与 FadingIn 入场动画（Idle→FadingIn、淡入中途按键跳 Visible、淡出中途按键重置、alpha 递增、入场完成转 Visible）
- ✅ tick 状态流转（Idle 返回透明、Visible 保持、超时进入 FadingOut、淡出完成回 Idle）
- ✅ 淡出 easeOutCubic 曲线（前 1/3 时长 alpha 降至 0.5 以下，先快后慢）
- ✅ enabled / 分类标志过滤、三个教学预设（classroom/recording/streaming）
- ✅ 三套主题配色预设（deep_space/cream_white/neon_blue，只改配色不改形状与时长）
- ✅ 重置默认值、完整生命周期、持久化恢复（全字段/空对象/非法 JSON/非法字段跳过/opacity 钳制）

#### hook.rs - 13 个测试
- ✅ KeyTracker 初始化、修饰键检测与状态设置
- ✅ 按键格式化（单键/Ctrl 组合/连击计数/全修饰键组合）
- ✅ 键名转换（方向键、特殊键、数字键、F 功能键）与鼠标按钮转换

#### renderer/mod.rs - 3 个测试
- ✅ 连击乘数入场动画进度函数（无 birth 即完成、边界值、单调递增）

### 前端检查 (✅ 0 errors / 0 warnings)

`svelte-check` 已清零：无类型错误、无 a11y 警告（设置界面 label 均已与控件关联，控件组使用 `role="group"` + `aria-labelledby`）。

### 前端单元测试 (✅ 6/6 通过)

- ✅ `tests/types.test.ts` (3 个) - 类型定义与默认样式常量
- ✅ `tests/CustomStyleEditor.test.ts` (3 个) - 自定义样式编辑器挂载、重置按钮、6 个控件渲染

## 注意事项

`npm run test:all` 会依次执行 `svelte-check` 与 `cargo test`，两者目前均为全绿，可作为提交前的验收门槛。

如需单独运行前端检查：
```bash
npm run check
```

如需单独运行前端单元测试：
```bash
npm run test
```
