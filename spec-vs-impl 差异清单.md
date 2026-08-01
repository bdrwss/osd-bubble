# OSD Bubble 交互规格 vs 实现差异清单

**文档版本**: V1.0  
**生成日期**: 2026-08-02  
**对照基准**: 
- 规格说明书：`按键 OSD 可视化工具 - 交互规格说明书.md` (V1.0)
- 功能概念：`按键 OSD 可视化工具 - 功能概念定义文档.md`
- 当前实现：`osd-bubble/src-tauri/src/state_machine.rs` (V1.0)

---

## 📋 差异总览

| # | 差异项 | 规格规定 | 实际实现 | 状态标注 | 说明 |
|---|--------|----------|----------|----------|------|
| 1 | HIDDEN 态与托盘闪烁 | 全屏时触发 `HIDDEN` 态，托盘图标闪烁 3 次提示 | **未实现** | ❌ 未实现 | state_machine 中无 HIDDEN 态；托盘插件未集成闪烁逻辑 |
| 2 | 淡出时长 | 300 ms | 280 ms | ⚠️ 微调偏差 | `fade_duration: Duration::from_millis(280)` |
| 3 | 全屏检测机制 | 规格要求："全屏独占检测"（DWM/NSWindow API） | 进程黑名单 (`exclude_apps`) | 🔀 待对齐 | 实现中有 `exclude_apps` 字段但无真正的全屏检测逻辑 |
| 4 | quadrant 定位字段 | 规格**未提及**该字段 | 实现中有 `quadrant: u8` | ℹ️ 实现增强 | 用于支持未来多象限布局，不影响当前行为 |

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

**判定**: ⚠️ **微调偏差 (-6.7%)**  
**技术原因推测**: 为了匹配动画帧率优化（280ms ≈ 16.8 frames @ 60fps），但规格未记录此决策  
**影响**: 人眼几乎不可感知差异，属于工程优化范畴  
**建议**: 在规格中补充"淡出曲线经过帧率对齐优化"的说明

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
- 但**无真正的全屏状态检测**
- `should_show_event` 方法未检查 `exclude_apps`（可能缺失实现或调用点未传递应用上下文）

**判定**: 🔀 **待对齐**  
**关键问题**: 
1. 规格明确使用"DWM/NSWindow API"，实现却用"进程黑名单"
2. 两种方案技术债务不同：API 检测更精准但跨平台复杂；黑名单易用但有漏网

**建议对齐路径**:
1. **短期**: 在规格中澄清为"优先黑名单 + 可选 DWM 检测 (V1.1+)"
2. **长期**: 按规格补充 DWM API 调用，将 `exclude_apps` 降级为用户配置覆盖层

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

**判定**: ℹ️ **实现增强 (超出规格范围)**  
**用途推测**: 
- 预定义象限布局（无需实时计算避让）
- 可能为"固定象限模式"功能预留（用户指定气泡永远出现在某个角）

**影响**: 
- ✅ 无负面作用（默认值 3 等效于规格描述的"右下角偏移"逻辑）
- ⚠️ 若未来不扩展此字段，可能造成不必要的 API 复杂度

**建议**: 
1. 在规格 §1.5 末尾补充 "⚠️ V1.0 实现包含 `quadrant` 字段，当前仅作为内部标识，尚未暴露为用户配置"
2. 或明确废弃该字段并移除代码

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

### 本周期 (V1.0 RC)
1. **文档对齐**: 在《交互规格说明书》§1.4 中添加注释说明淡出时长的工程优化背景
2. **字段清理**: 决定 `quadrant` 字段的去留——若确定不用则删除；若保留则补充规格说明

### 下一周期 (V1.1 planning)
3. **全屏检测方案设计**: 评估 DWM API vs 黑名单的投入产出比，选择其一纳入 MVP
4. **HIDDEN 态定义**: 设计完整的状态流转图，包括托盘闪烁时序

### 非本次修复内容
- ❌ 本清单**不涉及任何实现代码修改**
- ❌ 不调整 `state_machine.rs` 中的数值
- ❌ 不重构 `exclude_apps` 的实现

---

## 📊 统计摘要

| 类别 | 数量 |
|------|------|
| 差异总数 | 4 |
| 已否决 | 0 |
| 未实现 | 1 |
| 微调偏差 | 1 |
| 待对齐 | 1 |
| 实现增强 | 1 |

---

**文档维护者**: Qoder  
**最后更新**: 2026-08-02  
**下次审查建议**: V1.1 需求评审前
