---
name: osd-motion-design
description: OSD Bubble 项目的动效与视觉设计规范技能。当修改气泡渲染器（renderer/）、动画状态机（state_machine.rs）、覆盖层（overlay.rs）、缓动（easing.rs）或设置界面样式与转场（+page.svelte、CustomStyleEditor.svelte）时必须使用本技能，确保时长、缓动、配色与渲染约束符合项目规范。
---

# OSD Bubble 动效设计规范技能

本技能固化了 OSD Bubble（按键 OSD 可视化工具，Tauri 2 + Svelte 5 + Rust tiny-skia 软渲染）的动效与视觉标准。完整规范见 `docs/动效设计规范.md`，本文件列出核心约束与验收检查项。

## 触发条件

满足以下任一条件时激活本技能：

- 修改 `src-tauri/src/renderer/`、`state_machine.rs`、`overlay.rs`、`easing.rs`
- 修改 `src/routes/+page.svelte`、`src/lib/components/CustomStyleEditor.svelte` 的样式、转场、动画
- 新增气泡风格、主题配色、动画效果

## 核心时长标准

| 场景 | 数值 | 缓动 |
|---|---|---|
| 气泡入场淡入 | 120ms | easeOutCubic |
| 气泡淡出 | 280ms | easeOutCubic |
| 连击乘数入场 | 150ms（scale 0.8→1.0） | easeOutCubic |
| Tab 切换 | 入 180ms / 出 120ms | cubicOut |
| 模态对话框 | 入 scale 200ms（起点 0.95）/ 出 fade 150ms | cubicOut |
| Toast | 入 fly 200ms / 出 fade 150ms | cubicOut |
| 卡片微交互 | 150ms | ease-out |

## 核心规则（不可违反）

1. **退场/淡出一律 easeOut**，禁止线性淡出；Rust 用 `easing::ease_out_cubic`，前端用 `svelte/easing` 的 `cubicOut`
2. **动画不得阻塞响应**：入场（FadingIn）期间再次按键直接跳 `Visible` 态重置计时
3. 连击乘数递增不重播入场动画（`MULTIPLIER_BIRTH` 保留首次时间戳，消失才清空）
4. 气泡最终 alpha = 动画 alpha × 全局 opacity（0.4–1.0）
5. 主题配色预设（deep_space/cream_white/neon_blue）只换配色，不改气泡形状与时间参数；前后端预设 ID 必须一致
6. `CustomStyle` 必须完整 7 字段（bg_color/bg_opacity/text_color/border_color/border_width/radius/shadow_color），缺字段会反序列化失败回退默认
7. 新增配色文字对比度 ≥ WCAG AA（4.5:1）
8. 前端位移/缩放类微交互必须提供 `prefers-reduced-motion` 降级与 `:focus-visible` 焦点环

## tiny-skia 渲染路径约束（重要，易踩坑）

1. **CPU 软渲染无 blur filter**：软阴影用多层同心圆角矩形模拟（外扩 2/4/6/8px、alpha 40/25/15/8），画布四周预留 12px padding，否则被裁切
2. `Paint.shader` 是**公有字段**直接赋值，没有 `set_shader` 方法；`GradientStop::new(f32, Color)` 接收**非预乘** Color
3. 缩放文字 = 调整字号 + 基线锚点偏移（`scaled_y = text_y - (24.0 - font_size) * 0.7`），无逐字形变换
4. 单帧渲染预算 < 8ms；tick 线程 16ms 投递 `WM_TICK`（60fps）
5. `Visible` 稳态 `tick()` 返回 `needs_redraw=false`——若有新动画需要稳态重绘，必须在 `overlay.rs` wndproc 中显式强制（参考乘数动画的实现）
6. pixmap 为 RGBA，写 DIBSection 前必须手动转 BGRA，经 `UpdateLayeredWindow` + `AC_SRC_ALPHA` 上屏

## 设置界面数据流约束

1. 滑块/开关变更走响应式块：invoke 后端 100ms 防抖，持久化（settings.json）700ms 防抖，**禁止每次 input 直接写盘**
2. 持久化键 `osdBubbleSettings`（tauri-plugin-store），后端经 `apply_persisted_settings` 逐字段防御式恢复，新增字段须补充非法值跳过测试

## 验收检查清单

修改完成后逐项确认：

- [ ] 时长/缓动符合上表；若调整数值，同步更新 `docs/动效设计规范.md`
- [ ] `cargo test` 全部通过（easing/state_machine/renderer 均有回归测试，新动画须补进度/边界测试）
- [ ] `cargo clippy` 无新增警告
- [ ] `npm run check` 零错误、`npm run test` 通过
- [ ] 连击 10 键无掉帧、入场动画不阻塞响应
- [ ] reduced-motion 与 focus-visible 无障碍项未被破坏
