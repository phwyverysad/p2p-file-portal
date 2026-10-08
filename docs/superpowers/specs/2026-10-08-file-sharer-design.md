# Design Specification: Local File Sharer Desktop Application

**Date:** 2026-10-08  
**Author:** Antigravity  
**Status:** Approved  

---

## 1. Overview & Purpose

**Local File Sharer** is a high-performance Windows desktop application built with **Tauri v2 (Rust backend)** and **Svelte 4 + TypeScript + Tailwind CSS (Frontend, bundled with esbuild)**.

The software enables users to share any file directly from their local computer via secure, instant download links without uploading to any third-party cloud storage or intermediate servers. It features deep Windows Shell integration (Right-click context menu "แชร์ลิ้ง" on any file), single-instance IPC, zero-config Cloudflare Quick Tunneling, local LAN link generation, download tracking/analytics, and a customizable desktop management GUI whose layout and settings strictly follow the design structure of `local-video-streamer`.

---

## 2. Tech Stack & Architecture

### Frontend
- **Framework:** Svelte 4 (`svelte@4.x`)
- **Language:** TypeScript
- **Styling:** Tailwind CSS + PostCSS
- **Bundler:** esbuild (fast native bundling targeting `dist/`)
- **Icons:** SVG icons conforming to `local-video-streamer` aesthetic
- **Localization:** Thai (default) & English toggle

### Desktop Engine & Backend
- **Framework:** Tauri v2 (`@tauri-apps/api`, `@tauri-apps/cli`, Tauri Rust crates)
- **Language:** Rust (2021 edition)
- **HTTP Server:** Embedded `axum` on `tokio` runtime running locally on `127.0.0.1:3030` (with dynamic fallback to available port if occupied)
- **Tunneling:** Cloudflare Quick Tunnel (`cloudflared` binary child process management)
- **Windows Integration:**
  - Context menu registered in `HKCU\Software\Classes\*\shell\FileSharer` via `winreg` (no admin rights required)
  - Single-instance IPC via `tauri-plugin-single-instance` to handle file arguments passed from context menu
  - Native System Tray with minimize-to-tray and background execution
  - Windows Native Notifications for instant link copy feedback
- **Data Persistence:** `data/shares.json` storing active share items, titles, descriptions, download counts, and views

---

## 3. Core Functional Requirements

### 3.1 Windows Right-Click Context Menu ("แชร์ลิ้ง")
1. Register right-click entry under `HKCU\Software\Classes\*\shell\FileSharer`:
   - Display label: `แชร์ลิ้ง (Share Link)`
   - Command: `"<path-to-exe>" "%1"`
   - Optional icon pointer to app executable
2. Context menu can be enabled or disabled dynamically from the Settings modal inside the GUI.
3. When clicked on any file:
   - If app is running: the second instance passes `%1` to the running instance, window restores/focuses, file registers immediately, link is copied (if auto-copy enabled), and a Windows toast notification is displayed.
   - If app is not running: app launches, spins up background server & tunnel, registers the file, and displays the UI.

### 3.2 File Streaming & Download Engine
1. **HTTP 206 Partial Content (Byte-Range):**
   - Supports `Range: bytes=start-end` requests
   - Enables multi-threaded download acceleration (e.g. Internet Download Manager / IDM)
   - Enables download pausing and resuming across unstable network connections
2. **Zero-Copy Streaming:** Streams directly from local disk in chunks (64KB - 1MB) without loading whole files into memory, easily handling files from a few KB up to 100GB+.
3. **Security & Path Traversal Prevention:**
   - Physical file paths are NEVER exposed to the public web.
   - Files are addressed solely by opaque tokens (`f_<hex6>`).
   - Requests are verified against the active in-memory / persisted registry. Revoked files instantly return 404.
4. **Content-Disposition:** Sets `Content-Disposition: attachment; filename="<filename>"` for direct downloads, preserving original file names and extensions.

### 3.3 Public Tunneling & Local LAN Sharing
1. **Cloudflare Quick Tunnel:**
   - Spawns `cloudflared tunnel --url http://127.0.0.1:<PORT>`
   - Extracts public HTTPS URL (e.g. `https://<random>.trycloudflare.com`)
   - Emits real-time connection status (`connecting`, `online`, `offline`, `error`) to UI
   - Automatic reconnect and manual retry button
2. **Local LAN IP Fallback:**
   - Detects local IPv4 interface (e.g. `http://192.168.1.50:<PORT>`)
   - Users can choose between public Cloudflare link or high-speed local LAN link

### 3.4 Download Landing Page & Social Preview
1. **Landing Page (`/d/:id`):**
   - Clean, modern web page displaying file name, extension icon, formatted size, and shared date.
   - Large prominent "ดาวน์โหลดไฟล์ (Download File)" button pointing to `/api/download/:id`.
   - Copy Direct Link button.
2. **Discord / Social Rich Embed:**
   - Serves Open Graph (`og:title`, `og:description`, `og:site_name`, `og:image`) and Twitter Card meta tags for rich link previews in Discord, Facebook, LINE, and Twitter.

### 3.5 Download Tracking & Analytics
1. **Downloads Counter:** Tracks number of successful or initiated downloads per file.
2. **Views Counter:** Tracks visits to the landing page.
3. Real-time updates reflected in the desktop GUI.

