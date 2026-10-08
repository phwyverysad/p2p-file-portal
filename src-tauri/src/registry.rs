use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use chrono::Utc;
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareItem {
    pub id: String,
    pub file_path: String,
    pub original_name: String,
    pub title: String,
    pub description: String,
    pub size: u64,
    pub mime_type: String,
    pub created_at: String,
    pub downloads: u64,
    pub views: u64,
    pub short_url: Option<String>,
    #[serde(default)]
    pub direct_download: bool,
    #[serde(default)]
    pub is_directory: bool,
    #[serde(default)]
    pub file_count: u64,
}

fn calculate_dir_stats(path: &Path) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut file_count = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    let (sub_size, sub_count) = calculate_dir_stats(&p);
                    total_size += sub_size;
                    file_count += sub_count;
                } else if meta.is_file() {
                    total_size += meta.len();
                    file_count += 1;
                }
            }
        }
    }
    (total_size, file_count)
}

#[derive(Clone)]
pub struct FileRegistry {
    storage_path: PathBuf,
    shares: Arc<RwLock<HashMap<String, ShareItem>>>,
}

impl FileRegistry {
    pub fn new(storage_path: PathBuf) -> Self {
        let registry = Self {
            storage_path,
            shares: Arc::new(RwLock::new(HashMap::new())),
        };
        registry.reload_from_disk();
        registry
    }

    pub fn reload_from_disk(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if self.storage_path.exists() {
            if let Ok(raw) = fs::read_to_string(&self.storage_path) {
                if let Ok(list) = serde_json::from_str::<Vec<ShareItem>>(&raw) {
                    let mut lock = self.shares.write().unwrap();
                    lock.clear();
                    for item in list {
                        if Path::new(&item.file_path).exists() {
                            lock.insert(item.id.clone(), item);
                        }
                    }
                }
            }
        }
    }

    pub fn save_to_disk(&self) {
        let list: Vec<ShareItem> = {
            let lock = self.shares.read().unwrap();
            lock.values().cloned().collect()
        };

        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub fn register(
        &self,
        file_path: &str,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<ShareItem, String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File does not exist: {}", file_path));
        }

        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let is_dir = metadata.is_dir();
        if !metadata.is_file() && !is_dir {
            return Err("Specified path is not a file or directory".to_string());
        }

        let original_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("item")
            .to_string();

        let resolved_title = match title {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => original_name.clone(),
        };

        let resolved_desc = description.unwrap_or_default();

        let (size, file_count, mime) = if is_dir {
            let (total_size, count) = calculate_dir_stats(path);
            (total_size, count, "application/zip".to_string())
        } else {
            let m = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            (metadata.len(), 1, m)
        };

        let mut rng = rand::thread_rng();
        let random_hex: String = (0..4)
            .map(|_| format!("{:02x}", rng.gen::<u8>()))
            .collect();
        let id = format!("f_{}", random_hex);

        let item = ShareItem {
            id: id.clone(),
            file_path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().to_string(),
            original_name,
            title: resolved_title,
            description: resolved_desc,
            size,
            mime_type: mime,
            created_at: Utc::now().to_rfc3339(),
            downloads: 0,
            views: 0,
            short_url: None,
            direct_download: false,
            is_directory: is_dir,
            file_count,
        };

        {
            let mut lock = self.shares.write().unwrap();
            lock.insert(id, item.clone());
        }

        self.save_to_disk();
        Ok(item)
    }

    pub fn get(&self, id: &str) -> Option<ShareItem> {
        let lock = self.shares.read().unwrap();
        lock.get(id).cloned()
    }

    pub fn list(&self) -> Vec<ShareItem> {
        let lock = self.shares.read().unwrap();
        let mut list: Vec<ShareItem> = lock.values().cloned().collect();
        // Sort newest first
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn update_meta(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        direct_download: Option<bool>,
    ) -> Result<ShareItem, String> {
        let mut lock = self.shares.write().unwrap();
        if let Some(item) = lock.get_mut(id) {
            if let Some(t) = title {
                if !t.trim().is_empty() {
                    item.title = t.trim().to_string();
                }
            }
            if let Some(d) = description {
                item.description = d.trim().to_string();
            }
            if let Some(dd) = direct_download {
                item.direct_download = dd;
            }
            let updated = item.clone();
            drop(lock);
            self.save_to_disk();
            Ok(updated)
        } else {
            Err(format!("Share not found: {}", id))
        }
    }

    pub fn revoke(&self, id: &str) -> bool {
        let mut lock = self.shares.write().unwrap();
        let removed = lock.remove(id).is_some();
        drop(lock);
        if removed {
            self.save_to_disk();
        }
        removed
    }

    pub fn increment_views(&self, id: &str) {
        let mut lock = self.shares.write().unwrap();
        if let Some(item) = lock.get_mut(id) {
            item.views += 1;
            drop(lock);
            self.save_to_disk();
        }
    }

    pub fn increment_downloads(&self, id: &str) {
        let mut lock = self.shares.write().unwrap();
        if let Some(item) = lock.get_mut(id) {
            item.downloads += 1;
            drop(lock);
            self.save_to_disk();
        }
    }

    pub fn set_short_url(&self, id: &str, short_url: String) {
        let mut lock = self.shares.write().unwrap();
        if let Some(item) = lock.get_mut(id) {
            item.short_url = Some(short_url);
            drop(lock);
            self.save_to_disk();
        }
    }
}
