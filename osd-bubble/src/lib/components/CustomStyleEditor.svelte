<script lang="ts">
  import { type CustomStyleParams } from "$lib/types";

  let {
    customStyle = $bindable(),
    bubbleStyle = 'default',
    onReset,
  }: {
    customStyle: CustomStyleParams;
    bubbleStyle?: string;
    onReset: () => void;
  } = $props();
</script>

<div class="editor">
  <div class="toolbar">
    <span class="title">自定义样式</span>
    <div class="toolbar-actions">
      <button class="btn-reset" onclick={onReset}>重置</button>
    </div>
  </div>

  <div class="grid">
    <!-- 背景色 -->
    <div class="control">
      <label for="cs-bg-color">背景色</label>
      <div class="color-picker-group">
        <input id="cs-bg-color" type="color" bind:value={customStyle.bg_color} />
        <span class="color-value">{customStyle.bg_color}</span>
      </div>
    </div>

    <!-- 透明度 -->
    <div class="control">
      <label for="cs-bg-opacity">透明度 ({Math.round(customStyle.bg_opacity * 100)}%)</label>
      <input id="cs-bg-opacity" type="range" min="0" max="1" step="0.05" bind:value={customStyle.bg_opacity} />
    </div>

    <!-- 文字颜色 -->
    <div class="control">
      <label for="cs-text-color">文字色</label>
      <div class="color-picker-group">
        <input id="cs-text-color" type="color" bind:value={customStyle.text_color} />
        <span class="color-value">{customStyle.text_color}</span>
      </div>
    </div>

    <!-- 圆角 -->
    <div class="control">
      <label for="cs-radius">圆角 ({customStyle.radius}px)</label>
      <input id="cs-radius" type="range" min="0" max="32" step="1" bind:value={customStyle.radius} />
    </div>

    <!-- 边框粗细 -->
    <div class="control">
      <label for="cs-border-width">边框粗细 ({customStyle.border_width}px)</label>
      <input id="cs-border-width" type="range" min="0" max="5" step="0.5" bind:value={customStyle.border_width} />
    </div>

    <!-- 边框颜色 -->
    <div class="control">
      <label for="cs-border-color">边框色</label>
      <div class="color-picker-group">
        <input id="cs-border-color" type="color" bind:value={customStyle.border_color} />
        <span class="color-value">{customStyle.border_color}</span>
      </div>
    </div>

    {#if bubbleStyle === '3d_key' || bubbleStyle === 'cartoon'}
      <!-- 阴影颜色 -->
      <div class="control">
        <label for="cs-shadow-color">阴影色</label>
        <div class="color-picker-group">
          <input id="cs-shadow-color" type="color" bind:value={customStyle.shadow_color} />
          <span class="color-value">{customStyle.shadow_color}</span>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .editor {
    background: var(--well, #ecede4);
    border: 1px solid var(--line, #d7d8cc);
    border-radius: 8px;
    padding: 14px 16px;
    color: var(--ink, #1d2126);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .title {
    font-family: var(--font-mono, monospace);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    color: var(--muted, #6d737c);
  }

  .toolbar-actions {
    display: flex;
    gap: 8px;
  }

  .btn-reset {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 5px 14px;
    min-height: 28px;
    border-radius: 6px;
    border: 1px solid var(--edge, #c0c2b4);
    background: var(--panel, #f7f7f2);
    color: var(--muted, #6d737c);
    box-shadow: 0 2px 0 var(--edge, #c0c2b4);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    transition: color 150ms ease-out, border-color 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .btn-reset:hover {
    color: var(--ink, #1d2126);
  }

  .btn-reset:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge, #c0c2b4);
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px 16px;
  }

  .control {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 2px 0;
  }

  .control label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--muted, #6d737c);
  }

  .color-picker-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .color-picker-group input[type="color"] {
    -webkit-appearance: none;
    appearance: none;
    border: none;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    cursor: pointer;
    padding: 0;
    background: transparent;
  }

  .color-picker-group input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  .color-picker-group input[type="color"]::-webkit-color-swatch {
    border: 1px solid var(--edge, #c0c2b4);
    border-radius: 8px;
  }

  .color-value {
    font-family: var(--font-mono, monospace);
    font-size: 0.78rem;
    color: var(--muted, #6d737c);
  }

  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 22px;
    margin-top: 2px;
    background: transparent;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
    background: var(--edge, #c0c2b4);
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 20px;
    margin-top: -8px;
    border-radius: 4px;
    background: var(--panel, #f7f7f2);
    border: 1px solid var(--edge, #c0c2b4);
    box-shadow: 0 2px 0 var(--edge, #c0c2b4);
    box-sizing: border-box;
    transition: border-color 150ms ease-out;
  }

  input[type="range"]:hover::-webkit-slider-thumb {
    border-color: var(--accent, #bc4708);
  }

  input[type="range"]::-moz-range-track {
    height: 4px;
    border-radius: 2px;
    background: var(--edge, #c0c2b4);
  }

  input[type="range"]::-moz-range-thumb {
    width: 12px;
    height: 20px;
    border-radius: 4px;
    background: var(--panel, #f7f7f2);
    border: 1px solid var(--edge, #c0c2b4);
    box-shadow: 0 2px 0 var(--edge, #c0c2b4);
    box-sizing: border-box;
  }

  input[type="range"]:focus-visible {
    outline: 2px solid var(--accent, #bc4708);
    outline-offset: 4px;
    border-radius: 4px;
  }

  @media (prefers-reduced-motion: reduce) {
    .btn-reset,
    .btn-reset:active {
      transform: none;
    }
  }
</style>
