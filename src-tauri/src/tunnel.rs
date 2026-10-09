use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub fn find_cloudflared_binary() -> Option<PathBuf> {
    // 1. Next to current executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let next_to_exe = parent.join("cloudflared.exe");
            if next_to_exe.exists() {
                return Some(next_to_exe);
            }
            let res = parent.join("resources").join("cloudflared.exe");
            if res.exists() {
                return Some(res);
            }
        }
    }

    // 2. Relative to current project directory
    let candidates = [
        "resources/cloudflared.exe",
        "bin/cloudflared.exe",
        "src-tauri/resources/cloudflared.exe",
        "../resources/cloudflared.exe",
        "../bin/cloudflared.exe",
    ];

    for c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return Some(p.canonicalize().unwrap_or(p.to_path_buf()));
        }
    }

    // 3. Check AppData local directory or extract embedded copy
    if let Some(data_dir) = dirs::data_local_dir() {
        let app_bin = data_dir.join("FileSharer").join("bin").join("cloudflared.exe");
        if app_bin.exists() {
            return Some(app_bin);
        }
        // Extract embedded cloudflared binary if not on disk
        if let Some(parent) = app_bin.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        static EMBEDDED_CLOUDFLARED: &[u8] = include_bytes!("../resources/cloudflared.exe");
        if std::fs::write(&app_bin, EMBEDDED_CLOUDFLARED).is_ok() {
            return Some(app_bin);
        }
    }

    // 4. Fallback to PATH
    if let Ok(output) = std::process::Command::new("where.exe")
        .arg("cloudflared.exe")
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = s.lines().next() {
                let p = PathBuf::from(first_line.trim());
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    None
}

pub type TunnelNotifier = Arc<dyn Fn(&str, Option<&str>) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct TunnelManager {
    pub status: Arc<RwLock<String>>, // "offline" | "starting" | "online" | "error"
    pub public_url: Arc<RwLock<Option<String>>>,
    pub error_message: Arc<RwLock<Option<String>>>,
    child_handle: Arc<Mutex<Option<Child>>>,
    pub on_status_changed: Arc<RwLock<Option<TunnelNotifier>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new("offline".to_string())),
            public_url: Arc::new(RwLock::new(None)),
            error_message: Arc::new(RwLock::new(None)),
            child_handle: Arc::new(Mutex::new(None)),
            on_status_changed: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_notifier(&self, notifier: TunnelNotifier) {
        *self.on_status_changed.write().unwrap() = Some(notifier);
    }

    pub fn set_status(&self, status: &str, url: Option<String>, err: Option<String>) {
        *self.status.write().unwrap() = status.to_string();
        if url.is_some() {
            *self.public_url.write().unwrap() = url.clone();
        }
        *self.error_message.write().unwrap() = err;
        if let Ok(guard) = self.on_status_changed.read() {
            if let Some(ref cb) = *guard {
                cb(status, url.as_deref());
            }
        }
    }

    pub async fn stop(&self) {
        let mut lock = self.child_handle.lock().await;
        if let Some(mut child) = lock.take() {
            let _ = child.kill().await;
        }
        self.set_status("offline", None, None);
    }

    pub async fn start(&self, port: u16) {
        self.stop().await;

        let binary = match find_cloudflared_binary() {
            Some(b) => b,
            None => {
                self.set_status(
                    "error",
                    None,
                    Some("cloudflared binary not found on machine".to_string()),
                );
                return;
            }
        };

        self.set_status("starting", None, None);

        let url_target = format!("http://127.0.0.1:{}", port);

        #[cfg(target_os = "windows")]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = Command::new(&binary);
        cmd.args([
            "tunnel",
            "--no-autoupdate",
            "--edge-ip-version",
            "4",
            "--protocol",
            "http2",
            "--retries",
            "3",
            "--grace-period",
            "1s",
            "--url",
            &url_target,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        match cmd.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                let status_clone = self.status.clone();
                let url_clone = self.public_url.clone();
                let err_clone = self.error_message.clone();
                let notif_clone = self.on_status_changed.clone();

                if let Some(out) = stdout {
                    let s_c = status_clone.clone();
                    let u_c = url_clone.clone();
                    let e_c = err_clone.clone();
                    let n_c = notif_clone.clone();
                    tokio::spawn(async move {
                        scan_stream_for_url(out, u_c, s_c, e_c, n_c).await;
                    });
                }

                if let Some(err) = stderr {
                    let s_c = status_clone.clone();
                    let u_c = url_clone.clone();
                    let e_c = err_clone.clone();
                    let n_c = notif_clone.clone();
                    tokio::spawn(async move {
                        scan_stream_for_url(err, u_c, s_c, e_c, n_c).await;
                    });
                }

                let mut lock = self.child_handle.lock().await;
                *lock = Some(child);
            }
            Err(e) => {
                self.set_status("error", None, Some(format!("Failed to spawn tunnel: {}", e)));
            }
        }
    }
}

async fn scan_stream_for_url<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader: R,
    url_target: Arc<RwLock<Option<String>>>,
    status_target: Arc<RwLock<String>>,
    err_target: Arc<RwLock<Option<String>>>,
    notifier_target: Arc<RwLock<Option<TunnelNotifier>>>,
) {
    let buf = BufReader::new(reader);
    let mut lines = buf.lines();
    let re = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(mat) = re.find(&line) {
            let public_url = mat.as_str().to_string();
            *url_target.write().unwrap() = Some(public_url.clone());
            *status_target.write().unwrap() = "online".to_string();
            *err_target.write().unwrap() = None;
            if let Ok(guard) = notifier_target.read() {
                if let Some(ref cb) = *guard {
                    cb("online", Some(&public_url));
                }
            }
        }
    }
}
