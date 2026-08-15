<div align="center">

<img src="../osd-bubble/static/logo.png" alt="OSD Bubble Logo" width="128" height="128" />

# OSD Bubble

**A Modern, Lightweight & Aesthetic Desktop Keystroke & Mouse OSD Overlay**

Tailored for **Screen Recording · Live Streaming · Online Tutorials · Presentations**

[![Release](https://img.shields.io/badge/release-v1.0.0-blue.svg?style=flat-square)](https://github.com/)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2B-0078D6.svg?style=flat-square&logo=windows)](https://github.com/)
[![Rust](https://img.shields.io/badge/Rust-1.77%2B-DEA584.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2.0-24C8DB.svg?style=flat-square&logo=tauri)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-v5.0-FF3E00.svg?style=flat-square&logo=svelte)](https://svelte.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)

[English](./README_EN.md) | [简体中文](../README.md)

</div>

---

## ✨ Features

- ⚡ **Zero-Latency Low-Level Hook**: Raw Win32 global input capture for keystrokes, shortcuts, mouse clicks, and wheel scrolling.
- 🎨 **Hardware-Accelerated Layered Window**: Built with Rust + `Tiny-Skia` on top of Windows `WS_EX_LAYERED` layered window for butter-smooth antialiased rendering with zero webview overhead.
- 🌊 **Mouse Click Glow & Ripples**: 260ms reactive ripple effects for Left (Cyan), Right (Amber), and Middle (Mint Green) clicks.
- 📜 **Keystroke History Waterfall Stream**: Smooth multi-queue display (2 / 3 / 4 items) with independent row decay and lifecycles.
- 🔢 **Smart Combo Multiplier Badge**: 600ms automatic combo detection with Keyviz-style bouncing pill badges (`×2`, `×3`...).
- 🖥️ **Dual Positioning Modes & Multi-Monitor Snapping**:
  - **Follow Mouse Mode**: Dynamic positioning around the cursor in 4 quadrants.
  - **Fixed Screen Anchor Mode**: Lock to 6 screen corners (Bottom Right, Bottom Center, Bottom Left, Top Right, Top Center, Top Left) with multi-monitor smart snapping.
- 🎈 **Rich Easing Physics Animations**: Built-in `bounce`, `slide_up`, `fade`, and `instant` curves.
- 🛡️ **Foreground Process Blocklist**: Automatic silence when targeted games or fullscreen apps (e.g., `csgo.exe`) are in focus.
- 🧠 **Position Memory & Auto Centering**: Smart initial center positioning with persistent coordinates across reboots.
- 🚀 **Ultra-Lightweight & Portable**: Single portable standalone `.exe` (~13MB) with minimal memory footprint (< 25MB).

---

## 📥 Installation

Download the latest version from the [Releases page](https://github.com/bdrwss/osd-bubble/releases):

- **Portable Standalone (`osd-bubble.exe`)**: No installation required, run directly anywhere.
- **Installer Setup (`osd-bubble_1.0.0_x64-setup.exe`)**: Standard Windows installer with start menu & autostart integration.

---

## ⌨️ Shortcuts

- `Ctrl + Shift + ,`: Toggle Settings Window
- `Ctrl + Shift + K`: Pause / Resume Keystroke Display
- `Esc`: Close dialogs / About window

---

## 📄 License

Distributed under the [MIT License](../LICENSE).

---

## 👤 Developer

- **Developer**: **摆渡人吾师 (Baiduren Wushi)**
