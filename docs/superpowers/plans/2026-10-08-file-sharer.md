# Local File Sharer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight Windows desktop application using Tauri v2, Rust backend, and Svelte 4 frontend that allows users to share any file directly from their PC via a public link (Cloudflare Quick Tunnel) or local LAN link, with deep Windows right-click Context Menu integration, download tracking, and GUI layout mirroring `local-video-streamer`.

**Architecture:** A native Rust engine runs an embedded Tokio/Axum HTTP server providing HTTP 206 Byte-Range streaming directly from disk for ultra-fast downloads with IDM & resume support. Cloudflare Quick Tunnel child process provides zero-config HTTPS public URLs. Windows shell registry keys register "แชร์ลิ้ง" in File Explorer, and Tauri single-instance IPC brings the window to the front and adds the file. A Svelte 4 + Tailwind CSS + esbuild frontend mirrors the complete UI cards and settings of `local-video-streamer`.

**Tech Stack:** 
- Backend: Rust 2021, Tauri v2, `axum`, `tokio`, `serde`, `winreg`, `tauri-plugin-single-instance`, `tauri-plugin-dialog`, `reqwest`
- Frontend: Svelte 4, TypeScript, Tailwind CSS, esbuild
- Tunneling: Cloudflare Tunnel (`cloudflared`)

**Spec:** `docs/superpowers/specs/2026-10-08-file-sharer-design.md`

## Global Constraints
- Target platform: Windows 10 / 11 (64-bit)
- Windows Context Menu MUST use `HKCU\Software\Classes\*\shell\FileSharer` so it requires NO administrator privileges.
- Zero server file upload: files are served directly from the user's local disk.
- HTTP Byte-Range requests (`Range: bytes=...`) MUST be supported with 206 Partial Content.
- Physical file paths MUST remain private; public URLs access files solely via random tokens `f_<hex>`.
- UI structure and settings MUST match the visual and structural design of `local-video-streamer`.

---

### Task 1: Project Scaffolding & Dependencies

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Create: `build.js`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: Initial project buildable environment with `npm install` and `cargo check`.

- [ ] **Step 1: Write package.json with Svelte 4, esbuild, Tailwind, and Tauri v2 dependencies**
- [ ] **Step 2: Write tsconfig.json, tailwind.config.js, postcss.config.js, and build.js**
- [ ] **Step 3: Write src-tauri/Cargo.toml with Tauri v2, axum, tokio, serde, winreg, and dependencies**
- [ ] **Step 4: Write src-tauri/tauri.conf.json**
- [ ] **Step 5: Run npm install to verify dependencies resolve**
- [ ] **Step 6: Commit**

```bash
git add package.json tsconfig.json tailwind.config.js postcss.config.js build.js src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: scaffold project structure with Tauri v2, Svelte 4, and dependencies"
```

---

### Task 2: Rust Core - Share Registry & Persistence (`registry.rs`)

**Files:**
- Create: `src-tauri/src/registry.rs`
- Create: `src-tauri/tests/registry_test.rs`

**Interfaces:**
- Produces: `struct ShareItem`, `struct VideoRegistry` (or `FileRegistry`) with:
  - `register(file_path: &str, title: Option<String>, description: Option<String>) -> Result<ShareItem, String>`
  - `get(id: &str) -> Option<ShareItem>`
  - `list() -> Vec<ShareItem>`
  - `update_meta(id: &str, title: Option<String>, description: Option<String>) -> Result<ShareItem, String>`
  - `revoke(id: &str) -> bool`
  - `increment_views(id: &str)`
  - `increment_downloads(id: &str)`
  - `set_short_url(id: &str, short_url: String)`

