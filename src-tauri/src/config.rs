use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Use Tauri's path API for cross-platform compatibility
use tauri::Manager;
use tauri::path::BaseDirectory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local_storage: Option<LocalStorageConfig>,
    pub github_storage: Option<GithubStorageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub enabled: bool,
    pub data_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubStorageConfig {
    pub enabled: bool,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub token: String,
    pub file_path: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SecurityConfig {
//     pub encryption_salt: Vec<u8>,
//     pub double_encrypt_descriptions: bool, // 是否双重加密描述信息
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    // pub security: SecurityConfig,
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        // Use relative path that will be resolved by Tauri's path API when needed
        let data_path = PathBuf::from("passwords.json");

        Self {
            storage: StorageConfig {
                local_storage: Some(LocalStorageConfig {
                    enabled: true,
                    data_path,
                }),
                github_storage: None,
            },
            // security: SecurityConfig {
            //     encryption_salt: vec![0u8; 32],
            //     double_encrypt_descriptions: false,
            // },
            version: "1.0.0".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Config =
            serde_json::from_str(&content).map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
        }

        fs::write(path, content).map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    // Cross-platform config path using Tauri's AppConfig directory
    pub fn get_config_path() -> PathBuf {
        // For mobile platforms (Android/iOS), use AppConfig directory
        // For desktop platforms, this will use appropriate system config directories
        PathBuf::from("password_manager").join("config.json")
    }

    // Get the full config path using Tauri's path resolution
    pub async fn get_full_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
        let config_dir = app_handle
            .path()
            .resolve("password_manager", BaseDirectory::AppConfig)
            .map_err(|e| anyhow!("Failed to resolve config directory: {}", e))?;

        Ok(config_dir.join("config.json"))
    }

    // Resolve data path using Tauri's AppData directory
    pub async fn resolve_data_path(&mut self, app_handle: &tauri::AppHandle) -> Result<()> {
        if let Some(local_storage) = &mut self.storage.local_storage {
            let data_dir = app_handle
                .path()
                .resolve("password_manager", BaseDirectory::AppData)
                .map_err(|e| anyhow!("Failed to resolve data directory: {}", e))?;

            local_storage.data_path = data_dir.join("passwords.json");
        }
        Ok(())
    }
}
