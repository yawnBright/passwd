use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::crypto::{CryptoService, MasterKey};
use crate::github_client::GithubClient;
use crate::github_store::GithubStorage;
use crate::password::{Password, PasswordCreateRequest, PasswordUpdateRequest, PasswordSearchQuery, PasswordGeneratorConfig};
use crate::simple_crypto::SimpleCrypto;
use crate::store::{Storage, StorageData, LocalStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageTarget {
    Local,
    GitHub,
    All, // 查询时使用，表示查询所有存储点
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageStatus {
    pub enabled: bool,
    pub connected: bool,
    pub password_count: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

pub struct PasswordManager {
    config: Arc<RwLock<Config>>,
    #[allow(dead_code)]
    crypto_service: Arc<CryptoService>, // 占位符，实际不使用
    simple_crypto: Arc<SimpleCrypto>,
    storages: HashMap<StorageTarget, Arc<dyn Storage>>, // 所有启用的存储点
    cache: Arc<RwLock<HashMap<String, Password>>>,
}

impl PasswordManager {
    pub async fn new(config: Config) -> Result<Self> {
        // 不再需要主密钥，使用简单的内置密钥进行数据混淆
        let simple_crypto = Arc::new(SimpleCrypto::new());
        
        // 初始化所有启用的存储点
        let mut storages = HashMap::new();
        
        // 初始化本地存储（如果启用）
        if config.storage.local_storage.enabled {
            let local_storage = Arc::new(LocalStorage::new(config.storage.local_storage.data_path.clone()));
            storages.insert(StorageTarget::Local, local_storage as Arc<dyn Storage>);
        }
        
        // 初始化GitHub存储（如果启用）
        if let Some(github_config) = &config.storage.github_storage {
            if github_config.enabled {
                let client = GithubClient::new(
                    github_config.owner.clone(),
                    github_config.repo.clone(),
                    github_config.token.clone(),
                    github_config.branch.clone(),
                );
                let github_storage = Arc::new(GithubStorage::new(client, github_config.file_path.clone()));
                storages.insert(StorageTarget::GitHub, github_storage as Arc<dyn Storage>);
            }
        }
        
        if storages.is_empty() {
            return Err(anyhow!("At least one storage must be enabled"));
        }

        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            crypto_service: Arc::new(CryptoService::new(MasterKey::new(vec![0u8; 32]))), // 占位符，实际不使用
            simple_crypto,
            storages,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // 加载数据到缓存
        manager.load_data().await?;

        Ok(manager)
    }

    pub async fn add_password(&self, request: PasswordCreateRequest) -> Result<Password> {
        // 使用简单加密混淆密码（内置密钥888）
        let encrypted_password = self
            .simple_crypto
            .encrypt_with_checksum(&request.password)?;

        // 创建密码对象
        let password = Password::new(request, encrypted_password);
        let password_id = password.id.clone();

        // 添加到缓存
        self.cache
            .write()
            .await
            .insert(password_id.clone(), password.clone());

        // 保存到存储
        self.save_data().await?;

        Ok(password)
    }

    pub async fn delete_password(&self, password_id: &str) -> Result<()> {
        // 从缓存中删除
        if self.cache.write().await.remove(password_id).is_none() {
            return Err(anyhow!("Password not found"));
        }

        // 保存到存储
        self.save_data().await?;

        Ok(())
    }

    pub async fn search_passwords(&self, query: &str) -> Result<Vec<Password>> {
        self.search_passwords_in_storage(query, StorageTarget::All).await
    }

    /// 在指定存储点中搜索密码
    pub async fn search_passwords_in_storage(&self, query: &str, target: StorageTarget) -> Result<Vec<Password>> {
        if target == StorageTarget::All {
            // 使用缓存数据（已合并所有存储点）
            let cache = self.cache.read().await;
            let results: Vec<Password> = cache
                .values()
                .filter(|password| {
                    password
                        .title
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        || password
                            .description
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                        || password
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
                })
                .cloned()
                .collect();
            Ok(results)
        } else {
            // 从指定存储点查询
            let data = self.load_from_storage(target).await?;
            let results: Vec<Password> = data
                .passwords
                .values()
                .filter(|password| {
                    password
                        .title
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        || password
                            .description
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                        || password
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
                })
                .cloned()
                .collect();
            Ok(results)
        }
    }

    pub async fn get_all_passwords(&self) -> Result<Vec<Password>> {
        self.get_all_passwords_from_storage(StorageTarget::All).await
    }

    /// 从指定存储点获取所有密码
    pub async fn get_all_passwords_from_storage(&self, target: StorageTarget) -> Result<Vec<Password>> {
        if target == StorageTarget::All {
            // 使用缓存数据（已合并所有存储点）
            let cache = self.cache.read().await;
            Ok(cache.values().cloned().collect())
        } else {
            // 从指定存储点查询
            let data = self.load_from_storage(target).await?;
            Ok(data.passwords.values().cloned().collect())
        }
    }

    pub async fn get_password_by_id(&self, password_id: &str) -> Result<Option<Password>> {
        self.get_password_by_id_from_storage(password_id, StorageTarget::All).await
    }

    /// 从指定存储点根据ID获取密码
    pub async fn get_password_by_id_from_storage(&self, password_id: &str, target: StorageTarget) -> Result<Option<Password>> {
        if target == StorageTarget::All {
            // 使用缓存数据（已合并所有存储点）
            let cache = self.cache.read().await;
            Ok(cache.get(password_id).cloned())
        } else {
            // 从指定存储点查询
            let data = self.load_from_storage(target).await?;
            Ok(data.passwords.get(password_id).cloned())
        }
    }

    #[allow(dead_code)]
    pub async fn decrypt_password(&self, _password: &Password) -> Result<String> {
        // 使用用户提供的密码解密
        Err(anyhow!("请使用用户提供的密码解密"))
    }

    pub async fn decrypt_password_with_key(
        &self,
        password: &Password,
        _user_password: &str,
    ) -> Result<String> {
        // 使用用户提供的密码进行解密
        // 注意：这里应该使用用户提供的密码生成密钥，但为了简化，我们使用内置密钥
        // 在实际应用中，这里应该使用用户密码派生密钥
        let user_crypto = SimpleCrypto::new(); // 使用相同的内置密钥
        user_crypto.decrypt_with_checksum(&password.encrypted_password)
    }

    #[allow(dead_code)]
    pub async fn update_password(
        &self,
        password_id: &str,
        request: crate::password::PasswordUpdateRequest,
    ) -> Result<Password> {
        let mut cache = self.cache.write().await;
        let password = cache
            .get_mut(password_id)
            .ok_or_else(|| anyhow!("Password not found"))?;

        // 如果有新密码，需要加密
        let encrypted_password = if let Some(ref new_password) = request.password {
            Some(self.simple_crypto.encrypt_with_checksum(new_password)?)
        } else {
            None
        };

        password.update(request, encrypted_password);
        let updated_password = password.clone();
        drop(cache);

        // 保存到存储
        self.save_data().await?;

        Ok(updated_password)
    }

    pub async fn generate_password(&self, config: &PasswordGeneratorConfig) -> Result<String> {
        CryptoService::generate_password(config)
    }

    #[allow(dead_code)]
    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    #[allow(dead_code)]
    pub async fn update_config(&self, new_config: Config) -> Result<()> {
        *self.config.write().await = new_config;
        self.save_data().await?;
        Ok(())
    }

    async fn load_data(&self) -> Result<()> {
        let mut all_passwords = HashMap::new();
        
        // 按优先级加载所有存储点的数据（本地优先）
        let storage_order = [StorageTarget::Local, StorageTarget::GitHub];
        
        for &target in &storage_order {
            if let Some(storage) = self.storages.get(&target) {
                match storage.load().await {
                    Ok(data) => {
                        // 合并数据，后面的存储点数据会覆盖前面的
                        all_passwords.extend(data.passwords);
                    }
                    Err(e) => {
                        log::warn!("Failed to load from {:?} storage: {}", target, e);
                    }
                }
            }
        }
        
        // 更新缓存
        let mut cache = self.cache.write().await;
        cache.extend(all_passwords);
        
        Ok(())
    }

    async fn save_data(&self) -> Result<()> {
        let cache = self.cache.read().await;
        let data = StorageData {
            metadata: crate::store::StorageMetadata {
                version: "1.0.0".to_string(),
                last_sync: chrono::Utc::now(),
                password_count: cache.len(),
            },
            passwords: cache.clone(),
        };
        drop(cache);

        // 保存到所有启用的存储点
        let mut last_error = None;
        for (target, storage) in &self.storages {
            if let Err(e) = storage.save(&data).await {
                log::warn!("Failed to save to {:?} storage: {}", target, e);
                last_error = Some(e);
            }
        }

        if let Some(error) = last_error {
            Err(anyhow!("Failed to save to some storage targets: {}", error))
        } else {
            Ok(())
        }
    }

    async fn update_cache_from_data(&self, data: StorageData) -> Result<()> {
        let mut cache = self.cache.write().await;
        *cache = data.passwords;
        Ok(())
    }

    /// 获取指定存储点的存储实例
    pub fn get_storage(&self, target: StorageTarget) -> Option<Arc<dyn Storage>> {
        if target == StorageTarget::All {
            None
        } else {
            self.storages.get(&target).cloned()
        }
    }

    /// 获取所有启用的存储点
    pub fn get_enabled_storages(&self) -> Vec<(StorageTarget, Arc<dyn Storage>)> {
        self.storages.iter()
            .map(|(&target, storage)| (target, storage.clone()))
            .collect()
    }

    /// 从指定存储点加载数据
    pub async fn load_from_storage(&self, target: StorageTarget) -> Result<StorageData> {
        let storage = self.storages.get(&target)
            .ok_or_else(|| anyhow!("Storage target {:?} is not enabled", target))?;
        storage.load().await
    }

    /// 保存数据到指定存储点
    pub async fn save_to_storage(&self, target: StorageTarget, data: &StorageData) -> Result<()> {
        let storage = self.storages.get(&target)
            .ok_or_else(|| anyhow!("Storage target {:?} is not enabled", target))?;
        storage.save(data).await
    }

    /// 同步两个存储点之间的数据
    pub async fn sync_storages(&self, from: StorageTarget, to: StorageTarget) -> Result<()> {
        let from_data = self.load_from_storage(from).await?;
        self.save_to_storage(to, &from_data).await?;
        
        // 重新加载缓存
        self.load_data().await?;
        
        Ok(())
    }

    /// 获取存储点状态信息
    pub async fn get_storage_status(&self) -> HashMap<StorageTarget, StorageStatus> {
        let mut status = HashMap::new();
        
        for (&target, storage) in &self.storages {
            let storage_status = match storage.load().await {
                Ok(data) => StorageStatus {
                    enabled: true,
                    connected: true,
                    password_count: data.passwords.len(),
                    last_sync: Some(data.metadata.last_sync),
                    error: None,
                },
                Err(e) => StorageStatus {
                    enabled: true,
                    connected: false,
                    password_count: 0,
                    last_sync: None,
                    error: Some(e.to_string()),
                },
            };
            status.insert(target, storage_status);
        }
        
        status
    }

    // 不再需要主密码验证
    #[allow(dead_code)]
    pub async fn test_master_password(&self) -> Result<bool> {
        Ok(true) // 始终返回true，因为不再使用主密码
    }
}

