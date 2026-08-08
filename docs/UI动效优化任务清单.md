# UI 界面优化与动效设计 — 原子级任务清单

> 版本：v1.1（完结版） | 日期：2026-08-08
> 来源：基于四项落地建议拆解（缓动曲线升级 / 设置界面动效 / 气泡视觉升级 / 技能沉淀）
> 总工作量：约 11.5 人日 | 任务总数：16

> ✅ **完结状态（2026-08-08 核实）**：以下 16 个任务已全部落地并通过验证（`cargo test` 46/46、`svelte-check` 0 errors / 0 warnings、前端单测 6/6）。
> 各任务完成证据：
> - T1.1–T1.4：`src-tauri/src/easing.rs`；`state_machine.rs` 的 `FadingIn` 状态、easeOutCubic 淡出、`fade_in_duration` 持久化
> - T2.1–T2.5：`+page.svelte` 的 Tab/模态/Toast 转场、预设卡片微交互（含 `prefers-reduced-motion`）、滑块实时预览 invoke 通道
> - T3.1–T3.4：`renderer/mod.rs` 的 `SHADOW_LAYERS` 软阴影、`LinearGradient` 渐变、三套主题配色预设、连击乘数入场动画
> - T4.1–T4.2：`docs/动效设计规范.md`、`.qoder/skills` 的 `osd-motion-design` 技能
> 后续开发方向见《开发计划-V1.1》（规格差异收敛与 V1.1 功能增量），本文档仅作为历史任务基线保留。

---

## 关键代码事实（任务设计依据，为 v1.0 时点快照）

- ~~当前只有淡出动画（`FadingOut`），气泡是**瞬间出现**的，无入场动画~~ → 已新增 `FadingIn` 入场状态（T1.3）
- ~~`state_machine.rs` `tick()` 中注释写 "ease-out fade" 但实际是**线性**衰减~~ → 已改为真正的 easeOutCubic（T1.2）
- `+page.svelte` 已 import `slide/fade/scale`，说明部分转场已有基础，任务侧重补齐与统一
- ~~tiny-skia 渲染器无阴影模糊、无渐变~~ → 已实现多层软阴影与 LinearGradient 渐变（T3.1/T3.2）

---

## 建议一：Rust 侧缓动曲线升级（优先级 P0，收益最高）

### T1.1 提取缓动函数模块 `easing`

| 项目 | 内容 |
|---|---|
| 实现目标 | 在 `src-tauri/src/` 新建 `easing.rs`（或在 state_machine 内建），实现 `linear`、`ease_out_cubic`、`ease_in_out_quad`、`ease_out_back` 四个纯函数，签名 `fn ease(progress: f32) -> f32`，输入输出均在 `[0,1]` |
| 技术要求 | 无新依赖，纯数学实现；`ease_out_cubic: 1 - (1-t)^3`；`ease_out_back` 带轻微过冲（c1=1.70158） |
| 预期效果 | 为后续所有动画提供统一缓动基础设施 |
| 验收标准 | ① 每个函数有单元测试覆盖边界（t=0→0、t=1→1）与单调性；② `cargo test` 全部通过；③ clippy 无警告 |
| 工作量 | 0.5 人日 |
| 依赖 | 无 |

### T1.2 修复淡出曲线（线性 → easeOutCubic）

| 项目 | 内容 |
|---|---|
| 实现目标 | 将 `state_machine.rs` `tick()` 中 FadingOut 分支的 `alpha = 1.0 - progress` 改为 `alpha = 1.0 - ease_out_cubic(progress)` |
| 技术要求 | 不改变 `fade_duration`（默认 280ms）与状态机结构；同步更新误导性的 "ease-out fade" 注释 |
| 预期效果 | 淡出先快后慢，消失更自然，无"戛然而止"感 |
| 验收标准 | ① 现有 15 个单元测试全部通过（`test_tick_fading_out_calculates_alpha` 仍断言 `0 < alpha < 1` 不受影响）；② 手动录屏逐帧对比：前 1/3 时长 alpha 下降幅度 > 50%；③ 运行时无卡顿 |
| 工作量 | 0.5 人日 |
| 依赖 | T1.1 |

### T1.3 新增入场淡入动画（FadingIn 状态）

