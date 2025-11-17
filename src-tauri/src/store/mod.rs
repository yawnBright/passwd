use crate::password::Password;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

pub mod github_store;
pub mod local_store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageTarget {
    Local,
    GitHub,
    // All, // 查询时使用，表示查询所有存储点
}

impl Display for StorageTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageTarget::Local => write!(f, "Local"),
            StorageTarget::GitHub => write!(f, "GitHub"),
            // StorageTarget::All =>
        }
    }
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

impl StorageData {
    pub fn new() -> Self {
        StorageData {
            metadata: StorageMetadata {
                version: "1".to_string(),
                last_sync: Utc::now(),
                password_count: 0,
            },
            passwords: HashMap::new(),
        }
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn load(&self) -> Result<StorageData>;
    async fn save(&self, data: &StorageData) -> Result<()>;
    // #[allow(dead_code)]
    async fn test_connection(&self) -> Result<()>;
    async fn has_encrypted_data(&self) -> Result<bool>;
}