---

## 4. User Interface Specification (Matching `local-video-streamer`)

### Main Window Layout
- **Header / Status Bar:**
  - App Caption & Status dot (`พร้อมใช้งาน` / `ออนไลน์` / `เชื่อมต่อสำเร็จ`)
  - Server port and tunnel indicator
- **Top Card (Add / Share File):**
  - Drag & Drop zone supporting ALL file types (`ทุกนามสกุล`)
  - Upload / Add Progress bar
  - Divider: `หรือใส่พาธไฟล์ในเครื่อง` (Or enter local file path)
  - Text input + Browse button + `แชร์` (Share) button
- **Active Shares List Card:**
  - Section Title: `ไฟล์ทั้งหมด` (All Files) with badge count (`N ไฟล์`)
  - Item Rows displaying:
    - File icon according to extension (Archive, Document, Video, Audio, Executable, Generic)
    - Custom Title / Original Filename
    - File Size badge
    - Download count (`ดาวน์โหลด N ครั้ง`) & View count (`ดู N ครั้ง`)
    - Action Buttons:
      - `คัดลอกลิงก์` (Copy Public Link)
      - `คัดลอก LAN` (Copy Local LAN Link)
      - `ย่อลิงก์` (Shorten URL via spoo.me / da.gd)
      - `เปิดโฟลเดอร์` (Show in File Explorer)
      - `แก้ไข` (Edit Title & Description)
      - `ลบ` (Revoke / Delete share)
- **Bottom-Left Floating Gear Button:**
  - Opens Settings Modal

### Settings Modal
- **Language:** Segmented control `[ไทย] [English]`
- **System Tray:** Toggle `ทำงานในถาดระบบ (System Tray)`
- **Auto Start:** Toggle `เปิดโปรแกรมอัตโนมัติเมื่อเปิดเครื่อง`
- **Auto Copy Link:** Toggle `คัดลอกลิงก์อัตโนมัติหลังแชร์ไฟล์`
- **Windows Context Menu:** Toggle `เพิ่มเมนูคลิกขวาใน Windows ("แชร์ลิ้ง")`
- App info box: App name, version, server status & port

### Additional Modals
- **Edit Modal:** Form to edit display title and optional description
- **Delete Modal:** Confirmation modal warning that public link access will be revoked immediately (original file remains untouched on disk)
- **Toast Notifications:** Feedback on link copy, file addition, and settings change

---

## 5. Data Models & API Contracts

### Data Model: `ShareItem`
```rust
pub struct ShareItem {
    pub id: String,              // e.g. "f_3a7b9c"
    pub file_path: String,       // Absolute path on host machine
    pub original_name: String,   // File basename e.g. "archive.zip"
    pub title: String,           // User-customizable display title
    pub description: String,     // Optional description
    pub size: u64,               // File size in bytes
    pub mime_type: String,       // e.g. "application/zip"
    pub created_at: String,      // ISO 8601 timestamp
    pub downloads: u64,          // Number of downloads
    pub views: u64,              // Number of landing page views
    pub short_url: Option<String>,
}
```

### HTTP Endpoints
- `GET /api/status`: Returns server and tunnel status
- `GET /api/shares`: Lists all active shares
- `POST /api/shares/local`: Registers a local file path `{ "filePath": "...", "title": "..." }`
- `PUT /api/shares/:id`: Updates title or description
- `DELETE /api/shares/:id`: Revokes share
- `POST /api/shares/:id/shorten`: Generates shortened URL
- `GET /d/:id`: HTML Landing Page with rich social meta tags
- `GET /api/download/:id`: Streaming file download with HTTP 206 Byte-Range support
- `POST /api/tunnel/retry`: Reconnects Cloudflare tunnel

### Tauri IPC Commands
- `add_local_share(file_path: String, title: Option<String>) -> Result<ShareItem, String>`
- `list_shares() -> Result<Vec<ShareItem>, String>`
- `update_share(id: String, title: Option<String>, description: Option<String>) -> Result<ShareItem, String>`
- `revoke_share(id: String) -> Result<bool, String>`
- `shorten_share_url(id: String) -> Result<String, String>`
- `get_app_settings() -> Result<AppSettings, String>`
- `save_app_settings(settings: AppSettings) -> Result<AppSettings, String>`
- `set_context_menu_enabled(enabled: bool) -> Result<bool, String>`
- `pick_file_dialog() -> Result<Option<String>, String>`
- `show_in_folder(path: String) -> Result<(), String>`
- `copy_to_clipboard(text: String) -> Result<(), String>`
- `retry_tunnel() -> Result<(), String>`

---

## 6. Verification & Quality Gates
- **Rust Unit & Integration Tests:**
  - Registry storage & CRUD
  - Byte-range parser and chunking logic
  - Context menu registry keys formatting
  - Token generation & path security
- **Frontend Verification:**
  - TypeScript type check (`tsc --noEmit`)
  - esbuild bundling verification
  - Card & Modal rendering tests
- **System Integration:**
  - Launch Tauri v2 app
  - Verify HTTP server serving `/d/:id` and `/api/download/:id`
  - Verify context menu registration / unregistration
