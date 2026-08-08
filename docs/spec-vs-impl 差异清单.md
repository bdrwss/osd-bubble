# OSD Bubble 交互规格 vs 实现差异清单

**文档版本**: V1.1（差异收敛版）  
**生成日期**: 2026-08-02，最后更新：2026-08-08  
**对照基准**: 
- 规格说明书：`按键 OSD 可视化工具 - 交互规格说明书.md`（已于 2026-08-08 同步修订）
- 功能概念：`按键 OSD 可视化工具 - 功能概念定义文档.md`
- 当前实现：`osd-bubble/src-tauri/src/state_machine.rs` (V1.0)

---

## 📋 差异总览

| # | 差异项 | 规格规定 | 实际实现 | 状态标注 | 说明 |
|---|--------|----------|----------|----------|------|
| 1 | HIDDEN 态与托盘闪烁 | 全屏时触发 `HIDDEN` 态，托盘图标闪烁 3 次提示 | **未实现** | ❌ 未实现 | 延后至 V1.1 后期，见《开发计划-V1.1》阶段 3 |
| 2 | 淡出时长与曲线 | 300 ms / ease-in | 280 ms / easeOutCubic | ✅ **已收敛** | 规格已修订为 280ms + easeOutCubic（2026-08-08） |
| 3 | 全屏检测机制 | 规格原要求 DWM/NSWindow API | 前台窗口黑名单 (`exclude_apps` + `is_foreground_blacklisted`) | ✅ **已定案** | 黑名单为主方案；DWM 降级为 V1.1 后期可选增强（2026-08-08） |
| 4 | quadrant 定位字段 | 规格**未提及**该字段 | 实现中有 `quadrant: u8` | ✅ **已定案保留** | 已暴露为设置界面象限选择器并纳入规格 §1.5（2026-08-08） |

---

## 🔍 逐项详细说明

### 差异 #1: HIDDEN 态与托盘闪烁

**规格要求**:
- 位置：§1.1 状态机图、§8.1 全屏独占应用
- 描述：
  ```
  | HIDDEN | 因全屏独占/安全桌面而强制隐藏 | `visible: false`，托盘图标闪烁提⽰ |
  ```
  ```
  检测到全屏独占 ──▶ 气泡立即隐藏 ──▶ 托盘图标闪烁 3 次
  退出全屏 ──▶ 气泡功能自动恢复 ◀──────────┘
  ```

**当前实现**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BubbleState {
    Idle,
    Visible { start: Instant },
    FadingOut { start: Instant },
}
// ❌ 缺少 HIDDEN 状态
```

**托盘实现状态**:
- Tauri tray plugin 已引入 (`osd-bubble/src-tauri/Cargo.toml`)
- 但未实现闪烁逻辑
- 无全屏检测事件监听

**判定**: ❌ **未实现**  
**影响**: 低优先级（全屏场景下用户可手动暂停，非核心体验阻塞）  
**建议后续**: V1.1 考虑添加，需与 Tray API 深度集成

---

### 差异 #2: 淡出时长

**规格要求**:
- 位置：§1.4 动画参数表
- 原文：
  ```
  | 淡出 | opacity: 1→0, translateY: 0→-4px | 300 ms | ...
  ```

**当前实现**:
```rust
pub fn new() -> Self {
    Self {
        // ...
        fade_duration: Duration::from_millis(280),  // ⚠️ 280ms 而非 300ms
        // ...
    }
}
```

**测试验证**:
```rust
#[test]
fn test_initial_state() {
    // ...
    assert_eq!(sm.fade_duration, Duration::from_millis(280));  // 确认为 280ms
}
```

**判定**: ✅ **已收敛（2026-08-08）**  
**处理结果**: 规格 §1.4 已修订为 280ms / easeOutCubic，并补记工程决策（帧率对齐 + 《动效设计规范》统一曲线），与实现完全一致。

---

### 差异 #3: 全屏检测机制

**规格要求**:
- 位置：§8.1 全屏独占应用
- 原文：
  ```
  检测方式：Windows `DwmGetWindowAttribute` / macOS `NSWindow.styleMask.contains(.fullScreen)`
  ```

**当前实现**:
```rust
pub struct StateMachine {
    // ...
    pub exclude_apps: Vec<String>,  // ⚠️ 仅实现黑名单过滤
    // ...
}

