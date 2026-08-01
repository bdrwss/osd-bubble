---
kind: frontend_style
name: SvelteKit + 内联 CSS 样式体系
category: frontend_style
scope:
    - '**'
source_files:
    - osd-bubble/src/routes/+page.svelte
    - osd-bubble/src/routes/custom-style/+page.svelte
    - osd-bubble/static/custom-style.html
    - osd-bubble/svelte.config.js
    - osd-bubble/vite.config.js
    - osd-bubble/src/app.html
---

本项目采用 SvelteKit 作为前端框架，样式系统完全基于 Svelte 组件内联 `<style>` 块与原生 CSS，未引入任何第三方 UI 组件库或 CSS 框架（如 Tailwind、Bootstrap、Ant Design 等）。

**样式架构与组织方式**
- 所有样式均以内联 `<style>` 标签形式写在 `.svelte` 组件文件中，遵循 Svelte 的 scoped CSS 机制，每个组件拥有独立的作用域样式。
- 主页面 `src/routes/+page.svelte` 包含完整的设置界面样式，包括预设卡片、开关控件、滑块、象限选择器、主题分段控件、黑名单列表、底部操作栏等。
- 自定义样式页面 `src/routes/custom-style/+page.svelte` 提供气泡外观实时调节面板（背景色、透明度、文字颜色、圆角、边框粗细/颜色），其样式同样内联在组件中。
- 同时存在一个独立的静态 HTML 文件 `static/custom-style.html`，使用原生 JavaScript + CSS 实现相同功能，通过 Tauri v1 风格的 `window.__TAURI__.core` API 与后端通信，作为兼容方案存在。

**主题与配色策略**
- 通过动态设置 `document.documentElement` 的 `data-theme` 属性（值为 `dark` / `light` / 移除）来实现深色/浅色/跟随系统三种主题模式。
- 使用 CSS `@media (prefers-color-scheme: dark)` 媒体查询实现系统级暗色模式适配。
- 主色调为 iOS 风格蓝色 `#007aff`，用于激活状态、按钮高亮、边框选中态等；辅助色包括红色 `#ff3b30`（删除/关闭）、灰色系 `#888`/`#666`（描述文本）。
- 字体栈统一使用系统字体：`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`。

**CSS 方法论与约定**
- 采用 BEM 风格的类名命名（如 `.setting-card`、`.toggle-row`、`.theme-segmented`、`.blacklist-item`），语义清晰且避免冲突。
- 大量使用 Flexbox 和 CSS Grid 进行布局，未使用任何预处理器（SCSS/Less）。
- 交互状态通过 CSS 类切换实现（如 `.active`、`.expanded`），配合 `transition` 过渡动画。
- 响应式仅依赖 `@media (prefers-color-scheme: dark)`，无断点式移动端适配（桌面端工具应用）。

**构建与集成**
- 通过 `svelte.config.js` 配置 `vitePreprocess()` 启用 Vite 预处理。
- `vite.config.js` 固定开发端口 1420，HMR 仅在 Tauri 开发模式下启用。
- 使用 `@sveltejs/adapter-static` 以 SPA 模式构建，fallback 到 `index.html`。
- 项目根目录无全局 CSS 文件或样式入口，样式完全分散在各组件内部。