| 项目 | 内容 |
|---|---|
| 实现目标 | 在 `BubbleState` 枚举新增 `FadingIn { start: Instant }`；`on_key_press()` 从 `Idle` 进入 `FadingIn` 而非直接 `Visible`；`tick()` 中 FadingIn 阶段 alpha 按 `ease_out_cubic(progress)` 从 0 升到 1，时长固定 120ms |
| 技术要求 | 若 FadingIn 期间再次按键，直接跳 `Visible`（重置计时），保证连击响应不延迟；`Visible { start }` 语义不变 |
| 预期效果 | 气泡柔和出现，消除瞬间闪现的突兀感 |
| 验收标准 | ① 新增 ≥4 个单元测试（入场中途按键、入场完成转 Visible、入场 alpha 递增）；② 手动测试：连按 5 键无延迟感；③ 入场动画期间气泡可正常重绘 |
| 工作量 | 1 人日 |
| 依赖 | T1.1 |

### T1.4 动画时长纳入可配置项（可选增强）

| 项目 | 内容 |
|---|---|
| 实现目标 | `CustomStyle` 或顶层设置新增 `fade_in_duration`（默认 120ms），走现有 `apply_persisted_settings` 防御式解析模式 |
| 技术要求 | 沿用现有逐字段解析模式（缺失/非法保持默认）；前端暂不暴露 UI，预留字段 |
| 预期效果 | 为后续设置界面开放动画调节做准备 |
| 验收标准 | ① 新增持久化往返测试（含非法值跳过用例，对齐现有 `test_apply_persisted_settings_invalid_fields_skipped` 模式）；② 旧配置文件升级后不崩溃 |
| 工作量 | 0.5 人日 |
| 依赖 | T1.3 |

---

## 建议二：设置界面动效（优先级 P1）

### T2.1 Tab 切换转场

| 项目 | 内容 |
|---|---|
| 实现目标 | `+page.svelte` 中 `currentTab` 切换时内容区应用 `{#key currentTab}` + `transition:fade\|slide`（180ms ease-out） |
| 技术要求 | 使用已 import 的 `svelte/transition`，不引入新依赖；转场期间不阻塞控件交互 |
| 预期效果 | Tab 切换平滑，无硬切换闪烁 |
| 验收标准 | ① 三个 Tab 间两两切换均平滑；② 快速连续点击 Tab 无动画残留/错乱；③ `npm run check` 通过 |
| 工作量 | 0.5 人日 |
| 依赖 | 无 |

### T2.2 模态层出入场动画

| 项目 | 内容 |
|---|---|
| 实现目标 | `showCustomEditor`（自定义样式编辑器）与 `showResetConfirm`（重置确认）两个模态统一使用 `fade`（遮罩 150ms）+ `scale`（面板 200ms，起点 0.95）组合；关闭时反向播放 |
| 技术要求 | 用 `{#if}` 块包裹以触发 out 转场；遮罩点击关闭时同样有出场动画 |
| 预期效果 | 模态出现/消失有层次感 |
| 验收标准 | ① 打开/关闭各 3 次无跳变；② 出场动画播完才真正卸载组件（Svelte `{#if}` 默认行为，确认无手动 DOM 移除）；③ 关闭动画期间快速重开无状态错乱 |
| 工作量 | 0.5 人日 |
| 依赖 | 无 |

### T2.3 Toast 提示动画

| 项目 | 内容 |
|---|---|
| 实现目标 | `toastVisible` 控制的提示条加 `slide` + `fade` 组合转场（入场 200ms、出场 150ms），自动消失定时器不变 |
| 技术要求 | 复用现有 toast 逻辑，仅包 `{#if}` + transition |
| 预期效果 | 保存成功等反馈更柔和 |
| 验收标准 | 连续两次触发 toast（如连续保存）时旧动画被正确中断，不出现双重叠影 |
| 工作量 | 0.5 人日 |
| 依赖 | 无 |

### T2.4 预设卡片微交互