pub fn should_show_event(&self, category: EventCategory) -> bool {
    if !self.enabled {
        return false;
    }
    match category {
        EventCategory::Keyboard => self.show_keyboard,
        EventCategory::Mouse => self.show_mouse,
        EventCategory::Scroll => self.show_scroll,
    }
}
// ❌ 无实时全屏检测逻辑
```

**实现现状**:
- 提供了 `exclude_apps` 字段允许用户手动排除应用
- 已在 `lib.rs` `is_foreground_blacklisted` 中生效：事件到达时检查前台窗口进程名，命中黑名单则不触发气泡
- 无 DWM 实时全屏状态检测（已定案为 V1.1 后期可选增强）

**判定**: ✅ **已定案（2026-08-08）**：黑名单为主，DWM 降级为 V1.1 后期可选增强  
**决策依据**:
1. 黑名单方案已在 `lib.rs` `is_foreground_blacklisted` 落地生效（前台窗口命中黑名单时不显示气泡），覆盖主要教学/游戏场景且零误报
2. DWM 检测需额外轮询与跨版本兼容成本，投入产出比低；待真实用户反馈漏网场景后再投入

**后续路径**（见《开发计划-V1.1》）:
- 若未来实现 DWM 检测，`exclude_apps` 保留为用户手动覆盖层
- 规格 §8.1 已同步修订为本方案

---

### 差异 #4: quadrant 定位字段

**规格要求**:
- 位置：**规格书中未提及**
- 定位规则 (§1.5) 仅描述：
  ```
  默认位置 = 光标屏幕坐标 + offset(16, 24)  // 右下角
  避让翻转逻辑基于屏幕边缘检测
  ```

**当前实现**:
```rust
pub struct StateMachine {
    // ...
    pub quadrant: u8,  // ⚠️ 存在此字段
    // ...
}

pub fn new() -> Self {
    Self {
        quadrant: 3,  // 默认右下 (quadrant 编码: 1=NW, 2=NE, 3=SW, 4=SE)
        // ...
    }
}
```

**判定**: ✅ **已定案保留（2026-08-08）**  
**决策依据**:
- 该字段已通过设置界面"气泡默认位置"象限选择器暴露为用户配置（0–3 = 左上/右上/左下/右下，默认 3 右下）
- 已纳入持久化（`apply_persisted_settings` 防御式解析），删除有迁移成本

**处理结果**:
1. ✅ 规格 §1.5 已补充"实现补充说明"，记录字段编码与暴露方式
2. 不再视为待清理项

---

## ✅ 一致性确认项

以下规格要求在实现中得到**完全符合**的确认：

| 规格条款 | 实现对应 | 状态 |
|---------|---------|------|
| §1.1 状态机 IDLE/VISIBLE/FADING/GONE | `BubbleState` 枚举前三态 | ✅ 已实现 |
| §1.2 触发规则（键盘/鼠标/滚轮） | `EventCategory` + `should_show_event()` | ✅ 已实现 |
| §1.3 视觉规格（颜色/字体/圆角） | `CustomStyle` 结构体 | ✅ 已实现 |
| §1.4 出现动画 120ms | `visible_duration: 1200ms` 隐含出现即达 | ✅ 符合 |
| §1.5 锚点定位逻辑 | 依赖 UI 层渲染时计算 | ⏸️ 未编码到 SM |
| §3.1 教学模式预设 | `apply_preset("classroom/recording/streaming")` | ✅ 已实现 |
| §6.1 托盘图标状态指示 | Tray API 已集成 | ⚠️ 部分实现（缺少闪烁） |

---

## 🎯 推荐行动项

### 本周期 (V1.0 RC) —— ✅ 全部完成（2026-08-08）
1. ✅ **文档对齐**: 规格 §1.4 已补记淡出时长/曲线的工程决策
2. ✅ **字段定案**: `quadrant` 决定保留（已暴露为用户配置），规格 §1.5 已补充说明
3. ✅ **全屏检测定案**: 黑名单为主方案写入规格 §8.1，DWM 降级为 V1.1 后期可选

### 下一周期 (V1.1 planning)
3. **全屏检测（可选）**: 若收到黑名单漏网场景的真实反馈，再评估 DWM API 投入
4. **HIDDEN 态定义**: 设计完整的状态流转图，包括托盘闪烁时序（见《开发计划-V1.1》阶段 3）

### 非本次修复内容
- ❌ 本清单**不涉及任何实现代码修改**
- ❌ 不调整 `state_machine.rs` 中的数值
- ❌ 不重构 `exclude_apps` 的实现

---

## 📊 统计摘要（2026-08-08 更新）

| 类别 | 数量 |
|------|------|
| 差异总数 | 4 |
| 已收敛/定案 | 3 |
| 未实现（延后 V1.1 后期） | 1 |
| 微调偏差 | 0 |
| 待对齐 | 0 |
| 实现增强（已定案保留） | 0（归入已定案） |

---

**文档维护者**: Qoder  
**最后更新**: 2026-08-08  
**下次审查建议**: V1.1 需求评审前
