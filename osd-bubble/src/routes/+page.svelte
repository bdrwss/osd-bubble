<script>
  import { onMount } from 'svelte';
  import { slide, fade, scale, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { browser } from '$app/environment';
  import { LazyStore } from '@tauri-apps/plugin-store';
  import CustomStyleEditor from '$lib/components/CustomStyleEditor.svelte';
  import { invoke } from '@tauri-apps/api/core';

  // 持久化层：tauri-plugin-store 为唯一事实来源（键 osdBubbleSettings）
  const SETTINGS_KEY = 'osdBubbleSettings';
  const settingsStore = new LazyStore('settings.json');

  function isTauriRuntime() {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  }

  // Reactive state variables
  let fadeDelay = 1000;
  let opacity = 85;
  let quadrant = '3';
  let bubbleScale = 120;
  let fontFamily = "'Microsoft YaHei UI'";
  let bubbleStyle = 'default';
  let customStyle = {
    bg_color: '#000000',
    bg_opacity: 0.7,
    border_width: 0,
    border_color: '#000000',
    radius: 8,
    text_color: '#ffffff',
    shadow_color: '#000000'
  };
  let enabled = true;
  let showKeyboard = true;
  let showMouse = true;
  let showScroll = false;
  let onlyShortcuts = false;
  let mergeRepeats = true;
  let animStyle = 'bounce';
  let theme = 'system';
  /** @type {string[]} */
  let excludeApps = [];
  let newAppInput = '';
  let autoStart = false;
  let toastVisible = false;
  let toastMessage = '';
  let currentTab = 'bubble';
  let showResetConfirm = false;
  let showCustomEditor = false;
  let showAbout = false;
  let reduceMotion = false;
  let isLoaded = false;

  // Presets
  const presets = [
    {
      id: 'default',
      name: '默认配置',
      desc: '标准按键显示，适合日常使用',
      icon: '<svg viewBox="0 0 24 24"><path d="M20 5H4c-1.1 0-2 .9-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2zM5 17l7-7 7 7H5z"/></svg>',
      fadeDelay: 1000,
      opacity: 85,
      quadrant: '3',
      bubbleScale: 120,
      fontFamily: "'Microsoft YaHei UI'",
      style: 'default'
    },
    {
      id: 'minimal',
      name: '极简模式',
      desc: '更小更透明，不干扰视觉',
      icon: '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="none"/><circle cx="12" cy="12" r="4"/></svg>',
      fadeDelay: 500,
      opacity: 60,
      quadrant: '3',
      bubbleScale: 90,
      fontFamily: "'SimHei'",
      style: 'retro_terminal'
    },
    {
      id: 'gaming',
      name: '游戏模式',
      desc: '高对比度，快速响应',
      icon: '<svg viewBox="0 0 24 24"><polygon points="12 2 15 10 22 10 17 15 19 22 12 18 5 22 7 15 2 10 9 10"/></svg>',
      fadeDelay: 300,
      opacity: 100,
      quadrant: '3',
      bubbleScale: 140,
      fontFamily: "'KaiTi'",
      style: 'cartoon'
    }
  ];

  // Load saved settings
  onMount(async () => {
    reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (browser) {
      let saved = null;
      if (isTauriRuntime()) {
        try {
          saved = await settingsStore.get(SETTINGS_KEY);
          if (!saved) {
            // 旧版本数据一次性迁移：从 localStorage 读取后由下次保存写入 store
            const legacy = localStorage.getItem('osdBubbleSettings');
            if (legacy) {
              saved = JSON.parse(legacy);
            }
          }
        } catch (e) {
          console.error('Failed to load settings from store:', e);
        }
      } else {
        // 非 Tauri 环境（纯 vite dev）回退 localStorage
        const legacy = localStorage.getItem('osdBubbleSettings');
        if (legacy) {
          try {
            saved = JSON.parse(legacy);
          } catch (e) {
            console.error('Failed to load settings:', e);
          }
        }
      }

      if (saved) {
        try {
          fadeDelay = saved.fadeDelay || 1000;
          opacity = saved.opacity || 85;
          quadrant = saved.quadrant || '3';
          bubbleScale = saved.bubbleScale || 120;
          fontFamily = saved.fontFamily || "'Microsoft YaHei UI'";
          bubbleStyle = saved.bubbleStyle || 'default';
          customStyle = saved.customStyle || customStyle;
          enabled = saved.enabled !== undefined ? saved.enabled : true;
          showKeyboard = saved.showKeyboard !== undefined ? saved.showKeyboard : true;
          showMouse = saved.showMouse !== undefined ? saved.showMouse : true;
          showScroll = saved.showScroll !== undefined ? saved.showScroll : false;
          onlyShortcuts = saved.onlyShortcuts !== undefined ? saved.onlyShortcuts : false;
          mergeRepeats = saved.mergeRepeats !== undefined ? saved.mergeRepeats : true;
          animStyle = saved.animStyle || 'bounce';
          theme = saved.theme || 'system';
          excludeApps = saved.excludeApps || [];
          autoStart = saved.autoStart || false;
        } catch (e) {
          console.error('Failed to apply loaded settings:', e);
        }
      }
    }

    isLoaded = true;
    // Set theme
    applyTheme();
  });

  // Apply theme
  function applyTheme() {
    if (!browser) return;
    
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      document.documentElement.setAttribute('data-theme', theme);
    }
  }

  // 仅同步运行时状态到 Rust 后端（不落盘），供拖动滑块时实时预览
  function syncToBackend() {
    if (!isTauriRuntime()) return;
    invoke('update_settings', { fadeDelay });
    invoke('update_opacity', { opacity: opacity / 100 });
    invoke('update_position', { quadrant: parseInt(quadrant) });
    invoke('update_bubble_style', { style: bubbleStyle });
    invoke('update_anim_style', { style: animStyle });
    invoke('update_custom_style', { style: customStyle });
    invoke('toggle_enabled', { enabled });
    invoke('update_show_keyboard', { show: showKeyboard });
    invoke('update_show_mouse', { show: showMouse });
    invoke('update_show_scroll', { show: showScroll });
    invoke('update_only_shortcuts', { only: onlyShortcuts });
    invoke('update_merge_repeats', { merge: mergeRepeats });
    invoke('update_exclude_apps', { apps: excludeApps });
  }

  // 仅持久化设置（store / localStorage），供防抖后延迟写入
  function persistSettings() {
    if (!browser) return;
    const settings = {
      fadeDelay,
      opacity,
      quadrant,
      bubbleScale,
      fontFamily,
      bubbleStyle,
      animStyle,
      customStyle,
      enabled,
      showKeyboard,
      showMouse,
      showScroll,
      onlyShortcuts,
      mergeRepeats,
      theme,
      excludeApps,
      autoStart
    };
    if (isTauriRuntime()) {
      settingsStore.set(SETTINGS_KEY, settings)
        .then(() => settingsStore.save())
        .catch((e) => console.error('Failed to save settings to store:', e));
    } else {
      localStorage.setItem('osdBubbleSettings', JSON.stringify(settings));
    }
  }

  // Save settings
  function saveSettings() {
    persistSettings();
    syncToBackend();
  }

  // 防抖计时器：后端同步 100ms（拖动滑块时限制 invoke 频率），
  // 持久化 700ms（拖动停顿/松手后才写 settings.json）
  let syncTimer = 0;
  let persistTimer = 0;
  let firstRun = true;

  $: {
    fadeDelay; opacity; quadrant; bubbleScale; fontFamily; bubbleStyle; animStyle; customStyle; enabled; showKeyboard; showMouse; showScroll; onlyShortcuts; mergeRepeats; theme; excludeApps; autoStart;
    if (isLoaded && browser) {
      if (firstRun) {
        // 首次加载回填不触发保存，与旧行为保持一致
        firstRun = false;
      } else {
        clearTimeout(syncTimer);
        syncTimer = setTimeout(syncToBackend, 100);
        clearTimeout(persistTimer);
        persistTimer = setTimeout(persistSettings, 700);
      }
    }
  }


  /** @type {Record<string, { bg_color: string; bg_opacity: number; border_width: number; border_color: string; radius: number; text_color: string; shadow_color: string; }>} */
  const stylePresets = {
    'default': { bg_color: '#000000', bg_opacity: 0.7, border_width: 0, border_color: '#000000', radius: 8, text_color: '#ffffff', shadow_color: '#000000' },
    '3d_key': { bg_color: '#fafafa', bg_opacity: 1.0, border_width: 1.0, border_color: '#e0e0e0', radius: 8, text_color: '#212121', shadow_color: '#b4b4b4' },
    'cartoon': { bg_color: '#ffffff', bg_opacity: 1.0, border_width: 1.5, border_color: '#000000', radius: 16, text_color: '#111111', shadow_color: '#000000' },
    'retro_terminal': { bg_color: '#050505', bg_opacity: 0.95, border_width: 2.0, border_color: '#00ff41', radius: 0, text_color: '#00ff41', shadow_color: '#000000' },
    // 主题配色预设：只改配色不改气泡形状，渲染端同步支持（state_machine::apply_preset）
    'deep_space': { bg_color: '#101418', bg_opacity: 0.85, border_width: 1.0, border_color: '#2e3a46', radius: 12, text_color: '#e8eaed', shadow_color: '#000000' },
    'cream_white': { bg_color: '#fdf6ec', bg_opacity: 0.95, border_width: 1.0, border_color: '#e8d5b7', radius: 12, text_color: '#3d3229', shadow_color: '#8a7a63' },
    'neon_blue': { bg_color: '#0a1a2f', bg_opacity: 0.9, border_width: 1.5, border_color: '#00d4ff', radius: 10, text_color: '#7df9ff', shadow_color: '#001f33' }
  };

  // 主题预设（仅配色，不改变气泡形状）
  const themePresets = ['deep_space', 'cream_white', 'neon_blue'];

  // Set style and preview
  /** @param {string} style */
  function setStyleAndPreview(style) {
    bubbleStyle = style;
    customStyle = { ...stylePresets[style] };
    if (style === 'retro_terminal') {
      fontFamily = "monospace";
    }
  }

  // 应用主题配色预设：只替换配色，保留当前气泡形状
  /** @param {string} id */
  function applyThemePreset(id) {
    customStyle = { ...stylePresets[id] };
  }

  // 判断某个主题配色是否处于选中态
  /** @param {string} id */
  function isThemeActive(id) {
    const preset = stylePresets[id];
    return customStyle.bg_color === preset.bg_color && customStyle.text_color === preset.text_color;
  }

  // Add excluded app
  function addExcludeApp() {
    if (newAppInput.trim()) {
      excludeApps.push(newAppInput.trim());
      newAppInput = '';
      saveSettings();
      showToast('已添加到黑名单');
    }
  }

  // Remove excluded app
  /** @param {string} app */
  function removeExcludeApp(app) {
    excludeApps = excludeApps.filter(a => a !== app);
    saveSettings();
    showToast('已从黑名单移除');
  }

  // Toggle auto start
  async function toggleAutoStart() {
    autoStart = !autoStart;
    try {
      const response = await fetch('/api/auto-start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enable: autoStart })
      });
      if (!response.ok) {
        autoStart = !autoStart;
        showToast('设置开机自启动失败');
      } else {
        saveSettings();
        showToast(autoStart ? '已启用开机自启动' : '已禁用开机自启动');
      }
    } catch (error) {
      autoStart = !autoStart;
      showToast('网络错误');
    }
  }

  // Show toast
  /** @param {string} message */
  function showToast(message) {
    toastMessage = message;
    toastVisible = true;
    setTimeout(() => {
      toastVisible = false;
    }, 2000);
  }

  // Handle reset custom style only
  function handleReset() {
    if (stylePresets[bubbleStyle]) {
      customStyle = { ...stylePresets[bubbleStyle] };
      saveSettings();
      showToast('自定义样式已重置');
    }
  }

  // Reset defaults
  function resetDefaults() {
    fadeDelay = 1000;
    opacity = 85;
    quadrant = '3';
    bubbleScale = 120;
    fontFamily = "'Microsoft YaHei UI'";
    bubbleStyle = 'default';
    customStyle = {
      bg_color: '#000000',
      bg_opacity: 0.7,
      border_width: 0,
      border_color: '#000000',
      radius: 8,
      text_color: '#ffffff',
      shadow_color: '#000000'
    };
    enabled = true;
    showKeyboard = true;
    showMouse = true;
    showScroll = false;
    onlyShortcuts = false;
    mergeRepeats = true;
    animStyle = 'bounce';
    theme = 'system';
    excludeApps = [];
    autoStart = false;
    saveSettings();
    showResetConfirm = false;
    showToast('已恢复默认设置');
  }

  // App version
  const appVersion = '1.0.0';

  // Escape 关闭弹层类交互
  /** @param {KeyboardEvent} e */
  function handleKeydown(e) {
    if (e.key === 'Escape') {
      showAbout = false;
      showResetConfirm = false;
    }
  }

  // 模态打开时聚焦关闭按钮（autofocus 的无障碍替代）
  /** @param {HTMLElement} node */
  function focusOnMount(node) {
    node.focus();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-shell">

  <header class="topbar">
    <div class="topbar-inner">
      <div class="wordmark-row">
        <span class="wordmark">OSD BUBBLE</span>
        <span class="wordmark-tag">KEYSTROKE OSD · v{appVersion}</span>
      </div>
      <div class="tab-container">
        <button
          class="tab-key {currentTab === 'bubble' ? 'active' : ''}"
          onclick={() => currentTab = 'bubble'}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="6" width="18" height="13" rx="2.5"></rect>
            <path d="M7.5 10.5h.01M12 10.5h.01M16.5 10.5h.01M7.5 15h9"></path>
          </svg>
          气泡设置
        </button>
        <button
          class="tab-key {currentTab === 'other' ? 'active' : ''}"
          onclick={() => currentTab = 'other'}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 8h8.5M19 8h1M4 16h1.5M11 16h9"></path>
            <circle cx="15.5" cy="8" r="2.5"></circle>
            <circle cx="8" cy="16" r="2.5"></circle>
          </svg>
          其他设置
        </button>
      </div>
    </div>
  </header>

  <main class="scroll-area">
    <div class="container">
      {#if toastVisible}
        <div class="toast" in:fly={{ y: reduceMotion ? 0 : 12, duration: 200, easing: cubicOut }} out:fade={{ duration: 150 }}>
          {toastMessage}
        </div>
      {/if}
      
      {#if currentTab === 'bubble'}
        <div class="tab-content" in:fade={{ duration: 180, easing: cubicOut }} out:fade={{ duration: 120 }}>
          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">DISPLAY</span>
              <span class="panel-title">显示</span>
            </header>

            <div class="row">
              <div class="row-line">
                <label for="fade-delay">气泡显示时长</label>
                <span class="value">{fadeDelay} ms</span>
              </div>
              <input id="fade-delay" type="range" min="300" max="5000" step="100" bind:value={fadeDelay} />
              <p class="description">操作停止后气泡继续显示的时间</p>
            </div>

            <div class="row">
              <div class="row-line">
                <span class="row-label" id="anim-style-label">入场动效</span>
              </div>
              <div class="theme-segmented" role="group" aria-labelledby="anim-style-label">
                <button class="theme-seg {animStyle === 'bounce' ? 'active' : ''}" onclick={() => { animStyle = 'bounce'; }}>
                  弹性回弹
                </button>
                <button class="theme-seg {animStyle === 'fade' ? 'active' : ''}" onclick={() => { animStyle = 'fade'; }}>
                  平滑渐显
                </button>
                <button class="theme-seg {animStyle === 'slide_up' ? 'active' : ''}" onclick={() => { animStyle = 'slide_up'; }}>
                  向上滑入
                </button>
                <button class="theme-seg {animStyle === 'instant' ? 'active' : ''}" onclick={() => { animStyle = 'instant'; }}>
                  极简瞬显
                </button>
              </div>
              <p class="description">按键气泡浮现时的动画缓动曲线</p>
            </div>

            <div class="row">
              <div class="row-line">
                <label for="opacity">气泡不透明度</label>
                <span class="value">{opacity}%</span>
              </div>
              <input id="opacity" type="range" min="40" max="100" step="5" bind:value={opacity} />
              <p class="description">调节气泡整体的透明程度</p>
            </div>

            <div class="row">
              <div class="row-line">
                <label for="bubble-scale">气泡缩放比例</label>
                <span class="value">{bubbleScale}%</span>
              </div>
              <input id="bubble-scale" type="range" min="80" max="200" step="5" bind:value={bubbleScale} />
              <p class="description">调整气泡的整体大小</p>
            </div>

            <div class="row">
              <div class="row-line">
                <span class="row-label" id="quadrant-label">气泡默认位置</span>
              </div>
              <div class="quadrant-selector" role="group" aria-labelledby="quadrant-label">
                <button class="quad-btn {quadrant === '0' ? 'active' : ''}" onclick={() => { quadrant = '0'; }} aria-label="左上方"></button>
                <button class="quad-btn {quadrant === '1' ? 'active' : ''}" onclick={() => { quadrant = '1'; }} aria-label="右上方"></button>
                <button class="quad-btn {quadrant === '2' ? 'active' : ''}" onclick={() => { quadrant = '2'; }} aria-label="左下方"></button>
                <button class="quad-btn {quadrant === '3' ? 'active' : ''}" onclick={() => { quadrant = '3'; }} aria-label="右下方 (推荐)"></button>
                <div class="center-cursor">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M4 4l7.07 17 2.51-7.39L21 11.07z"/>
                  </svg>
                </div>
              </div>
              <p class="description">气泡出现在鼠标的哪个方位</p>
            </div>
          </section>

          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">TYPE</span>
              <span class="panel-title">文字</span>
            </header>
            <div class="row">
              <div class="row-line">
                <label for="font-family">字体选择</label>
              </div>
              <select id="font-family" class="font-selector" bind:value={fontFamily}>
                <option value="'-apple-system, BlinkMacSystemFont, Segoe UI, Roboto, sans-serif'">系统默认字体</option>
                <option value="'Microsoft YaHei UI'">微软雅黑</option>
                <option value="'SimHei'">黑体</option>
                <option value="'KaiTi'">楷体</option>
                <option value="'FangSong'">仿宋</option>
                <option value="Arial, sans-serif">Arial</option>
                <option value="'Times New Roman', Times, serif">Times New Roman</option>
                <option value="monospace">等宽字体</option>
              </select>
              <p class="description">选择按键气泡的文字字体</p>
            </div>
          </section>

          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">LOOK</span>
              <span class="panel-title">外观</span>
            </header>

            <div class="row">
              <div class="row-line">
                <span class="row-label" id="bubble-style-label">气泡样式</span>
                <button class="btn-edit-custom {showCustomEditor ? 'active' : ''}" onclick={() => showCustomEditor = !showCustomEditor}>
                  {showCustomEditor ? '收起自定义' : '自定义配置'}
                </button>
              </div>
              <div class="style-selector" role="group" aria-labelledby="bubble-style-label">
                <button class="style-btn {bubbleStyle === 'default' ? 'active' : ''}" onclick={() => setStyleAndPreview('default')}>
                  <div class="preview preview-default">
                    <span class="key-cap">Ctrl</span>
                    <span class="key-cap">C</span>
                  </div>
                  <span class="style-name">默认样式</span>
                </button>
                <button class="style-btn {bubbleStyle === '3d_key' ? 'active' : ''}" onclick={() => setStyleAndPreview('3d_key')}>
                  <div class="preview preview-3d-key"><span class="key-cap">Ctrl</span><span class="key-cap">C</span></div>
                  <span class="style-name">3D 实体</span>
                </button>
                <button class="style-btn {bubbleStyle === 'cartoon' ? 'active' : ''}" onclick={() => setStyleAndPreview('cartoon')}>
                  <div class="preview preview-cartoon"><span class="key-cap">Ctrl</span><span class="key-cap">C</span></div>
                  <span class="style-name">卡通泡泡</span>
                </button>
                <button class="style-btn {bubbleStyle === 'retro_terminal' ? 'active' : ''}" onclick={() => setStyleAndPreview('retro_terminal')}>
                  <div class="preview preview-retro-terminal"><span class="key-cap">Ctrl</span><span class="key-cap">C</span></div>
                  <span class="style-name">极客终端</span>
                </button>
              </div>
              {#if showCustomEditor}
                <div class="custom-editor-wrapper" in:slide={{ duration: reduceMotion ? 1 : 250, easing: cubicOut }} out:slide={{ duration: reduceMotion ? 1 : 180 }}>
                  <CustomStyleEditor bind:customStyle bubbleStyle={bubbleStyle} onReset={handleReset} />
                </div>
              {/if}
              <p class="description">选择按键气泡的视觉风格</p>
            </div>

            <div class="row">
              <div class="row-line">
                <span class="row-label" id="theme-color-label">主题配色</span>
              </div>
              <div class="theme-color-selector" role="group" aria-labelledby="theme-color-label">
                {#each themePresets as id}
                  <button
                    class="theme-swatch {isThemeActive(id) ? 'active' : ''}"
                    style="--swatch-bg: {stylePresets[id].bg_color}; --swatch-fg: {stylePresets[id].text_color}; --swatch-border: {stylePresets[id].border_color};"
                    onclick={() => applyThemePreset(id)}
                    aria-label="应用主题配色 {id}"
                  >
                    <span class="swatch-preview">Aa</span>
                  </button>
                {/each}
              </div>
              <p class="description">一键切换整套配色，保留当前气泡形状</p>
            </div>
          </section>
        </div>
      {/if}

      {#if currentTab === 'other'}
        <div class="tab-content" in:fade={{ duration: 180, easing: cubicOut }} out:fade={{ duration: 120 }}>
          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">RUN</span>
              <span class="panel-title">运行</span>
            </header>
            <div class="row">
              <div class="row-line">
                <label class="enable-label" for="enable-switch">
                  <span class="status-dot {enabled ? 'on' : ''}" aria-hidden="true"></span>
                  {enabled ? '已启用' : '已暂停'}
                </label>
                <label class="switch switch-large">
                  <input id="enable-switch" type="checkbox" bind:checked={enabled} />
                  <span class="slider"></span>
                </label>
              </div>
              <p class="description">关闭后暂停所有按键气泡显示</p>
            </div>
            <div class="row">
              <div class="toggle-row">
                <label for="toggle-keyboard">键盘按键</label>
                <label class="switch">
                  <input id="toggle-keyboard" type="checkbox" bind:checked={showKeyboard} />
                  <span class="slider"></span>
                </label>
              </div>
              <div class="toggle-row">
                <label for="toggle-mouse">鼠标点击</label>
                <label class="switch">
                  <input id="toggle-mouse" type="checkbox" bind:checked={showMouse} />
                  <span class="slider"></span>
                </label>
              </div>
              <div class="toggle-row">
                <label for="toggle-scroll">滚轮操作</label>
                <label class="switch">
                  <input id="toggle-scroll" type="checkbox" bind:checked={showScroll} />
                  <span class="slider"></span>
                </label>
              </div>
            </div>
            <div class="row">
              <div class="row-line">
                <label for="toggle-only-shortcuts">仅显示快捷键/组合键</label>
                <label class="switch">
                  <input id="toggle-only-shortcuts" type="checkbox" bind:checked={onlyShortcuts} />
                  <span class="slider"></span>
                </label>
              </div>
              <p class="description">常规打字时不弹气泡，仅在按下包含 Ctrl、Alt、Shift、Win 的快捷键时显示</p>
            </div>
            <div class="row">
              <div class="row-line">
                <label for="toggle-merge-repeats">合并连续按键计数</label>
                <label class="switch">
                  <input id="toggle-merge-repeats" type="checkbox" bind:checked={mergeRepeats} />
                  <span class="slider"></span>
                </label>
              </div>
              <p class="description">连续快速敲击相同按键或快捷键时，合并展示为 ×2、×3 胶囊角标</p>
            </div>
          </section>

          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">SYSTEM</span>
              <span class="panel-title">系统</span>
            </header>
            <div class="row">
              <div class="row-line">
                <span class="row-label" id="theme-seg-label">外观主题</span>
              </div>
              <div class="theme-segmented" role="group" aria-labelledby="theme-seg-label">
                <button class="theme-seg {theme === 'dark' ? 'active' : ''}" onclick={() => { theme = 'dark'; }}>
                  <svg viewBox="0 0 24 24"><path d="M20 13.6A8.5 8.5 0 1 1 10.4 4a7 7 0 0 0 9.6 9.6z"/></svg>
                  深色
                </button>
                <button class="theme-seg {theme === 'light' ? 'active' : ''}" onclick={() => { theme = 'light'; }}>
                  <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>
                  浅色
                </button>
                <button class="theme-seg {theme === 'system' ? 'active' : ''}" onclick={() => { theme = 'system'; }}>
                  <svg viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="13" rx="2"/><path d="M9 21h6M12 17v4"/></svg>
                  跟随系统
                </button>
              </div>
            </div>
            <div class="row">
              <div class="row-line">
                <label for="toggle-autostart">开机自启动</label>
                <label class="switch">
                  <input id="toggle-autostart" type="checkbox" bind:checked={autoStart} onchange={toggleAutoStart} />
                  <span class="slider"></span>
                </label>
              </div>
              <p class="description">开机后自动在后台运行</p>
            </div>
          </section>

          <section class="panel">
            <header class="panel-head">
              <span class="panel-eyebrow">BLOCKLIST</span>
              <span class="panel-title">黑名单</span>
            </header>
            <div class="row">
              <p class="description description-top">以下应用中不会显示按键气泡</p>
              <div class="blacklist-input-group">
                <input type="text" placeholder="例如：csgo.exe" bind:value={newAppInput} onkeydown={(e) => e.key === 'Enter' && addExcludeApp()} />
                <button class="add-btn" onclick={addExcludeApp}>添加</button>
              </div>
              <div class="blacklist">
                {#each excludeApps as app}
                  <div class="blacklist-item">
                    <span>{app}</span>
                    <button class="remove-btn" onclick={() => removeExcludeApp(app)} aria-label="移除 {app}">×</button>
                  </div>
                {/each}
                {#if excludeApps.length === 0}
                  <p class="blacklist-empty">当前没有添加黑名单</p>
                {/if}
              </div>
            </div>
          </section>
        </div>
      {/if}
    </div>
  </main>

  <footer class="footer">
    <div class="footer-inner">
      <div class="footer-left">
        <button class="footer-btn reset-btn" onclick={() => showResetConfirm = true}>恢复默认设置</button>
        {#if showResetConfirm}
          <div class="confirm-dialog" in:scale={{ duration: reduceMotion ? 1 : 200, start: reduceMotion ? 1 : 0.95, easing: cubicOut }} out:fade={{ duration: 150 }}>
            <span>确定恢复默认设置？</span>
            <button class="confirm-yes" onclick={resetDefaults}>确定</button>
            <button class="confirm-no" onclick={() => showResetConfirm = false}>取消</button>
          </div>
        {/if}
      </div>
      <div class="footer-right">
        <button class="footer-btn about-btn" onclick={() => showAbout = true}>关于</button>
      </div>
    </div>
  </footer>

  {#if showAbout}
    <div class="modal-backdrop" in:fade={{ duration: 150 }} out:fade={{ duration: 150 }} role="presentation" onclick={(e) => { if (e.target === e.currentTarget) showAbout = false; }} onkeydown={handleKeydown}>
      <div
        class="about-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
        tabindex="-1"
        in:scale={{ duration: reduceMotion ? 1 : 200, start: reduceMotion ? 1 : 0.95, easing: cubicOut }}
        out:fade={{ duration: 150 }}
      >
        <header class="about-head">
          <span class="panel-eyebrow">ABOUT</span>
          <span class="panel-title" id="about-title">关于</span>
          <button class="about-close" onclick={() => showAbout = false} aria-label="关闭">×</button>
        </header>
        <div class="about-body">
          <div class="about-brand-visual">
            <img src="/logo.png" alt="OSD Bubble Logo" class="about-logo" />
          </div>
          <div class="about-name">OSD BUBBLE</div>
          <div class="about-meta">
            <span class="value">v{appVersion}</span>
            <span class="about-tag">按键可视化工具 · KEYSTROKE OSD</span>
          </div>
        </div>
        <footer class="about-foot">
          <button class="footer-btn" onclick={() => showAbout = false} use:focusOnMount>关闭</button>
        </footer>
      </div>
    </div>
  {/if}
</div>

<style>
  :root {
    --font-body: "Segoe UI", "Microsoft YaHei UI", "PingFang SC", system-ui, sans-serif;
    --font-mono: ui-monospace, "Cascadia Code", "JetBrains Mono", Consolas, "Courier New", monospace;

    --bg: #e6e7e0;
    --panel: #f7f7f2;
    --well: #ecede4;
    --ink: #1d2126;
    --muted: #6d737c;
    --line: #d7d8cc;
    --edge: #c0c2b4;
    --accent: #bc4708;
    --accent-soft: rgba(188, 71, 8, 0.12);
    --on-accent: #fff6ee;
    --danger: #b3261e;
    --shadow: 0 1px 2px rgba(24, 26, 20, 0.06);
    --scrim: rgba(30, 32, 26, 0.4);

    font-family: var(--font-body);
    color: var(--ink);
    background-color: var(--bg);
  }

  :root[data-theme="dark"] {
    --bg: #14171b;
    --panel: #1d2126;
    --well: #171a1f;
    --ink: #e6e8e2;
    --muted: #9aa1ab;
    --line: #2d3339;
    --edge: #0a0c0f;
    --accent: #f08c00;
    --accent-soft: rgba(240, 140, 0, 0.16);
    --on-accent: #221302;
    --shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    --scrim: rgba(0, 0, 0, 0.6);
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-theme]) {
      --bg: #14171b;
      --panel: #1d2126;
      --well: #171a1f;
      --ink: #e6e8e2;
      --muted: #9aa1ab;
      --line: #2d3339;
      --edge: #0a0c0f;
      --accent: #f08c00;
      --accent-soft: rgba(240, 140, 0, 0.16);
      --on-accent: #221302;
      --shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
      --scrim: rgba(0, 0, 0, 0.6);
    }
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background-color: var(--bg);
    overflow: hidden;
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    position: relative;
  }

  .scroll-area {
    position: fixed;
    top: 92px;
    bottom: 50px;
    left: 0;
    right: 0;
    overflow-y: auto;
    z-index: 1;
  }

  .scroll-area::-webkit-scrollbar {
    width: 8px;
    background: transparent;
  }

  .scroll-area::-webkit-scrollbar-track {
    background: transparent;
  }

  .scroll-area::-webkit-scrollbar-thumb {
    background-color: var(--edge);
    border-radius: 4px;
    border: 2px solid transparent;
    background-clip: content-box;
  }

  .scroll-area::-webkit-scrollbar-thumb:hover {
    background-color: var(--muted);
  }

  .container {
    max-width: 640px;
    margin: 0 auto;
    padding: 16px 20px 24px;
  }

  /* 顶栏：字标 + 键帽页签 */
  .topbar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
  }

  .topbar-inner {
    max-width: 640px;
    margin: 0 auto;
    padding: 12px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .wordmark-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .wordmark {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.22em;
  }

  .wordmark-tag {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--muted);
  }

  .tab-container {
    display: flex;
    gap: 10px;
  }

  .tab-key {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 9px 12px;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: 0 2px 0 var(--edge);
    color: var(--muted);
    font-family: var(--font-body);
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: color 150ms ease-out, background 150ms ease-out, border-color 150ms ease-out,
      box-shadow 150ms ease-out, transform 150ms ease-out;
  }

  .tab-key svg {
    width: 16px;
    height: 16px;
    stroke: currentColor;
    fill: none;
    stroke-width: 2;
  }

  .tab-key:hover {
    color: var(--ink);
  }

  .tab-key:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .tab-key.active {
    color: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 2px 0 var(--accent);
    transform: translateY(1px);
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  /* 面板：发丝线分组的设置区域 */
  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 10px;
    box-shadow: var(--shadow);
    overflow: hidden;
  }

  .panel-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 11px 18px;
    border-bottom: 1px solid var(--line);
  }

  .panel-eyebrow {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.18em;
    color: var(--accent);
  }

  .panel-title {
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .row {
    padding: 14px 18px;
  }

  .row + .row {
    border-top: 1px solid var(--line);
  }

  .row-line {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .row-line label,
  .row-line .row-label {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
    color: var(--ink);
    background: var(--well);
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 2px 8px;
  }

  .description {
    margin: 6px 0 0;
    font-size: 0.78rem;
    color: var(--muted);
  }

  .description-top {
    margin: 0 0 10px;
  }

  .btn-edit-custom {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
    padding: 5px 10px;
    border-radius: 6px;
    border: 1px solid var(--edge);
    background: var(--well);
    color: var(--muted);
    cursor: pointer;
    box-shadow: 0 2px 0 var(--edge);
    transition: color 150ms ease-out, border-color 150ms ease-out, background 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .btn-edit-custom:hover {
    color: var(--ink);
  }

  .btn-edit-custom:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .btn-edit-custom.active {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 2px 0 var(--accent);
  }

  .row-line .enable-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1.02rem;
    font-weight: 700;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--edge);
    transition: background 150ms ease-out, box-shadow 150ms ease-out;
  }

  .status-dot.on {
    background: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  /* 键帽式开关 */
  .switch {
    position: relative;
    display: inline-block;
    width: 46px;
    height: 26px;
    flex-shrink: 0;
  }

  .switch-large {
    width: 54px;
    height: 30px;
  }

  .switch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    margin: 0;
    cursor: pointer;
  }

  .slider {
    position: absolute;
    inset: 0;
    border-radius: 7px;
    background: var(--well);
    border: 1px solid var(--edge);
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.14);
    transition: background 150ms ease-out, border-color 150ms ease-out;
  }

  .slider:before {
    content: "";
    position: absolute;
    left: 2px;
    top: 2px;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    background: var(--panel);
    border: 1px solid var(--edge);
    box-shadow: 0 2px 0 var(--edge);
    box-sizing: border-box;
    transition: transform 150ms ease-out, background 150ms ease-out, border-color 150ms ease-out;
  }

  .switch-large .slider:before {
    width: 24px;
    height: 24px;
  }

  input:checked + .slider {
    background: var(--accent-soft);
    border-color: var(--accent);
  }

  input:checked + .slider:before {
    transform: translateX(22px);
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.28);
  }

  .switch-large input:checked + .slider:before {
    transform: translateX(26px);
  }

  .switch input:focus-visible + .slider {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 11px 0;
  }

  .toggle-row + .toggle-row {
    border-top: 1px solid var(--line);
  }

  .toggle-row label {
    font-size: 0.92rem;
    font-weight: 600;
  }

  /* 键帽式滑杆 */
  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 22px;
    margin: 10px 0 2px;
    background: transparent;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
    background: var(--edge);
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 20px;
    margin-top: -8px;
    border-radius: 4px;
    background: var(--panel);
    border: 1px solid var(--edge);
    box-shadow: 0 2px 0 var(--edge);
    box-sizing: border-box;
    transition: border-color 150ms ease-out, background 150ms ease-out;
  }

  input[type="range"]:hover::-webkit-slider-thumb {
    border-color: var(--accent);
  }

  input[type="range"]::-moz-range-track {
    height: 4px;
    border-radius: 2px;
    background: var(--edge);
  }

  input[type="range"]::-moz-range-thumb {
    width: 12px;
    height: 20px;
    border-radius: 4px;
    background: var(--panel);
    border: 1px solid var(--edge);
    box-shadow: 0 2px 0 var(--edge);
    box-sizing: border-box;
  }

  input[type="range"]:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
    border-radius: 4px;
  }

  /* 象限键簇 */
  .quadrant-selector {
    position: relative;
    display: grid;
    grid-template-columns: repeat(2, 78px);
    grid-template-rows: repeat(2, 54px);
    gap: 10px;
    justify-content: center;
    margin: 14px auto 6px;
  }

  .quad-btn {
    border: 1px solid var(--edge);
    background: var(--well);
    border-radius: 8px;
    box-shadow: 0 2px 0 var(--edge);
    cursor: pointer;
    padding: 0;
    transition: border-color 150ms ease-out, background 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .quad-btn:hover {
    border-color: var(--muted);
  }

  .quad-btn:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .quad-btn.active {
    border-color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 2px 0 var(--accent);
  }

  .center-cursor {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    color: var(--ink);
    background: var(--panel);
    border: 1px solid var(--edge);
    border-radius: 50%;
    box-shadow: 0 2px 0 var(--edge);
    padding: 6px;
    display: flex;
  }

  .center-cursor svg {
    stroke: var(--panel);
  }

  /* 样式选择器 */
  .style-selector {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin: 6px 0 2px;
  }

  .style-btn {
    border: 1px solid var(--edge);
    background: var(--well);
    border-radius: 9px;
    box-shadow: 0 2px 0 var(--edge);
    cursor: pointer;
    padding: 8px 8px 7px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex: 1 1 0;
    min-width: 96px;
    transition: border-color 150ms ease-out, background 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .style-btn:hover {
    border-color: var(--muted);
  }

  .style-btn:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .style-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .style-btn.active {
    border-color: var(--accent);
    background: var(--panel);
    box-shadow: 0 2px 0 var(--accent);
  }

  .style-name {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--muted);
  }

  .style-btn.active .style-name {
    color: var(--accent);
  }

  /* 主题配色色板 */
  .theme-color-selector {
    display: flex;
    gap: 10px;
    margin-top: 6px;
  }

  .theme-swatch {
    width: 64px;
    height: 44px;
    border-radius: 8px;
    border: 1px solid var(--swatch-border, var(--edge));
    background: var(--swatch-bg, #333);
    color: var(--swatch-fg, #fff);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.9rem;
    box-shadow: 0 2px 0 var(--edge);
    transition: transform 150ms ease-out, box-shadow 150ms ease-out, border-color 150ms ease-out;
  }

  .theme-swatch:hover {
    border-color: var(--muted);
  }

  .theme-swatch:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .theme-swatch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .theme-swatch.active {
    border-color: var(--accent);
    box-shadow: 0 2px 0 var(--accent), 0 0 0 3px var(--accent-soft);
  }

  .preview {
    display: flex;
    gap: 4px;
    padding: 6px 10px;
    align-items: center;
    justify-content: center;
    width: 100%;
    box-sizing: border-box;
  }

  .key-cap {
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: bold;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    box-sizing: border-box;
  }

  .preview-default .key-cap {
    background: rgba(0, 0, 0, 0.7);
    border-radius: 6px;
    padding: 2px 5px;
    color: white;
  }
  .preview-default .key-cap:first-child {
    color: #a1a1aa;
  }

  .preview-3d-key .key-cap {
    background: #fafafa;
    border: 1px solid #e0e0e0;
    border-bottom: 3px solid #b4b4b4;
    border-radius: 6px;
    padding: 2px 5px;
    color: #333;
    box-shadow: 0 2px 4px rgba(0,0,0,0.05);
  }

  .preview-cartoon .key-cap {
    background: white;
    border: 1.5px solid #000;
    border-radius: 5px;
    padding: 2px 5px;
    color: #111;
    box-shadow: 0 3px 0 0 #000;
    margin-bottom: 3px;
  }
  .preview-cartoon .key-cap:first-child {
    color: #111;
  }

  .preview-retro-terminal .key-cap {
    background: #050505;
    border: 1.5px solid #00ff41;
    border-radius: 0px;
    padding: 2px 5px;
    color: #00ff41;
    font-family: monospace;
  }
  .preview-retro-terminal .key-cap:first-child {
    color: #00cc33;
    border-color: #00cc33;
  }

  /* 字体选择 */
  .font-selector {
    width: 100%;
    margin-top: 6px;
    padding: 9px 12px;
    border: 1px solid var(--edge);
    border-radius: 8px;
    font-size: 0.9rem;
    font-family: var(--font-body);
    background: var(--well);
    color: var(--ink);
    cursor: pointer;
  }

  .font-selector:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .custom-editor-wrapper {
    margin-top: 14px;
  }

  /* 主题分段键排 */
  .theme-segmented {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }

  .theme-seg {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 9px 8px;
    border: 1px solid var(--edge);
    border-radius: 8px;
    background: var(--well);
    box-shadow: 0 2px 0 var(--edge);
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--muted);
    transition: color 150ms ease-out, border-color 150ms ease-out, background 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .theme-seg svg {
    width: 14px;
    height: 14px;
    stroke: currentColor;
    fill: none;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .theme-seg:hover {
    color: var(--ink);
    border-color: var(--muted);
  }

  .theme-seg:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .theme-seg.active {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 2px 0 var(--accent);
  }

  /* 黑名单 */
  .blacklist-input-group {
    display: flex;
    gap: 8px;
  }

  .blacklist-input-group input {
    flex: 1;
    min-width: 0;
    padding: 8px 12px;
    border: 1px solid var(--edge);
    border-radius: 8px;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    background: var(--well);
    color: var(--ink);
    outline: none;
    transition: border-color 150ms ease-out;
  }

  .blacklist-input-group input::placeholder {
    color: var(--muted);
    opacity: 0.7;
  }

  .blacklist-input-group input:focus {
    border-color: var(--accent);
  }

  .add-btn {
    background: var(--accent);
    color: var(--on-accent);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 0 16px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.85rem;
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.25);
    transition: transform 150ms ease-out, box-shadow 150ms ease-out, filter 150ms ease-out;
  }

  .add-btn:hover {
    filter: brightness(1.07);
  }

  .add-btn:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.25);
  }

  .blacklist {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 10px;
  }

  .blacklist-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--well);
    border: 1px solid var(--line);
    padding: 7px 12px;
    border-radius: 8px;
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .blacklist-empty {
    margin: 12px 0 0;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .remove-btn {
    background: transparent;
    border: none;
    color: var(--danger);
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 6px;
    border-radius: 4px;
  }

  .remove-btn:hover {
    background: rgba(179, 38, 30, 0.12);
  }

  /* Toast */
  .toast {
    position: fixed;
    bottom: 64px;
    left: 0;
    right: 0;
    margin: 0 auto;
    width: max-content;
    background: var(--ink);
    color: var(--bg);
    padding: 8px 18px;
    border-radius: 8px;
    font-size: 0.8rem;
    font-weight: 600;
    z-index: 1000;
    box-shadow: 0 3px 10px rgba(0, 0, 0, 0.25);
  }

  /* 页脚 */
  .footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 101;
    background: var(--bg);
    border-top: 1px solid var(--line);
  }

  .footer-inner {
    max-width: 640px;
    margin: 0 auto;
    padding: 10px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .footer-left {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .footer-btn {
    padding: 6px 14px;
    border-radius: 7px;
    border: 1px solid var(--edge);
    background: var(--panel);
    box-shadow: 0 2px 0 var(--edge);
    cursor: pointer;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--muted);
    transition: color 150ms ease-out, border-color 150ms ease-out,
      transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .footer-btn:hover {
    color: var(--ink);
  }

  .footer-btn:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .reset-btn {
    color: var(--danger);
    border-color: var(--danger);
    box-shadow: 0 2px 0 rgba(179, 38, 30, 0.35);
  }

  .reset-btn:hover {
    color: var(--danger);
    background: rgba(179, 38, 30, 0.08);
  }

  .confirm-dialog {
    position: absolute;
    bottom: calc(100% + 10px);
    left: 0;
    background: var(--panel);
    border: 1px solid var(--line);
    padding: 10px 14px;
    border-radius: 10px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
    white-space: nowrap;
    z-index: 200;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.82rem;
    color: var(--muted);
  }

  .confirm-yes {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--danger);
    background: var(--danger);
    color: #fff;
    cursor: pointer;
    font-size: 0.78rem;
    font-weight: 600;
  }

  .confirm-no {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--edge);
    background: var(--well);
    color: var(--ink);
    cursor: pointer;
    font-size: 0.78rem;
  }

  /* 关于模态 */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 300;
    background: var(--scrim);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .about-dialog {
    width: 320px;
    max-width: 100%;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 12px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.28);
    overflow: hidden;
  }

  .about-head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 12px 11px 18px;
    border-bottom: 1px solid var(--line);
  }

  .about-close {
    margin-left: auto;
    width: 26px;
    height: 26px;
    border: 1px solid var(--edge);
    border-radius: 6px;
    background: var(--well);
    box-shadow: 0 2px 0 var(--edge);
    color: var(--muted);
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    transition: color 150ms ease-out, transform 150ms ease-out, box-shadow 150ms ease-out;
  }

  .about-close:hover {
    color: var(--ink);
  }

  .about-close:active {
    transform: translateY(1px);
    box-shadow: 0 1px 0 var(--edge);
  }

  .about-body {
    padding: 22px 18px 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }

  .about-brand-visual {
    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: 2px;
  }

  .about-logo {
    width: 68px;
    height: 68px;
    border-radius: 16px;
    object-fit: cover;
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.28), 0 0 0 1px var(--edge);
    transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .about-logo:hover {
    transform: scale(1.06) rotate(1deg);
  }

  .about-name {
    font-family: var(--font-mono);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.22em;
  }

  .about-meta {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .about-tag {
    font-size: 0.78rem;
    color: var(--muted);
  }

  .about-foot {
    padding: 4px 18px 18px;
    display: flex;
    justify-content: center;
  }

  button:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* 尊重系统减弱动效设置：关闭位移类微交互 */
  @media (prefers-reduced-motion: reduce) {
    .tab-key, .tab-key:active, .tab-key.active,
    .quad-btn, .quad-btn:active,
    .style-btn, .style-btn:active,
    .theme-swatch, .theme-swatch:active,
    .theme-seg, .theme-seg:active,
    .btn-edit-custom, .btn-edit-custom:active,
    .add-btn, .add-btn:active,
    .about-close, .about-close:active,
    .footer-btn, .footer-btn:active {
      transform: none;
      transition: border-color 150ms ease-out, background 150ms ease-out, color 150ms ease-out;
    }
  }
</style>
