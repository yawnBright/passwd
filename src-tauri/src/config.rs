use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::github_store::GitHubStoreConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    Local { file_path: String },
    GitHub { config: GitHubStoreConfig },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_password_length")]
    pub default_password_length: usize,
    pub auto_backup: bool,
    #[serde(default = "default_sync_enabled")]
    pub sync_all_stores: bool,
    pub master_key_hash: Option<String>,
}

const fn default_password_length() -> usize {
    12
}

const fn default_sync_enabled() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig::Local {
                file_path: "passwords.json".to_string(),
            },
            settings: Settings {
                default_password_length: default_password_length(),
                auto_backup: false,
                sync_all_stores: default_sync_enabled(),
                master_key_hash: None,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).context("读取配置文件失败")?;

            serde_json::from_str(&content).context("解析配置文件失败")
        } else {
            Err(anyhow::anyhow!("配置文件不存在"))
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).context("创建配置目录失败")?;
            }
        }

        let content = serde_json::to_string_pretty(self).context("序列化配置文件失败")?;

        std::fs::write(&config_path, content).context("写入配置文件失败")?;

        Ok(())
    }

    pub fn set_github_storage(&mut self, config: GitHubStoreConfig) {
        self.storage = StorageConfig::GitHub { config };
    }

    pub fn set_local_storage(&mut self, file_path: String) {
        self.storage = StorageConfig::Local { file_path };
    }

    pub fn get_github_config(&self) -> Option<crate::github_store::GitHubStoreConfig> {
        match &self.storage {
            StorageConfig::GitHub { config } => Some(config.clone()),
            _ => None,
        }
    }

    pub fn set_master_key_hash(&mut self, hash: String) {
        self.settings.master_key_hash = Some(hash);
    }

    pub fn verify_master_key(&self, master_key: &str) -> Result<bool> {
        match &self.settings.master_key_hash {
            Some(hash) => crate::crypto::verify_master_key(master_key, hash),
            None => Ok(true), // 如果没有设置主密钥哈希，则允许任何密钥
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("无法获取用户主目录")?;

    Ok(home_dir.join(".passwd").join("config.json"))
}

fn default_data_file() -> String {
    let home_dir = dirs::home_dir()
        .expect("无法获取用户主目录")
        .join(".passwd")
        .join("passwords.dat")
        .to_string_lossy()
        .to_string();

    home_dir
}