| 项目 | 内容 |
|---|---|
| 实现目标 | 三个预设卡片（默认/极简/游戏）增加 CSS hover 态（上浮 2px + 阴影加深，`transition: 150ms ease-out`）与选中态（边框高亮 + 轻微 scale 1.02） |
| 技术要求 | 纯 CSS，不引入 JS 动画；尊重 `prefers-reduced-motion` 媒体查询 |
| 预期效果 | 预设选择有明确的可交互反馈 |
| 验收标准 | ① hover/选中/点击三态视觉可区分；② 键盘 Tab 导航时 focus 态同样可见（无障碍）；③ `npm run test` 通过 |
| 工作量 | 0.5 人日 |
| 依赖 | 无 |

### T2.5 滑块调节实时预览联动

| 项目 | 内容 |
|---|---|
| 实现目标 | `opacity`、`fadeDelay` 滑块拖动时立即 `invoke` 到 Rust 侧生效，让屏幕上的真实气泡实时反馈 |
| 技术要求 | 拖动过程用 `input` 事件触发（已有 invoke 通道 `update_opacity`/`update_settings`），增加 100ms 防抖避免高频调用；松手时才写 store 持久化 |
| 预期效果 | 调参即所见，大幅提升设置体验 |
| 验收标准 | ① 拖动滑块时气泡透明度/停留时长肉眼可见地实时变化；② 拖动过程 CPU 无明显尖峰（防抖生效）；③ 仅松手后 `settings.json` 才写入 |
| 工作量 | 1 人日 |
| 依赖 | 无 |

---

## 建议三：tiny-skia 气泡视觉升级（优先级 P1–P2）

### T3.1 default 风格增加软阴影

| 项目 | 内容 |
|---|---|
| 实现目标 | `renderer/mod.rs` 的 default 分支在气泡背景外绘制 3–4 层同心圆角矩形，alpha 逐层递减（如 40/25/15/8）、尺寸逐层外扩（2/4/6/8px），模拟柔和投影 |
| 技术要求 | `BubbleLayout` 的 `total_width/height` 需预留阴影 padding（约 12px），否则阴影被裁切；`shadow_color` 字段复用；性能约束：单帧渲染 < 8ms（气泡典型尺寸 < 400×80） |
| 预期效果 | 气泡与桌面产生空间层次，观感对标 Keyviz |
| 验收标准 | ① 四个象限位置阴影均不被窗口边界裁切（与 overlay.rs 的 work_area 边距逻辑配合）；② 1080p/2K 下截图目视无锯齿；③ 连击 10 键 FPS 不掉帧（tick 间隔保持） |
| 工作量 | 1.5 人日 |
| 依赖 | 无 |

### T3.2 渐变高光（3d_key 风格增强）

| 项目 | 内容 |
|---|---|
| 实现目标 | 用 tiny-skia 的 `LinearGradient` shader 为 3d_key 顶面添加自上而下的明暗渐变（顶部亮 8%），并为卡通风格加顶面高光条 |
| 技术要求 | `Paint::set_shader(LinearGradient::new(start, end, stops))`；渐变色由 `bg_color` 程序化计算（亮度 ±8%），不新增配置项 |
| 预期效果 | 3D 按键立体感显著增强 |
| 验收标准 | ① 深色/浅色背景配置下渐变方向与明暗均正确；② 截图与设计稿（可先出 Figma 标注）一致 |
| 工作量 | 1 人日 |
| 依赖 | 无 |

### T3.3 新增主题配色预设

| 项目 | 内容 |
|---|---|
| 实现目标 | 新增 2–3 套内置配色（如"深空黑"、"奶油白"、"霓虹蓝"），作为 `CustomStyle` 快捷预设，走现有 `apply_preset` 通道扩展 |
| 技术要求 | 每套预设包含完整 7 字段（bg/text/border/shadow 等），确保文字对比度 ≥ WCAG AA（4.5:1）；预设 ID 稳定以便持久化 |
| 预期效果 | 用户一键切换整套视觉风格 |
| 验收标准 | ① 每套预设在纯黑/纯白/彩色壁纸上截图均清晰可读；② 新增单元测试覆盖 `apply_preset` 新 ID 与未知 ID 回退；③ 旧配置不受影响 |
| 工作量 | 1 人日 |
| 依赖 | 无 |

### T3.4 连击乘数动画（可选，P2）

