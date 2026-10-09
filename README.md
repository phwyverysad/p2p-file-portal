# P2P File Portal ⚡

> **Instant Local & Tunnel File Sharing Desktop App for Windows**  
> Share any file or folder directly from your computer as a high-speed download link without uploading to any cloud servers. Built with **Tauri Framework (Tauri v2)**, **Rust backend**, and **Svelte 4 + TypeScript + Tailwind CSS**.

---

## 📥 Downloads (การดาวน์โหลด)

You can download the latest version from [GitHub Releases](https://github.com/phwyverysad/p2p-file-portal/releases/latest):

1. **`P2P-File-Portal-Portable.exe` (Portable Standalone)**
   - Fully standalone executable with embedded tunnel engine.
   - Run anywhere (Desktop, Downloads, USB flash drive) with **no installation required**.
   - Auto-detects and seamlessly updates if an older version was running.

2. **`P2P-File-Portal-WebInstaller.exe` (Zero-Click Lightweight Web Installer)**
   - Ultra-small launcher (~3MB).
   - **Zero-click installation**: Launches immediately if already installed; silently fetches the latest release if not installed.
   - Automatically sets up Desktop & Start Menu shortcuts.
   - Keeps all your settings, preferences, and share history intact upon update.

---

## ✨ Key Features (ฟีเจอร์หลัก)

1. **Zero-Upload Direct Stream (แชร์ตรงจากเครื่อง ไม่ต้องอัปโหลด):**
   - Direct real-time streaming straight from your Windows machine to the downloader.
   - Supports files of any size (from a few KB up to 100GB+) and any file type.
   - Zero-RAM chunked streaming pipeline.

2. **Folder Sharing with Automatic ZIP Streaming (แชร์โฟลเดอร์ บีบอัด ZIP ทันที):**
   - Share entire directories with subfolders and files.
   - Streams on-the-fly as a standard `.zip` archive without waiting to compress to disk first.

3. **HTTP 206 Partial Content (Pause / Resume & IDM Support):**
   - Full HTTP Byte-Range support for maximum download speed and multi-connection downloaders (IDM, Free Download Manager, curl).
   - Seamlessly pause and resume interrupted downloads.

4. **Dual-Link Mode (LAN + Public Cloudflare Tunnel):**
   - **Public HTTPS Link:** Powered by Cloudflare Quick Tunnel — works through firewalls and NAT without router port-forwarding.
   - **Local LAN URL:** Ultra-fast gigabit transfers within your home or office Wi-Fi / Ethernet.

5. **Windows Context Menu ("แชร์ลิ้ง"):**
   - Right-click any file or folder in File Explorer and select **"แชร์ลิ้ง" (Share Link)** to share instantly.

6. **Rich File Previews & Landing Page:**
   - Beautiful responsive download landing page with genuine Windows shell icons, video/image thumbnails, and customized descriptions.
   - Direct download mode bypass option.

7. **Upload & Sharing History (ประวัติการแชร์):**
   - Searchable history log of all past shares.
   - Real-time disk availability detection (automatically strikes through deleted files).
   - One-click restore and bulk deletion.

8. **Settings & Customization:**
   - Bilingual support: **ไทย (Thai)** & **English**.
   - Minimize to system tray.
   - Auto-copy links to clipboard upon file selection.
   - URL shortener integration (da.gd / spoo.me).

---

## 🛠️ Tech Stack (โครงสร้างเทคโนโลยี)

- **Desktop Framework:** Tauri v2
- **Backend Core:** Rust 2021 + Axum + Tokio + Winreg + Cloudflared
- **Frontend UI:** Svelte 4 + TypeScript
- **Styling:** Tailwind CSS + PostCSS
- **Bundler:** esbuild

---

## 🚀 Development (การรันในโหมดพัฒนา)

### 1. Install dependencies
```bash
npm install
```

### 2. Build Frontend
```bash
node build.js
```

### 3. Run Tauri Desktop App
```bash
npm run tauri dev
```

### 4. Build Release Binaries
```bash
npm run build
cd src-tauri
cargo build --release --bins
```

Executable output:
- `src-tauri/target/release/p2p-file-portal.exe`
- `src-tauri/target/release/p2p-file-portal-installer.exe`

---

## 📄 License
MIT License. Free and open source.
