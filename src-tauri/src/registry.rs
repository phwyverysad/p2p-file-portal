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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub file_path: String,
    pub original_name: String,
    pub title: String,
    pub description: String,
    pub size: u64,
    pub mime_type: String,
    pub created_at: String,
    pub last_shared_at: String,
    #[serde(default)]
    pub direct_download: bool,
    #[serde(default)]
    pub is_directory: bool,
    #[serde(default)]
    pub file_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItemDto {
    #[serde(flatten)]
    pub item: HistoryItem,
    pub file_exists: bool,
    pub is_active: bool,
    pub active_share_id: Option<String>,
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
    history_path: PathBuf,
    shares: Arc<RwLock<HashMap<String, ShareItem>>>,
    history: Arc<RwLock<HashMap<String, HistoryItem>>>,
}

impl FileRegistry {
    pub fn new(storage_path: PathBuf) -> Self {
        let history_path = storage_path.with_file_name("history.json");
        let registry = Self {
            storage_path,
            history_path,
            shares: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
        };
        registry.reload_from_disk();
        registry
    }

    pub fn reload_from_disk(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // 1. Reload active shares
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

        // 2. Reload history
        let mut loaded_history = false;
        if self.history_path.exists() {
            if let Ok(raw) = fs::read_to_string(&self.history_path) {
                if let Ok(list) = serde_json::from_str::<Vec<HistoryItem>>(&raw) {
                    let mut lock = self.history.write().unwrap();
                    lock.clear();
                    for item in list {
                        lock.insert(item.id.clone(), item);
                    }
                    loaded_history = true;
                }
            }
        }

        // 3. Backfill history from active shares if history was empty
        if !loaded_history {
            let shares_lock = self.shares.read().unwrap();
            let mut hist_lock = self.history.write().unwrap();
            for s in shares_lock.values() {
                hist_lock.insert(
                    s.id.clone(),
                    HistoryItem {
                        id: s.id.clone(),
                        file_path: s.file_path.clone(),
                        original_name: s.original_name.clone(),
                        title: s.title.clone(),
                        description: s.description.clone(),
                        size: s.size,
                        mime_type: s.mime_type.clone(),
                        created_at: s.created_at.clone(),
                        last_shared_at: s.created_at.clone(),
                        direct_download: s.direct_download,
                        is_directory: s.is_directory,
                        file_count: s.file_count,
                    },
                );
            }
            drop(shares_lock);
            let needs_save = !hist_lock.is_empty();
            drop(hist_lock);
            if needs_save {
                self.save_history_to_disk();
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

    pub fn save_history_to_disk(&self) {
        let list: Vec<HistoryItem> = {
            let lock = self.history.read().unwrap();
            lock.values().cloned().collect()
        };

        if let Some(parent) = self.history_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = fs::write(&self.history_path, json);
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
            lock.insert(id.clone(), item.clone());
        }
        self.save_to_disk();

        // Also record to persistent history
        {
            let mut hist_lock = self.history.write().unwrap();
            hist_lock.insert(
                id,
                HistoryItem {
                    id: item.id.clone(),
                    file_path: item.file_path.clone(),
                    original_name: item.original_name.clone(),
                    title: item.title.clone(),
                    description: item.description.clone(),
                    size: item.size,
                    mime_type: item.mime_type.clone(),
                    created_at: item.created_at.clone(),
                    last_shared_at: Utc::now().to_rfc3339(),
                    direct_download: item.direct_download,
                    is_directory: item.is_directory,
                    file_count: item.file_count,
                },
            );
        }
        self.save_history_to_disk();

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

            // Update in persistent history too
            {
                let mut hist_lock = self.history.write().unwrap();
                if let Some(h) = hist_lock.get_mut(id) {
                    h.title = updated.title.clone();
                    h.description = updated.description.clone();
                    h.direct_download = updated.direct_download;
                    h.last_shared_at = Utc::now().to_rfc3339();
                }
            }
            self.save_history_to_disk();

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
        // Retain item in history so user never loses their share history!
        removed
    }

    pub fn restore(&self, item: ShareItem) -> ShareItem {
        let mut lock = self.shares.write().unwrap();
        lock.insert(item.id.clone(), item.clone());
        drop(lock);
        self.save_to_disk();

        {
            let mut hist_lock = self.history.write().unwrap();
            if let Some(h) = hist_lock.get_mut(&item.id) {
                h.last_shared_at = Utc::now().to_rfc3339();
            } else {
                hist_lock.insert(
                    item.id.clone(),
                    HistoryItem {
                        id: item.id.clone(),
                        file_path: item.file_path.clone(),
                        original_name: item.original_name.clone(),
                        title: item.title.clone(),
                        description: item.description.clone(),
                        size: item.size,
                        mime_type: item.mime_type.clone(),
                        created_at: item.created_at.clone(),
                        last_shared_at: Utc::now().to_rfc3339(),
                        direct_download: item.direct_download,
                        is_directory: item.is_directory,
                        file_count: item.file_count,
                    },
                );
            }
        }
        self.save_history_to_disk();

        item
    }

    pub fn get_history(&self, id: &str) -> Option<HistoryItem> {
        let lock = self.history.read().unwrap();
        lock.get(id).cloned()
    }

    pub fn list_history(&self) -> Vec<HistoryItemDto> {
        let history_lock = self.history.read().unwrap();
        let shares_lock = self.shares.read().unwrap();

        let mut list: Vec<HistoryItemDto> = history_lock
            .values()
            .map(|item| {
                let file_exists = Path::new(&item.file_path).exists();
                let is_active = shares_lock.contains_key(&item.id);
                let active_share_id = if is_active {
                    Some(item.id.clone())
                } else {
                    shares_lock
                        .values()
                        .find(|s| s.file_path == item.file_path)
                        .map(|s| s.id.clone())
                };

                HistoryItemDto {
                    item: item.clone(),
                    file_exists,
                    is_active: active_share_id.is_some(),
                    active_share_id,
                }
            })
            .collect();

        // Sort newest first by last_shared_at
        list.sort_by(|a, b| b.item.last_shared_at.cmp(&a.item.last_shared_at));
        list
    }

    pub fn delete_history_items(&self, ids: &[String]) -> usize {
        let mut lock = self.history.write().unwrap();
        let mut count = 0;
        for id in ids {
            if lock.remove(id).is_some() {
                count += 1;
            }
        }
        drop(lock);
        if count > 0 {
            self.save_history_to_disk();
        }
        count
    }

    pub fn clear_history(&self) {
        let mut lock = self.history.write().unwrap();
        lock.clear();
        drop(lock);
        self.save_history_to_disk();
    }

    pub fn restore_from_history(&self, id: &str) -> Result<ShareItem, String> {
        let hist_item = {
            let history_lock = self.history.read().unwrap();
            history_lock
                .get(id)
                .cloned()
                .ok_or_else(|| "ไม่พบรายการในประวัติ".to_string())?
        };

        let path = Path::new(&hist_item.file_path);
        if !path.exists() {
            return Err("ไฟล์นี้ถูกลบออกจากเครื่องแล้ว ไม่สามารถกู้คืนได้".to_string());
        }

        // If already active in shares, return existing
        {
            let shares_lock = self.shares.read().unwrap();
            if let Some(existing) = shares_lock.get(&hist_item.id) {
                return Ok(existing.clone());
            }
        }

        let share_item = ShareItem {
            id: hist_item.id.clone(),
            file_path: hist_item.file_path.clone(),
            original_name: hist_item.original_name.clone(),
            title: hist_item.title.clone(),
            description: hist_item.description.clone(),
            size: hist_item.size,
            mime_type: hist_item.mime_type.clone(),
            created_at: hist_item.created_at.clone(),
            downloads: 0,
            views: 0,
            short_url: None,
            direct_download: hist_item.direct_download,
            is_directory: hist_item.is_directory,
            file_count: hist_item.file_count,
        };

        {
            let mut shares_lock = self.shares.write().unwrap();
            shares_lock.insert(share_item.id.clone(), share_item.clone());
        }
        self.save_to_disk();

        {
            let mut history_lock = self.history.write().unwrap();
            if let Some(h) = history_lock.get_mut(&hist_item.id) {
                h.last_shared_at = Utc::now().to_rfc3339();
            }
        }
        self.save_history_to_disk();

        Ok(share_item)
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
