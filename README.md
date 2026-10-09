# P2P File Portal

> Instant Local and Tunnel File Sharing Desktop Application for Windows.  
> Share any file or folder directly from your computer as a high-speed download link without uploading to any cloud servers. Built with Tauri Framework v2, Rust backend, and Svelte 4, TypeScript, and Tailwind CSS.

---

## Downloads

You can download the latest version from [GitHub Releases](https://github.com/phwyverysad/p2p-file-portal/releases/latest):

1. **P2P-File-Portal-Portable.exe** (Standalone Executable)
   - Fully self-contained portable version with embedded Cloudflare tunnel engine.
   - No installation required. Runs directly from any location (Desktop, Downloads, USB flash drive).
   - Automatically handles process updates if an older instance was running in the background.

2. **P2P-File-Portal-WebInstaller.exe** (Zero-Click Lightweight Web Installer)
   - Ultra-small launcher (~3 MB).
   - Zero-click installation: Launches immediately if already installed; silently fetches and configures the latest release if not installed.
   - Automatically sets up Desktop and Start Menu shortcuts.
   - Preserves all your settings, preferences, and share history intact upon update.

---

## Key Features

1. **Zero-Upload Direct Streaming**
   - Direct real-time streaming straight from your Windows machine to the downloader.
   - Supports files of any size (from a few kilobytes up to 100 GB and above) and any file format.
   - Minimal memory footprint using chunked asynchronous streaming.

2. **Folder Sharing with Automatic ZIP Streaming**
   - Share entire directories with subfolders and files.
   - Streams dynamically on-the-fly as a standard ZIP archive without pre-compressing to disk first.

3. **HTTP 206 Partial Content (Pause / Resume and IDM Support)**
   - Full HTTP Byte-Range support for maximum download speed and multi-connection download managers (such as IDM, Free Download Manager, and curl).
   - Seamlessly pause and resume interrupted downloads.

4. **Dual-Link Mode (LAN and Public Cloudflare Tunnel)**
   - Public HTTPS Link: Powered by Cloudflare Quick Tunnel, operating through firewalls and NAT without router port-forwarding.
   - Local LAN URL: High-speed gigabit transfers within your local Wi-Fi or Ethernet network.

5. **Windows Context Menu Integration**
   - Right-click any file or directory in Windows File Explorer and select "Share Link" to share instantly.

6. **Rich File Previews and Download Landing Page**
   - Clean, responsive download landing page with authentic Windows shell icons, media previews, and custom descriptions.
   - Direct download mode option to trigger immediate file download without opening the landing page.

7. **Upload and Sharing History**
   - Searchable history log of all past shares.
   - Real-time disk availability detection (automatically marks deleted local files with strikethrough styling).
   - Single-click restore and bulk deletion capabilities.

8. **Settings and Customization**
   - Bilingual interface support: English and Thai.
   - Minimize to system tray for background operation.
   - Auto-copy links to clipboard upon file selection.
   - URL shortener integration (da.gd and spoo.me).

---

## Architecture and Tech Stack

- **Desktop Framework:** Tauri v2
- **Backend Core:** Rust 2021, Axum, Tokio, Winreg, Cloudflared
- **Frontend UI:** Svelte 4, TypeScript
- **Styling:** Tailwind CSS, PostCSS
- **Bundler:** esbuild

---

## Development

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

## License
MIT License. Free and open source.
