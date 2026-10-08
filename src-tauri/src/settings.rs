use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub minimize_to_tray: bool,
    pub auto_start: bool,
    pub auto_copy: bool,
    pub context_menu: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "th".to_string(),
            minimize_to_tray: true,
            auto_start: false,
            auto_copy: true,
            context_menu: true,
        }
    }
}

impl AppSettings {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(raw) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&raw) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
