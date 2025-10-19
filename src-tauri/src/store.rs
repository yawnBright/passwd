use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::password::EncryptedPassword;

#[async_trait]
pub trait StorePoint: Send + Sync {
    async fn save(&self, password: &EncryptedPassword, master_key: &str) -> Result<()>;
    async fn load(&self, key: &str, master_key: &str) -> Result<Option<EncryptedPassword>>;
    async fn delete(&self, key: &str, master_key: &str) -> Result<()>;
    async fn list_keys(&self, master_key: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str, master_key: &str) -> Result<bool>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub password_count: usize,
}

impl Default for StoreMetadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            password_count: 0,
        }
    }
}

pub struct LocalFileStore {
    file_path: String,
}

impl LocalFileStore {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }
}

#[async_trait]
impl StorePoint for LocalFileStore {
    async fn save(&self, password: &EncryptedPassword, master_key: &str) -> Result<()> {
        let mut passwords = self.load_all(master_key).await?;
        passwords.insert(password.key.clone(), password.clone());
        self.save_all(&passwords, master_key).await
    }

    async fn load(&self, key: &str, master_key: &str) -> Result<Option<EncryptedPassword>> {
        let passwords = self.load_all(master_key).await?;
        Ok(passwords.get(key).cloned())
    }

    async fn delete(&self, key: &str, master_key: &str) -> Result<()> {
        let mut passwords = self.load_all(master_key).await?;
        passwords.remove(key);
        self.save_all(&passwords, master_key).await
    }

    async fn list_keys(&self, master_key: &str) -> Result<Vec<String>> {
        let passwords = self.load_all(master_key).await?;
        Ok(passwords.keys().cloned().collect())
    }

    async fn exists(&self, key: &str, master_key: &str) -> Result<bool> {
        let passwords = self.load_all(master_key).await?;
        Ok(passwords.contains_key(key))
    }
}

impl LocalFileStore {
    async fn load_all(&self, master_key: &str) -> Result<HashMap<String, EncryptedPassword>> {
        if !std::path::Path::new(&self.file_path).exists() {
            return Ok(HashMap::new());
        }

        let encrypted_content = tokio::fs::read(&self.file_path)
            .await
            .context("读取文件失败")?;

        let decrypted_content =
            crate::crypto::decrypt(&encrypted_content, master_key).context("解密文件失败")?;

        serde_json::from_str(&decrypted_content).context("解析密码数据失败")
    }

    async fn save_all(
        &self,
        passwords: &HashMap<String, EncryptedPassword>,
        master_key: &str,
    ) -> Result<()> {
        let serialized = serde_json::to_string(passwords).context("序列化密码数据失败")?;

        let encrypted =
            crate::crypto::encrypt(&serialized, master_key).context("加密密码数据失败")?;

        tokio::fs::write(&self.file_path, encrypted)
            .await
            .context("写入文件失败")?;

        Ok(())
    }
}
