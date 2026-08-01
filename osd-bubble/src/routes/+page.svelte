<script>
  import { onMount } from 'svelte';
  import { slide, fade, scale } from 'svelte/transition';
  import { browser } from '$app/environment';
  import { message } from '@tauri-apps/plugin-dialog';
  import CustomStyleEditor from '$lib/components/CustomStyleEditor.svelte';
  import { invoke } from '@tauri-apps/api/core';

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
    text_color: '#ffffff'
  };
  let enabled = true;
  let showKeyboard = true;
  let showMouse = true;
  let showScroll = false;
  let theme = 'system';
  let excludeApps = [];
  let newAppInput = '';
  let autoStart = false;
  let toastVisible = false;
  let toastMessage = '';
  let currentTab = 'bubble';
  let showResetConfirm = false;
  let showCustomEditor = false;
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
  onMount(() => {
    if (browser) {
      const saved = localStorage.getItem('osdBubbleSettings');
      if (saved) {
        try {
          const settings = JSON.parse(saved);
          fadeDelay = settings.fadeDelay || 1000;
          opacity = settings.opacity || 85;
          quadrant = settings.quadrant || '3';
          bubbleScale = settings.bubbleScale || 120;
          fontFamily = settings.fontFamily || "'Microsoft YaHei UI'";
          bubbleStyle = settings.bubbleStyle || 'default';
          customStyle = settings.customStyle || customStyle;
          enabled = settings.enabled !== undefined ? settings.enabled : true;
          showKeyboard = settings.showKeyboard !== undefined ? settings.showKeyboard : true;
          showMouse = settings.showMouse !== undefined ? settings.showMouse : true;
          showScroll = settings.showScroll !== undefined ? settings.showScroll : false;
          theme = settings.theme || 'system';
          excludeApps = settings.excludeApps || [];
          autoStart = settings.autoStart || false;
        } catch (e) {
          console.error('Failed to load settings:', e);
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

  // Save settings
  function saveSettings() {
    if (!browser) return;
    
    const settings = {
      fadeDelay,
      opacity,
      quadrant,
      bubbleScale,
      fontFamily,
      bubbleStyle,
      customStyle,
      enabled,
      showKeyboard,
      showMouse,
      showScroll,
      theme,
      excludeApps,
      autoStart
    };
    localStorage.setItem('osdBubbleSettings', JSON.stringify(settings));

    // Sync to rust backend
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      invoke('update_settings', { fadeDelay });
      invoke('update_opacity', { opacity: opacity / 100 });
      invoke('update_position', { quadrant: parseInt(quadrant) });
      invoke('update_bubble_style', { style: bubbleStyle });
      invoke('update_custom_style', { style: customStyle });
      invoke('toggle_enabled', { enabled });
      invoke('update_show_keyboard', { show: showKeyboard });
      invoke('update_show_mouse', { show: showMouse });
      invoke('update_show_scroll', { show: showScroll });
      invoke('update_exclude_apps', { apps: excludeApps });
    }
  }

  $: {
    fadeDelay; opacity; quadrant; bubbleScale; fontFamily; bubbleStyle; customStyle; enabled; showKeyboard; showMouse; showScroll; theme; excludeApps; autoStart;
    if (isLoaded && browser) {
      saveSettings();
    }
  }


  const stylePresets = {
    'default': { bg_color: '#000000', bg_opacity: 0.7, border_width: 0, border_color: '#000000', radius: 8, text_color: '#ffffff', shadow_color: '#000000' },
    '3d_key': { bg_color: '#fafafa', bg_opacity: 1.0, border_width: 1.0, border_color: '#e0e0e0', radius: 8, text_color: '#212121', shadow_color: '#b4b4b4' },
    'cartoon': { bg_color: '#ffffff', bg_opacity: 1.0, border_width: 1.5, border_color: '#000000', radius: 16, text_color: '#111111', shadow_color: '#000000' },
    'retro_terminal': { bg_color: '#050505', bg_opacity: 0.95, border_width: 2.0, border_color: '#00ff41', radius: 0, text_color: '#00ff41', shadow_color: '#000000' }
  };

  // Set style and preview
  function setStyleAndPreview(style) {
    bubbleStyle = style;
    customStyle = { ...stylePresets[style] };
    if (style === 'retro_terminal') {
      fontFamily = "monospace";
    }
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
      text_color: '#ffffff'
    };
    enabled = true;
    showKeyboard = true;
    showMouse = true;
    showScroll = false;
    theme = 'system';
    excludeApps = [];
    autoStart = false;
    saveSettings();
    showResetConfirm = false;
    showToast('已恢复默认设置');
  }

  // App version
  const appVersion = '1.0.0';
</script>

<div class="app-shell">
  

  <div class="tab-navigation">
    <div class="tab-container">
      <button 
        class="tab-btn {currentTab === 'bubble' ? 'active' : ''}" 
        onclick={() => currentTab = 'bubble'}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <path d="M12 8v8"></path>
          <path d="M8 12h8"></path>
        </svg>
        气泡设置
      </button>
      <button 
        class="tab-btn {currentTab === 'other' ? 'active' : ''}" 
        onclick={() => currentTab = 'other'}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1.37.54A2 2 0 0 0 7 6.44V6a2 2 0 0 0-2-2H4.78A2 2 0 0 0 3 7.78v.44a2 2 0 0 1-.54 1.37A2 2 0 0 0 2 10.22v.44a2 2 0 0 0 2 2h.44a2 2 0 0 1 .54 1.37A2 2 0 0 0 7 16.22v.44a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1.37-.54A2 2 0 0 0 17 12.56V13a2 2 0 0 0 2-2h.44a2 2 0 0 0 2-2v-.44a2 2 0 0 0-2-2h-.44a2 2 0 0 1-.54-1.37A2 2 0 0 0 17 7.78V7.34a2 2 0 0 0-2-2z"></path>
          <circle cx="12" cy="12" r="3"></circle>
        </svg>
        其他设置
      </button>
    </div>
  </div>

  <main class="scroll-area">
    <div class="container">
      {#if toastVisible}
        <div class="toast" transition:fade={{ duration: 200 }}>
          {toastMessage}
        </div>
      {/if}
      
      {#if currentTab === 'bubble'}
        <div class="tab-content" transition:slide={{ duration: 200 }}>
          <div class="setting-card">
            <div class="setting-header">
              <label for="fade-delay">气泡显示时长</label>
              <span class="value">{fadeDelay} ms</span>
            </div>
            <input id="fade-delay" type="range" min="300" max="5000" step="100" bind:value={fadeDelay} />
            <p class="description">操作停止后气泡继续显示的时间</p>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label for="opacity">气泡不透明度</label>
              <span class="value">{opacity}%</span>
            </div>
            <input id="opacity" type="range" min="40" max="100" step="5" bind:value={opacity} />
            <p class="description">调节气泡整体的透明程度</p>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label>气泡默认位置</label>
            </div>
            <div class="quadrant-selector">
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

          <div class="setting-card">
            <div class="setting-header">
              <label for="bubble-scale">气泡缩放比例</label>
              <span class="value">{bubbleScale}%</span>
            </div>
            <input id="bubble-scale" type="range" min="80" max="200" step="5" bind:value={bubbleScale} />
            <p class="description">调整气泡的整体大小</p>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label>字体选择</label>
            </div>
            <select class="font-selector" bind:value={fontFamily}>
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

          <div class="setting-card">
            <div class="setting-header">
              <label>气泡样式</label>
              <button class="btn-edit-custom {showCustomEditor ? 'active' : ''}" onclick={() => showCustomEditor = !showCustomEditor}>
                {showCustomEditor ? '收起自定义' : '自定义配置'}
              </button>
            </div>
            <div class="style-selector">
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
              <div class="custom-editor-wrapper" transition:slide={{ duration: 250 }}>
                <CustomStyleEditor bind:customStyle bubbleStyle={bubbleStyle} onReset={handleReset} />
              </div>
            {/if}
            <p class="description">选择按键气泡的视觉风格</p>
          </div>
        </div>
      {/if}

      {#if currentTab === 'other'}
        <div class="tab-content" transition:slide={{ duration: 200 }}>
          <div class="setting-card enable-card">
            <div class="setting-header">
              <label class="enable-label">
                <span class="enable-icon">{enabled ? '✅' : '⏸️'}</span>
                {enabled ? '已启用' : '已暂停'}
              </label>
              <label class="switch switch-large">
                <input type="checkbox" bind:checked={enabled} />
                <span class="slider round"></span>
              </label>
            </div>
            <p class="description">关闭后暂停所有按键气泡显示</p>
          </div>

          <div class="setting-card">
            <div class="toggle-row">
              <label>键盘按键</label>
              <label class="switch">
                <input type="checkbox" bind:checked={showKeyboard} />
                <span class="slider round"></span>
              </label>
            </div>
            <div class="toggle-row">
              <label>鼠标点击</label>
              <label class="switch">
                <input type="checkbox" bind:checked={showMouse} />
                <span class="slider round"></span>
              </label>
            </div>
            <div class="toggle-row">
              <label>滚轮操作</label>
              <label class="switch">
                <input type="checkbox" bind:checked={showScroll} />
                <span class="slider round"></span>
              </label>
            </div>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label>外观主题</label>
            </div>
            <div class="theme-segmented">
              <button class="theme-seg {theme === 'dark' ? 'active' : ''}" onclick={() => { theme = 'dark'; }}>🌙 深色</button>
              <button class="theme-seg {theme === 'light' ? 'active' : ''}" onclick={() => { theme = 'light'; }}>☀️ 浅色</button>
              <button class="theme-seg {theme === 'system' ? 'active' : ''}" onclick={() => { theme = 'system'; }}>💻 跟随系统</button>
            </div>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label>黑名单应用</label>
            </div>
            <p class="description">以下应用中不会显示按键气泡</p>
            <div class="blacklist-input-group">
              <input type="text" placeholder="例如：csgo.exe" bind:value={newAppInput} onkeydown={(e) => e.key === 'Enter' && addExcludeApp()} />
              <button class="add-btn" onclick={addExcludeApp}>添加</button>
            </div>
            <div class="blacklist">
              {#each excludeApps as app}
                <div class="blacklist-item">
                  <span>{app}</span>
                  <button class="remove-btn" onclick={() => removeExcludeApp(app)}>×</button>
                </div>
              {/each}
              {#if excludeApps.length === 0}
                <p class="description" style="text-align: center; margin-top: 1rem;">当前没有添加黑名单</p>
              {/if}
            </div>
          </div>

          <div class="setting-card">
            <div class="setting-header">
              <label>开机自启动</label>
              <label class="switch">
                <input type="checkbox" bind:checked={autoStart} onchange={toggleAutoStart} />
                <span class="slider round"></span>
              </label>
            </div>
            <p class="description">开机后自动在后台运行</p>
          </div>
        </div>
      {/if}
    </div>
  </main>

  <footer class="footer">
    <div class="footer-inner">
      <div class="footer-left">
        <button class="footer-btn reset-btn" onclick={() => showResetConfirm = true}>恢复默认设置</button>
        {#if showResetConfirm}
          <div class="confirm-dialog">
            <span>确定恢复默认设置？</span>
            <button class="confirm-yes" onclick={resetDefaults}>确定</button>
            <button class="confirm-no" onclick={() => showResetConfirm = false}>取消</button>
          </div>
        {/if}
      </div>
      <div class="footer-right">
        <button class="footer-btn about-btn" onclick={async () => {
          await message(`OSD Bubble v${appVersion}\n按键可视化工具`, { title: '关于 OSD Bubble', kind: 'info' });
        }}>关于</button>
      </div>
    </div>
  </footer>
</div>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: #333;
    background-color: #f5f5f5;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background-color: #f5f5f5;
    overflow: hidden;
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: visible;
    position: relative;
  }

  .scroll-area {
    position: fixed;
    top: 90px;
    bottom: 56px;
    left: 0;
    right: 0;
    overflow-y: auto;
    z-index: 1;
    padding-right: 8px;
  }

  .scroll-area::-webkit-scrollbar {
    width: 8px;
    background: transparent;
    position: absolute;
    z-index: 10000;
  }

  .scroll-area::-webkit-scrollbar-track {
    background: transparent;
    position: absolute;
    z-index: 10000;
  }

  .scroll-area::-webkit-scrollbar-thumb {
    background-color: #ccc;
    border-radius: 4px;
    border: 2px solid transparent;
    background-clip: content-box;
    position: absolute;
    z-index: 10001;
  }

  .scroll-area::-webkit-scrollbar-thumb:hover {
    background-color: #aaa;
  }

  :root[data-theme="dark"] .scroll-area::-webkit-scrollbar-thumb {
    background-color: #555;
    position: absolute;
    z-index: 10001;
  }
  :root[data-theme="dark"] .scroll-area::-webkit-scrollbar-thumb:hover {
    background-color: #777;
    position: absolute;
    z-index: 10001;
  }

  .container {
    max-width: 600px;
    margin: 0 auto;
    padding: 1.5rem 1.75rem 2rem;
  }

  /* Tab Navigation */
  .tab-navigation {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background: #f5f5f5;
    z-index: 50;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
    padding: 1.5rem 1.75rem 1rem;
  }
  
  :root[data-theme="dark"] .tab-navigation {
    background: #1e1e1e;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }
  
  :root[data-theme="light"] .tab-navigation {
    background: #f5f5f5;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  }

  .tab-container {
    max-width: 600px;
    margin: 0 auto;
    display: flex;
    gap: 12px;
  }

  .tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 16px;
    border: none;
    background: transparent;
    border-radius: 10px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 600;
    color: #555;
    transition: all 0.2s ease;
  }
  
  :root[data-theme="dark"] .tab-btn {
    color: #ccc;
  }
  
  :root[data-theme="light"] .tab-btn {
    color: #555;
  }

  .tab-btn svg {
    width: 18px;
    height: 18px;
    stroke: currentColor;
    stroke-width: 2;
    fill: none;
  }

  .tab-btn:hover {
    background: rgba(0, 122, 255, 0.08);
    color: #007aff;
  }
  :root[data-theme="dark"] .tab-btn:hover {
    background: rgba(10, 132, 255, 0.12);
    color: #0a84ff;
  }

  .tab-btn.active {
    background: linear-gradient(135deg, rgba(0, 122, 255, 0.15), rgba(0, 122, 255, 0.2));
    color: #007aff;
    box-shadow: 0 2px 8px rgba(0, 122, 255, 0.15);
  }
  :root[data-theme="dark"] .tab-btn.active {
    background: linear-gradient(135deg, rgba(10, 132, 255, 0.2), rgba(10, 132, 255, 0.25));
    color: #0a84ff;
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.2);
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }


  /* Setting Card */
  .setting-card {
    background: white;
    padding: 1.25rem 1.5rem;
    border-radius: 12px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.05);
  }

  .setting-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  label {
    font-size: 1rem;
    font-weight: 500;
  }

  .value {
    font-family: monospace;
    font-size: 1rem;
    color: #666;
    background: #f0f0f0;
    padding: 0.15rem 0.5rem;
    border-radius: 6px;
  }

  .description {
    font-size: 0.85rem;
    color: #888;
    margin: 0;
    margin-top: 0.5rem;
  }

  .btn-edit-custom {
    font-size: 0.8rem;
    padding: 4px 10px;
    border-radius: 4px;
    border: 1px solid #d0d0d0;
    background: transparent;
    color: #555;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .btn-edit-custom:hover {
    background: #f5f5f5;
  }
  .btn-edit-custom.active {
    background: #eef5ff;
    border-color: #007aff;
    color: #007aff;
  }

  /* Enable Card */
  .enable-card {
    border: 2px solid rgba(0, 122, 255, 0.2);
    background: linear-gradient(135deg, rgba(0,122,255,0.03), rgba(0,122,255,0.06));
  }

  .enable-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .enable-icon {
    font-size: 1.2rem;
  }

  .enable-card .description {
    color: #999;
    font-size: 0.8rem;
  }

  /* Toggle Switch */
  .switch {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 26px;
    flex-shrink: 0;
  }

  .switch-large {
    width: 52px;
    height: 30px;
  }

  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0; left: 0; right: 0; bottom: 0;
    background-color: #ccc;
    transition: .3s;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 20px;
    width: 20px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: .3s;
  }

  .switch-large .slider:before {
    height: 24px;
    width: 24px;
  }

  input:checked + .slider {
    background-color: #007aff;
  }

  input:checked + .slider:before {
    transform: translateX(18px);
  }

  .switch-large input:checked + .slider:before {
    transform: translateX(22px);
  }

  .slider.round {
    border-radius: 34px;
  }

  .slider.round:before {
    border-radius: 50%;
  }

  /* Toggle Row */
  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
  }

  .toggle-row + .toggle-row {
    border-top: 1px solid #f0f0f0;
    padding-top: 12px;
  }

  .toggle-row label {
    font-size: 0.938rem;
  }

  /* Range Slider */
  input[type="range"] {
    width: 100%;
    margin-top: 0.25rem;
    margin-bottom: 0.5rem;
    accent-color: #007aff;
  }

  /* Quadrant Selector */
  .quadrant-selector {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    gap: 10px;
    width: 200px;
    height: 140px;
    margin: 1rem auto;
  }

  .quad-btn {
    border: 2px solid #eaeaea;
    background: #fafafa;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s ease;
    padding: 0;
  }

  .quad-btn:hover {
    border-color: #ccc;
    background: #f0f0f0;
  }

  .quad-btn.active {
    border-color: #007aff;
    background: rgba(0, 122, 255, 0.08);
    box-shadow: 0 0 0 1px #007aff;
  }

  .center-cursor {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    color: #444;
    background: white;
    border-radius: 50%;
    box-shadow: 0 3px 10px rgba(0,0,0,0.12);
    padding: 6px;
    display: flex;
    justify-content: center;
    align-items: center;
    border: 1px solid rgba(0,0,0,0.05);
  }

  /* Style Selector */
  .style-selector {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 0.75rem;
  }

  .style-btn {
    border: 2px solid #eaeaea;
    background: #fafafa;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s ease;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex: 1 1 0;
    min-width: 90px;
  }

  .style-btn:hover {
    border-color: #ccc;
    background: #f0f0f0;
  }

  .style-btn.active {
    border-color: #007aff;
    background: rgba(0, 122, 255, 0.04);
    box-shadow: 0 0 0 1px #007aff;
  }

  .style-name {
    font-size: 0.85rem;
    font-weight: 500;
    color: #333;
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
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
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

  .preview-custom {
    padding: 2px 10px;
    height: 28px;
    box-sizing: border-box;
  }

  /* Font Selector */
  .font-selector {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-size: 0.95rem;
    background: white;
    cursor: pointer;
  }
  
  :root[data-theme="dark"] .font-selector {
    background: #2a2a2a;
    border-color: #444;
    color: #eee;
  }

  /* Custom Editor Wrapper */
  .custom-editor-wrapper {
    margin-top: 16px;
  }

  /* Theme Segmented */
  .theme-segmented {
    display: flex;
    gap: 0;
    border: 2px solid #eaeaea;
    border-radius: 10px;
    overflow: hidden;
  }

  .theme-seg {
    flex: 1;
    padding: 10px 8px;
    border: none;
    background: #fafafa;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    color: #555;
    transition: all 0.2s ease;
    border-right: 1px solid #eaeaea;
  }

  .theme-seg:last-child {
    border-right: none;
  }

  .theme-seg:hover {
    background: #f0f0f0;
  }

  .theme-seg.active {
    background: #007aff;
    color: white;
  }

  /* Blacklist */
  .blacklist-input-group {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
  }

  .blacklist-input-group input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-size: 0.95rem;
    outline: none;
  }

  .blacklist-input-group input:focus {
    border-color: #007aff;
  }

  .add-btn {
    background: #007aff;
    color: white;
    border: none;
    padding: 0 16px;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
  }

  .add-btn:hover {
    background: #0066d6;
  }

  .blacklist {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .blacklist-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #fafafa;
    border: 1px solid #eee;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 0.9rem;
  }

  .remove-btn {
    background: transparent;
    border: none;
    color: #ff3b30;
    font-size: 1.2rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 6px;
    border-radius: 4px;
  }

  .remove-btn:hover {
    background: rgba(255, 59, 48, 0.1);
  }

  /* Toast */
  .toast {
    position: fixed;
    bottom: 70px;
    left: 50%;
    transform: translateX(-50%);
    background: #4CAF50;
    color: white;
    padding: 10px 24px;
    border-radius: 8px;
    font-size: 14px;
    z-index: 1000;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  }

  /* Footer */
  .footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 101;
    background: #f5f5f5;
    border-top: 1px solid #eaeaea;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.06);
  }
  
  :root[data-theme="dark"] .footer {
    background: #1e1e1e;
    border-color: #444;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.2);
  }
  
  :root[data-theme="light"] .footer {
    background: #f5f5f5;
    border-color: #eaeaea;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.06);
  }

  .footer-inner {
    max-width: 600px;
    margin: 0 auto;
    padding: 0.75rem 1.75rem;
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
    padding: 6px 16px;
    border-radius: 8px;
    border: 1px solid #ddd;
    background: #fafafa;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    color: #555;
    transition: all 0.2s ease;
  }

  .footer-btn:hover {
    background: #f0f0f0;
    border-color: #ccc;
  }

  .reset-btn {
    color: #ff3b30;
    border-color: rgba(255, 59, 48, 0.3);
  }

  .reset-btn:hover {
    background: rgba(255, 59, 48, 0.08);
  }

  .confirm-dialog {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    background: white;
    padding: 10px 14px;
    border-radius: 10px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    white-space: nowrap;
    z-index: 200;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: #555;
  }

  .confirm-yes {
    padding: 4px 12px;
    border-radius: 6px;
    border: none;
    background: #ff3b30;
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .confirm-no {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid #ddd;
    background: #fafafa;
    cursor: pointer;
    font-size: 0.8rem;
  }

  /* Dark theme */
  @media (prefers-color-scheme: dark) {
    :global(body) {
      background-color: #1e1e1e;
    }
    :root {
      color: #eee;
      background-color: #1e1e1e;
    }
  }

  :root[data-theme="dark"] {
    color: #eee;
    background-color: #1e1e1e;
  }
  :root[data-theme="dark"] ~ :global(body) {
    background-color: #1e1e1e;
  }

  :root[data-theme="light"] {
    color: #333;
    background-color: #f5f5f5;
  }
  :root[data-theme="light"] ~ :global(body) {
    background-color: #f5f5f5;
  }

  :root[data-theme="dark"] .setting-card {
    background: #2a2a2a;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
  }

  :root[data-theme="dark"] .enable-card {
    border-color: rgba(10, 132, 255, 0.3);
    background: linear-gradient(135deg, rgba(10,132,255,0.05), rgba(10,132,255,0.1));
  }

  :root[data-theme="dark"] .toggle-row + .toggle-row {
    border-color: #3a3a3a;
  }

  :root[data-theme="dark"] .quad-btn {
    border-color: #444;
    background: #2a2a2a;
  }
  :root[data-theme="dark"] .quad-btn:hover {
    border-color: #555;
    background: #333;
  }
  :root[data-theme="dark"] .quad-btn.active {
    border-color: #0a84ff;
    background: rgba(10, 132, 255, 0.15);
    box-shadow: 0 0 0 1px #0a84ff;
  }
  :root[data-theme="dark"] .center-cursor {
    background: #3a3a3a;
    color: #eee;
    border-color: rgba(255,255,255,0.1);
  }

  :root[data-theme="dark"] .value {
    background: #333;
    color: #ccc;
  }

  :root[data-theme="dark"] .style-btn {
    border-color: #444;
    background: #2a2a2a;
  }
  :root[data-theme="dark"] .style-btn:hover {
    border-color: #555;
    background: #333;
  }
  :root[data-theme="dark"] .style-btn.active {
    border-color: #0a84ff;
    background: rgba(10, 132, 255, 0.1);
    box-shadow: 0 0 0 1px #0a84ff;
  }
  :root[data-theme="dark"] .style-name { color: #eee; }

  :root[data-theme="dark"] .theme-segmented {
    border-color: #444;
  }
  :root[data-theme="dark"] .theme-seg {
    background: #2a2a2a;
    color: #ccc;
    border-color: #444;
  }
  :root[data-theme="dark"] .theme-seg:hover {
    background: #333;
  }
  :root[data-theme="dark"] .theme-seg.active {
    background: #0a84ff;
    color: white;
  }

  :root[data-theme="dark"] .blacklist-input-group input {
    background: #333;
    border-color: #555;
    color: #eee;
  }

  :root[data-theme="dark"] .blacklist-item {
    background: #333;
    border-color: #444;
  }

  :root[data-theme="dark"] .footer {
    background: #1e1e1e;
    border-color: #3a3a3a;
  }
  :root[data-theme="dark"] .footer-btn {
    background: #2a2a2a;
    border-color: #444;
    color: #ccc;
  }
  :root[data-theme="dark"] .footer-btn:hover {
    background: #333;
  }
  :root[data-theme="dark"] .confirm-dialog {
    background: #2a2a2a;
    color: #ccc;
  }
  :root[data-theme="dark"] .confirm-no {
    background: #333;
    border-color: #555;
    color: #ccc;
  }

  :root[data-theme="dark"] .preview-3d-key .key-cap {
    background: #444;
    border: 1px solid #555;
    border-bottom: 3px solid #222;
    color: #eee;
  }
</style>
