// use crate::password::Password;
use anyhow::Result;
use async_trait::async_trait;
// use serde::{Deserialize, Serialize};
use super::{Storage, StorageData, StorageMetadata};
use std::collections::HashMap;

pub struct LocalStorage {
    data_path: std::path::PathBuf,
}

impl LocalStorage {
    pub fn new(data_path: std::path::PathBuf) -> Self {
        Self { data_path }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn load(&self) -> Result<StorageData> {
        if !self.data_path.exists() {
            return Ok(StorageData {
                metadata: StorageMetadata {
                    version: "1.0.0".to_string(),
                    last_sync: chrono::Utc::now(),
                    password_count: 0,
                },
                passwords: HashMap::new(),
            });
        }

        let content = tokio::fs::read_to_string(&self.data_path).await?;
        let data: StorageData = serde_json::from_str(&content)?;
        Ok(data)
    }

    async fn save(&self, data: &StorageData) -> Result<()> {
        if let Some(parent) = self.data_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(data)?;
        tokio::fs::write(&self.data_path, content).await?;
        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        if let Some(parent) = self.data_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    async fn has_encrypted_data(&self) -> Result<bool> {
        if !self.data_path.exists() {
            return Ok(false);
        }

        let content = tokio::fs::read_to_string(&self.data_path).await?;
        let data: StorageData = serde_json::from_str(&content)?;

        // 如果有密码数据，说明存在加密数据
        Ok(!data.passwords.is_empty())
    }
}
