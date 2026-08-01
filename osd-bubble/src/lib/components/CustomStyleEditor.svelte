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
      <label>背景色</label>
      <div class="color-picker-group">
        <input type="color" bind:value={customStyle.bg_color} />
        <span class="color-value">{customStyle.bg_color}</span>
      </div>
    </div>

    <!-- 透明度 -->
    <div class="control">
      <label>透明度 ({Math.round(customStyle.bg_opacity * 100)}%)</label>
      <input type="range" min="0" max="1" step="0.05" bind:value={customStyle.bg_opacity} />
    </div>

    <!-- 文字颜色 -->
    <div class="control">
      <label>文字色</label>
      <div class="color-picker-group">
        <input type="color" bind:value={customStyle.text_color} />
        <span class="color-value">{customStyle.text_color}</span>
      </div>
    </div>

    <!-- 圆角 -->
    <div class="control">
      <label>圆角 ({customStyle.radius}px)</label>
      <input type="range" min="0" max="32" step="1" bind:value={customStyle.radius} />
    </div>

    <!-- 边框粗细 -->
    <div class="control">
      <label>边框粗细 ({customStyle.border_width}px)</label>
      <input type="range" min="0" max="5" step="0.5" bind:value={customStyle.border_width} />
    </div>

    <!-- 边框颜色 -->
    <div class="control">
      <label>边框色</label>
      <div class="color-picker-group">
        <input type="color" bind:value={customStyle.border_color} />
        <span class="color-value">{customStyle.border_color}</span>
      </div>
    </div>

    {#if bubbleStyle === '3d_key' || bubbleStyle === 'cartoon'}
      <!-- 阴影颜色 -->
      <div class="control">
        <label>阴影色</label>
        <div class="color-picker-group">
          <input type="color" bind:value={customStyle.shadow_color} />
          <span class="color-value">{customStyle.shadow_color}</span>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .editor {
    background: #2a2a2a;
    border-radius: 10px;
    padding: 16px;
    color: #e0e0e0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .title {
    font-size: 0.95rem;
    font-weight: 600;
    color: #e0e0e0;
  }

  .toolbar-actions {
    display: flex;
    gap: 8px;
  }

  .btn-reset {
    font-size: 0.75rem;
    padding: 6px 14px;
    min-height: 32px;
    border-radius: 4px;
    border: 1px solid #555;
    cursor: pointer;
    transition: background 0.2s ease, color 0.2s ease, border-color 0.2s ease;
    display: inline-flex;
    align-items: center;
  }

  .btn-reset {
    background: transparent;
    color: #aaa;
  }

  .btn-reset:hover {
    background: #3a3a3a;
    color: #e0e0e0;
    border-color: #777;
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .control {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 4px 0;
  }

  .control label {
    font-size: 0.85rem;
    color: #ccc;
    font-weight: 500;
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
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    padding: 0;
    background: transparent;
  }

  .color-picker-group input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  .color-picker-group input[type="color"]::-webkit-color-swatch {
    border: 2px solid #555;
    border-radius: 50%;
  }

  .color-value {
    font-family: monospace;
    font-size: 0.85rem;
    color: #999;
  }

  input[type="range"] {
    width: 100%;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: #444;
    border-radius: 3px;
    outline: none;
    margin-top: 4px;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #007aff;
    cursor: pointer;
    border: 2px solid #2a2a2a;
  }

  input[type="range"]::-webkit-slider-thumb:hover {
    background: #339aff;
  }
</style>