- [ ] **Step 1: Write failing unit test in src-tauri/tests/registry_test.rs**
- [ ] **Step 2: Run cargo test to verify it fails**
- [ ] **Step 3: Implement src-tauri/src/registry.rs with disk persistence to data/shares.json**
- [ ] **Step 4: Run cargo test to verify all registry tests pass**
- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/registry.rs src-tauri/tests/registry_test.rs
git commit -m "feat(backend): implement file registry and persistence"
```

---

### Task 3: Rust Core - File Streaming & Byte-Range Engine (`stream.rs`)

**Files:**
- Create: `src-tauri/src/stream.rs`
- Create: `src-tauri/tests/stream_test.rs`

**Interfaces:**
- Consumes: `std::fs::File`, `std::path::Path`
- Produces:
  - `parse_range_header(range_str: &str, file_size: u64) -> Option<(u64, u64)>`
  - `build_stream_response(file_path: &str, range_header: Option<&str>, download_name: &str) -> Result<axum::response::Response, (axum::http::StatusCode, String)>`

- [ ] **Step 1: Write failing unit tests for range header parsing and content-range calculations**
- [ ] **Step 2: Run cargo test to verify it fails**
- [ ] **Step 3: Implement stream.rs with chunked file streaming and HTTP 206 Partial Content**
- [ ] **Step 4: Run cargo test to verify tests pass**
- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/stream.rs src-tauri/tests/stream_test.rs
git commit -m "feat(backend): implement HTTP 206 byte-range streaming engine"
```

---

### Task 4: Rust Core - HTTP Web Server & Public Downloader Page (`server.rs`)

**Files:**
- Create: `src-tauri/src/server.rs`
- Create: `src-tauri/tests/server_test.rs`

**Interfaces:**
- Consumes: `FileRegistry`, `stream.rs`, `tunnel.rs`
- Produces:
  - `create_app(state: Arc<AppState>) -> axum::Router`
  - Routes:
    - `GET /d/:id`: HTML Landing Page with Discord/OpenGraph meta tags and Download button
    - `GET /api/download/:id`: Streaming file download endpoint
    - `GET /api/status`: Status of server & tunnel
    - `GET /api/shares`: JSON list of active shares

- [ ] **Step 1: Write failing tests for /d/:id and /api/download/:id routes in server_test.rs**
- [ ] **Step 2: Run cargo test to verify it fails**
- [ ] **Step 3: Implement server.rs with HTML landing page generation and axum routes**
- [ ] **Step 4: Run cargo test to verify tests pass**
- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/server.rs src-tauri/tests/server_test.rs
git commit -m "feat(backend): implement axum HTTP server and download landing page"
```

---

### Task 5: Rust Core - Cloudflare Tunnel & URL Shortener (`tunnel.rs` & `shortener.rs`)

**Files:**
- Create: `src-tauri/src/tunnel.rs`
- Create: `src-tauri/src/shortener.rs`
- Create: `src-tauri/tests/shortener_test.rs`

**Interfaces:**
- Produces:
  - `struct TunnelManager`: manages `cloudflared` process lifecycle, extracts `https://*.trycloudflare.com` URL, status (`offline`, `starting`, `online`, `error`)
  - `shorten_url(url: &str) -> Result<String, String>`: calls spoo.me or da.gd API

- [ ] **Step 1: Write failing tests for shortener and tunnel manager state**
- [ ] **Step 2: Run cargo test to verify it fails**
- [ ] **Step 3: Implement tunnel.rs and shortener.rs**
- [ ] **Step 4: Run cargo test to verify tests pass**
- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/tunnel.rs src-tauri/src/shortener.rs src-tauri/tests/shortener_test.rs
git commit -m "feat(backend): implement Cloudflare tunnel manager and URL shortener"
```

---

### Task 6: Rust Core - Windows Context Menu & Settings (`windows.rs` & `settings.rs`)

**Files:**
- Create: `src-tauri/src/windows.rs`
- Create: `src-tauri/src/settings.rs`
- Create: `src-tauri/tests/settings_test.rs`

**Interfaces:**
- Produces:
  - `register_context_menu(exe_path: &str) -> Result<(), String>`: writes `HKCU\Software\Classes\*\shell\FileSharer`
  - `unregister_context_menu() -> Result<(), String>`: removes registry key
  - `is_context_menu_registered() -> bool`
  - `struct AppSettings`: load, save settings to `data/settings.json` (tray, autostart, autocopy, language, contextMenu)

- [ ] **Step 1: Write failing tests for settings serialization and context menu key management**
- [ ] **Step 2: Run cargo test to verify it fails**
- [ ] **Step 3: Implement windows.rs with winreg and settings.rs**
- [ ] **Step 4: Run cargo test to verify tests pass**
- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/windows.rs src-tauri/src/settings.rs src-tauri/tests/settings_test.rs
git commit -m "feat(backend): implement Windows context menu registry helper and app settings"
```

