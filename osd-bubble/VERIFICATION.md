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

### Rust 测试 (✅ 26/26 通过)

#### state_machine.rs - 13 个测试
- ✅ test_initial_state - 验证初始状态
- ✅ test_on_key_press_transitions_to_visible - 按键事件触发可见状态
- ✅ test_tick_idle_returns_zero_opacity - Idle 状态返回透明值 0
- ✅ test_tick_visible_stays_visible - Visible 状态保持显示
- ✅ test_tick_visible_transitions_to_fading_out - 超时后进入淡出状态
- ✅ test_tick_fading_out_reaches_idle - 淡出完成后回到 Idle
- ✅ test_tick_fading_out_calculates_alpha - 淡出过程计算透明度
- ✅ test_should_show_event_respects_enabled_flag - enabled 标志控制
- ✅ test_should_show_event_respects_category_flags - 分类标志控制
- ✅ test_apply_preset_classroom - Classroom 预设配置
- ✅ test_apply_preset_unknown_ignores - 未知预设忽略
- ✅ test_reset_to_defaults - 重置到默认值
- ✅ test_full_lifecycle - 完整生命周期测试

#### hook.rs - 13 个测试
- ✅ test_key_tracker_new - KeyTracker 初始化
- ✅ test_is_modifier - 修饰键检测
- ✅ test_set_modifier - 设置修饰键状态
- ✅ test_format_current_with_no_modifiers_and_no_key - 无修饰键无主键
- ✅ test_format_current_with_single_key - 单个按键
- ✅ test_format_current_with_ctrl_and_key - Ctrl+ 按键
- ✅ test_format_current_with_repeat_count - 连击计数
- ✅ test_format_current_all_modifiers - 所有修饰键组合
- ✅ test_key_to_string_arrow_keys - 方向键转换
- ✅ test_key_to_string_special_keys - 特殊键转换
- ✅ test_key_to_string_numbers - 数字键转换
- ✅ test_key_to_string_f_keys - F 功能键转换
- ✅ test_button_to_string - 鼠标按钮转换

## 注意事项

前端类型检查 (`svelte-check`) 会显示一些现有的类型错误，这些问题不影响 Rust 测试的执行。Rust 测试是独立的且全部通过。

如需单独运行前端检查：
```bash
npm run check
```

如需单独运行前端单元测试：
```bash
npm run test
```
