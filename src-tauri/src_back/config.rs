use anyhow::{anyhow, Result};
// use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local_storage: LocalStorageConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_salt: Vec<u8>,
    pub double_encrypt_descriptions: bool, // 是否双重加密描述信息
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("password_manager");

        Self {
            storage: StorageConfig {
                local_storage: LocalStorageConfig {
                    enabled: true,
                    data_path: data_dir.join("passwords.json"),
                },
                github_storage: None,
            },
            security: SecurityConfig {
                encryption_salt: vec![0u8; 32],
                double_encrypt_descriptions: false,
            },
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

    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("password_manager")
            .join("config.json")
    }

    pub fn validate(&self) -> Result<()> {
        if self.storage.local_storage.enabled
            && self.storage.local_storage.data_path.as_os_str().is_empty()
        {
            return Err(anyhow!("Local storage path cannot be empty"));
        }

        if let Some(github) = &self.storage.github_storage {
            if github.enabled {
                if github.owner.is_empty() || github.repo.is_empty() || github.token.is_empty() {
                    return Err(anyhow!("GitHub storage configuration is incomplete"));
                }
            }
        }

        Ok(())
    }

    // 不再需要设置主密码哈希 - 直接使用主密钥验证
    pub fn set_encryption_salt(&mut self, salt: Vec<u8>) {
        self.security.encryption_salt = salt;
    }
}