---

### Task 7: Tauri Commands & App Integration (`lib.rs` & `main.rs`)

**Files:**
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: All backend modules (`registry`, `stream`, `server`, `tunnel`, `windows`, `settings`)
- Produces:
  - Tauri commands: `add_local_share`, `list_shares`, `update_share`, `revoke_share`, `shorten_share_url`, `get_app_settings`, `save_app_settings`, `set_context_menu_enabled`, `pick_file_dialog`, `show_in_folder`, `retry_tunnel`
  - Single-instance plugin wiring: handles argument passed from context menu (`FileSharer.exe "path\to\file"`), emits event to frontend, auto-adds share and copies link
  - Tokio runtime background server spawn

- [ ] **Step 1: Implement src-tauri/src/lib.rs wiring all IPC commands and AppState**
- [ ] **Step 2: Implement src-tauri/src/main.rs with single instance listener and window setup**
- [ ] **Step 3: Run cargo check to verify all Tauri bindings compile cleanly**
- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs
git commit -m "feat(backend): wire Tauri IPC commands and background server integration"
```

---

### Task 8: Svelte 4 Frontend - UI Layout & Components

**Files:**
- Create: `src/main.ts`
- Create: `src/App.svelte`
- Create: `src/components/Header.svelte`
- Create: `src/components/UploadCard.svelte`
- Create: `src/components/SharesList.svelte`
- Create: `src/components/ShareItemRow.svelte`
- Create: `src/components/SettingsModal.svelte`
- Create: `src/components/EditModal.svelte`
- Create: `src/components/DeleteModal.svelte`
- Create: `src/components/Toast.svelte`
- Create: `src/lib/i18n.ts`
- Create: `src/lib/api.ts`
- Create: `src/app.css`
- Create: `public/index.html`

**Interfaces:**
- Replicates the exact visual structure, colors, icons, and UX of `local-video-streamer`:
  - Top card: Status bar, Dropzone, local path input
  - Active shares card: Rows with file type icons, name, size, download/view counts, copy public/LAN link, shorten, edit, delete
  - Settings modal: Language toggle (TH/EN), System tray toggle, Auto-start toggle, Auto-copy toggle, Context menu toggle
  - Modals: Edit title, Delete confirmation, Toast alert

- [ ] **Step 1: Create i18n dictionary (Thai & English) and API client layer**
- [ ] **Step 2: Create Tailwind CSS design tokens and public/index.html**
- [ ] **Step 3: Implement Svelte components: Header, UploadCard, SharesList, ShareItemRow**
- [ ] **Step 4: Implement Svelte modals: SettingsModal, EditModal, DeleteModal, Toast**
- [ ] **Step 5: Assemble App.svelte with event listeners and state management**
- [ ] **Step 6: Run node build.js to verify frontend builds into dist/**
- [ ] **Step 7: Commit**

```bash
git add src/ public/
git commit -m "feat(frontend): implement Svelte 4 UI matching local-video-streamer structure"
```

---

### Task 9: Build, Test Verification & Stop

**Files:**
- Run: `npm run build`
- Run: `cargo test`
- Verification: Validate all acceptance criteria using `verify-and-stop`

- [ ] **Step 1: Run frontend build and ensure zero errors**
- [ ] **Step 2: Run all Rust tests and ensure 100% pass**
- [ ] **Step 3: Verify context menu registry code and HTTP download engine**
- [ ] **Step 4: Commit and finalize branch**

```bash
git add -A
git commit -m "test: verify all frontend and backend tests pass"
```
