use crate::password::Password;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod github_store;
pub mod local_store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageTarget {
    Local,
    GitHub,
    All, // 查询时使用，表示查询所有存储点
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub version: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub password_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageData {
    pub metadata: StorageMetadata,
    /// key是idgen生成的唯一id
    pub passwords: HashMap<String, Password>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn load(&self) -> Result<StorageData>;
    async fn save(&self, data: &StorageData) -> Result<()>;
    #[allow(dead_code)]
    async fn test_connection(&self) -> Result<()>;
    async fn has_encrypted_data(&self) -> Result<bool>;
}