| 项目 | 内容 |
|---|---|
| 实现目标 | `is_multiplier` 节点（"x2" 连击计数）出现时带 150ms 的 alpha 淡入 + 微缩放，强化连击爽感 |
| 技术要求 | 需在渲染侧引入 multiplier 的存活时间戳（状态机透传），`Transform::from_scale` 实现缩放 |
| 预期效果 | 连击反馈有节奏感 |
| 验收标准 | 快速连击时动画不叠加错乱；单帧渲染仍 < 8ms |
| 工作量 | 1.5 人日 |
| 依赖 | T1.1 |

---

## 建议四：设计规范沉淀为技能（优先级 P2）

### T4.1 编写项目动效设计规范

| 项目 | 内容 |
|---|---|
| 实现目标 | 产出 `docs/动效设计规范.md`：时长标准（入场 120ms / 淡出 280ms / UI 转场 150–200ms）、缓动标准（出场元素用 easeOut、模态用 scale+fade）、色彩 Token 表（对应 T3.3 预设） |
| 技术要求 | 规范数值必须与代码实现一致（以 T1/T3 落地值为准），避免文档漂移 |
| 预期效果 | 后续任何 UI 改动有统一基准，避免风格发散 |
| 验收标准 | 文档中每个数值可在代码中检索到对应实现；团队评审通过 |
| 工作量 | 0.5 人日 |
| 依赖 | T1.x、T3.x 完成 |

### T4.2 封装为 Qoder Skill

| 项目 | 内容 |
|---|---|
| 实现目标 | 用 `create-skill` 将 T4.1 规范 + 本项目 tiny-skia 渲染约束（无 GPU blur、RGBA/BGRA 转换、Layered Window 更新路径）封装为 `osd-motion-design` 技能，放入 `.qoder` |
| 技术要求 | SKILL.md 含触发条件（修改渲染器/动画/样式时激活）、核心约束清单、验收检查项 |
| 预期效果 | AI 辅助开发时自动遵守项目动效规范 |
| 验收标准 | 新会话中让 AI 修改气泡样式，其产出自动引用规范中的时长/缓动标准 |
| 工作量 | 0.5 人日 |
| 依赖 | T4.1 |

---

## 优先级与依赖总览

| 优先级 | 任务 | 依赖 | 合计工作量 |
|---|---|---|---|
| **P0** | T1.1 → T1.2、T1.3、T1.4 | 无 | 2.5 人日 |
| **P1** | T2.1–T2.5（并行可做） | 无 | 3 人日 |
| **P1** | T3.1 → T3.2、T3.3 | T3.1 需先定 padding 方案 | 3.5 人日 |
| **P2** | T3.4 | T1.1 | 1.5 人日 |
| **P2** | T4.1 → T4.2 | 需 P0/P1 完成后以实际数值为准 | 1 人日 |

### 依赖关系图

```
T1.1 (easing 模块)
 ├── T1.2 (淡出曲线)
 ├── T1.3 (入场淡入) ── T1.4 (时长可配置)
 └── T3.4 (连击乘数动画, P2)

T2.1 ~ T2.5 (互相独立，可并行)

T3.1 (软阴影, 需先定 padding 方案) ── T3.2 (渐变高光)
T3.3 (主题配色, 独立)

T1.x + T3.x 完成 ── T4.1 (动效规范) ── T4.2 (Qoder Skill)
```

### 推荐执行路径

1. **第 1 天**：T1.1 + T1.2（半天见效）→ T1.3
2. **第 2–3 天**：T2.1–T2.4 并行
3. **第 4–5 天**：T3.1
4. **第 6–7 天**：T2.5 / T3.2 / T3.3
5. **第 8 天**：T4.x 收尾（T3.4 视排期可选）

> 其中 P0 仅需 2.5 人日即可交付最核心的动画质感提升。

---

## 附：相关源码文件索引

| 文件 | 涉及任务 |
|---|---|
| `osd-bubble/src-tauri/src/state_machine.rs` | T1.1–T1.4 |
| `osd-bubble/src-tauri/src/renderer/mod.rs` | T3.1、T3.2 |
| `osd-bubble/src-tauri/src/renderer/layout.rs` | T3.1（padding 预留） |
| `osd-bubble/src-tauri/src/overlay.rs` | T3.1（边界裁切配合） |
| `osd-bubble/src/routes/+page.svelte` | T2.1–T2.5 |
| `osd-bubble/src/lib/components/CustomStyleEditor.svelte` | T2.2、T3.3 |
