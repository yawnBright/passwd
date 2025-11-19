use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
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
    // pub data_path: PathBuf,
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
    pub is_first_setup: bool,
    pub storage: StorageConfig,
    // pub security: SecurityConfig,
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        // Use relative path that will be resolved by Tauri's path API when needed
        // let data_path = PathBuf::from("passwords.json");

        Self {
            is_first_setup: true,
            storage: StorageConfig {
                local_storage: Some(LocalStorageConfig { enabled: true }),
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
    // pub fn new() -> Self {
    //     Self::default()
    // }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file[{:?}]: {}", path.to_str(), e))?;

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
    pub fn get_config_path(app_handle: &tauri::AppHandle) -> tauri::Result<PathBuf> {
        app_handle
            .path()
            .resolve("config.json", BaseDirectory::AppConfig)
    }
    pub fn get_data_path(app_handle: &tauri::AppHandle) -> tauri::Result<PathBuf> {
        app_handle
            .path()
            .resolve("passwords.json", BaseDirectory::AppData)
    }
}
