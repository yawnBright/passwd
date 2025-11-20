use anyhow::{Result, anyhow};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

use crate::crypto::EncryptedData;
use crate::password::{Password, PasswordCreateRequest, PasswordGeneratorConfig};
use crate::store::github_store::GithubStorage;
use crate::store::local_store::LocalStorage;
use crate::store::{Storage, StorageData, StorageTarget};
use crate::{CONF_PATH, DATA_PATH, crypto, info, password};

// #[derive(Debug, Clone, serde::Serialize)]
// pub struct StorageStatus {
//     pub enabled: bool,
//     pub connected: bool,
//     pub password_count: usize,
//     pub last_sync: Option<DateTime<Utc>>,
//     pub error: Option<String>,
// }

type Storages = HashMap<StorageTarget, Arc<dyn Storage>>;

// 每个存储点是独立的、互不干扰的(防止数据覆盖丢失)
// 后续考虑设计存储点间的数据同步机制
pub struct PasswordManager {
    config: RwLock<Config>,
    storages: RwLock<Storages>,                         // 所有启用的存储点
    cache: RwLock<HashMap<StorageTarget, StorageData>>, // 缓存策略是写透
}

impl PasswordManager {
    pub async fn new(config: Config) -> Result<Self> {
        let storages = Self::build_storages_from_config(&config)?;

        let manager = Self {
            config: RwLock::new(config),
            storages: RwLock::new(storages),
            cache: RwLock::new(HashMap::new()),
        };

        // 加载数据到缓存
        manager.load_data_to_cache().await?;

        Ok(manager)
    }

    fn build_storages_from_config(config: &Config) -> Result<Storages> {
        // 初始化所有启用的存储点
        let mut storages = HashMap::new();

        // 初始化本地存储（如果启用）
        if let Some(local_config) = &config.storage.local_storage
            && local_config.enabled
        {
            let data_path = DATA_PATH
                .get()
                .ok_or_else(|| anyhow!("DATA_PATH not set"))?;

            let local_storage = Arc::new(LocalStorage::new(data_path.clone()));
            storages.insert(StorageTarget::Local, local_storage as Arc<dyn Storage>);
        }

        // 初始化GitHub存储（如果启用）
        if let Some(github_config) = &config.storage.github_storage
            && github_config.enabled
        {
            let github_storage = Arc::new(GithubStorage::new(
                github_config.owner.clone(),
                github_config.repo.clone(),
                github_config.token.clone(),
                github_config.branch.clone(),
                github_config.file_path.clone(),
            ));
            storages.insert(StorageTarget::GitHub, github_storage as Arc<dyn Storage>);
        }

        Ok(storages)
    }

    // 更新配置
    pub async fn update_config(&self, new_config: Config) -> Result<()> {
        let mut config_inner = self.config.write().await;
        let mut storage_inner = self.storages.write().await;

        *config_inner = new_config;
        *storage_inner = Self::build_storages_from_config(&config_inner)?;

        // 保存新配置到文件
        config_inner.save_to_file(
            CONF_PATH
                .get()
                .ok_or_else(|| anyhow!("CONFIG_PATH not set"))?,
        )?;

        Ok(())
    }

    pub async fn add_password(&self, request: PasswordCreateRequest) -> Result<()> {
        let encrypted_password = crypto::encrypt_with_password(&request.password, &request.key)?;

        info!("加密后的密码: {:?}", encrypted_password);

        // 创建密码对象
        let password = Password::new(request, encrypted_password);
        let password_id = password.id.clone();

        // 添加到缓存
        let mut cache_inner = self.cache.write().await;
        let storage_inner = self.storages.read().await;

        let time_now = Utc::now();
        for k in storage_inner.keys() {
            if let Some(data) = cache_inner.get_mut(k) {
                data.passwords.insert(password_id.clone(), password.clone());
                data.metadata.password_count += 1;
                data.metadata.last_sync = time_now;
            } else {
                let mut data = StorageData::new();
                data.passwords.insert(password_id.clone(), password.clone());
                data.metadata.password_count += 1;
                data.metadata.last_sync = time_now;

                cache_inner.insert(*k, data);
            }
        }

        drop(cache_inner);
        drop(storage_inner);

        // 保存到存储
        self.save_data().await?;

        info!("密码 {} 已成功添加", password_id);

        Ok(())
    }

    pub async fn delete_password(&self, password_id: &str) -> Result<()> {
        let mut cache_inner = self.cache.write().await;
        let storage_inner = self.storages.read().await;

        let time_now = Utc::now();

        // 从缓存中删除
        for t in storage_inner.keys() {
            if let Some(data) = cache_inner.get_mut(t)
                && data.passwords.remove(password_id).is_some()
            {
                data.metadata.password_count -= 1;
                data.metadata.last_sync = time_now;
            }
        }

        drop(cache_inner);
        drop(storage_inner);

        // 保存到存储
        self.save_data().await?;

        Ok(())
    }

    pub async fn search_passwords(&self, query: &str) -> Result<Vec<Password>> {
        let mut ret = HashMap::new();

        let cache_inner = self.cache.read().await;
        let storage_inner = self.storages.read().await;

        // 直接从缓存中查询
        for t in storage_inner.keys() {
            if let Some(data) = cache_inner.get(t) {
                let parts = Self::search_in_storagedata(query, data);
                parts.into_iter().for_each(|p| {
                    ret.insert(p.id.clone(), p);
                });
            }
        }

        Ok(ret.into_values().collect())
    }

    #[inline]
    fn search_in_storagedata(query: &str, data: &StorageData) -> Vec<Password> {
        let mut ret = vec![];

        for p in data.passwords.values() {
            if Self::is_content_match(&p.title, query)
                || Self::is_content_match(&p.description, query)
            {
                ret.push(p.clone());
            }
        }

        ret
    }

    #[inline]
    fn is_content_match(s: &str, p: &str) -> bool {
        // 先简单的使用字符串全匹配
        s.contains(p)
    }

    pub async fn decrypt_password(&self, key: &str, data: &EncryptedData) -> Result<String> {
        crypto::decrypt_with_password(data, key)
    }

    pub async fn generate_password(&self, config: &PasswordGeneratorConfig) -> Result<String> {
        password::generate_password(config)
    }

    async fn load_data_to_cache(&self) -> Result<()> {
        let mut cache_inner = self.cache.write().await;
        let storage_inner = self.storages.read().await;

        for (t, s) in storage_inner.iter() {
            let data = s.load().await?;
            cache_inner.insert(*t, data);
        }
        Ok(())
    }

    async fn save_data(&self) -> Result<()> {
        let cache_inner = self.cache.read().await;
        let storage_inner = self.storages.read().await;

        // 保存到所有启用的存储点
        let mut err = None;
        for (target, data) in cache_inner.iter() {
            if let Some(storage) = storage_inner.get(target) {
                if let Err(e) = storage.save(data).await {
                    err = match err {
                        None => Some(e.context(format!("Failed to save to {}", target))),
                        Some(_e) => Some(anyhow!("{}\nFailed to save to {}: {}", _e, target, e)),
                    };
                }
            } else {
                err = match err {
                    None => Some(anyhow!("storage target {} is None", target)),
                    Some(e) => Some(anyhow!("{}\nstorage target {} is None", e, target)),
                };
            }
        }

        if let Some(e) = err { Err(e) } else { Ok(()) }
    }

    // 获取配置
    // pub fn get_config_ref(&self) -> Arc<RwLock<Config>> {
    //     self.config.clone()
    // }

    // 获取所有启用的存储点
    // pub fn get_enabled_storages(&self) -> Vec<(StorageTarget, Arc<dyn Storage>)> {
    //     self.storages
    //         .iter()
    //         .map(|(&target, storage)| (target, storage.clone()))
    //         .collect()
    // }

    // 从指定存储点加载数据
    // pub async fn load_from_storage(&self, target: StorageTarget) -> Result<StorageData> {
    //     let storage = self
    //         .storages
    //         .get(&target)
    //         .ok_or_else(|| anyhow!("Storage target {:?} is not enabled", target))?;
    //     storage.load().await
    // }

    // 保存数据到指定存储点
    // pub async fn save_to_storage(&self, target: StorageTarget, data: &StorageData) -> Result<()> {
    //     let storage = self
    //         .storages
    //         .get(&target)
    //         .ok_or_else(|| anyhow!("Storage target {:?} is not enabled", target))?;
    //     storage.save(data).await
    // }

    // 同步两个存储点之间的数据
    // pub async fn sync_storages(&self, from: StorageTarget, to: StorageTarget) -> Result<()> {
    //     let from_data = self.load_from_storage(from).await?;
    //     self.save_to_storage(to, &from_data).await?;
    //
    //     // 重新加载缓存
    //     self.load_data_to_cache().await?;
    //
    //     Ok(())
    // }

    // 获取存储点状态信息
    // pub async fn get_storage_status(&self) -> HashMap<StorageTarget, StorageStatus> {
    //     let mut status = HashMap::new();
    //
    //     for (&target, storage) in &self.storages {
    //         let storage_status = match storage.load().await {
    //             Ok(data) => StorageStatus {
    //                 enabled: true,
    //                 connected: true,
    //                 password_count: data.passwords.len(),
    //                 last_sync: Some(data.metadata.last_sync),
    //                 error: None,
    //             },
    //             Err(e) => StorageStatus {
    //                 enabled: true,
    //                 connected: false,
    //                 password_count: 0,
    //                 last_sync: None,
    //                 error: Some(e.to_string()),
    //             },
    //         };
    //         status.insert(target, storage_status);
    //     }
    //
    //     status
    // }

    pub async fn get_all_passwords_from_storage(
        &self,
        target: StorageTarget,
    ) -> Result<StorageData> {
        if let Some(data) = self.cache.read().await.get(&target) {
            Ok(data.clone())
        } else {
            Err(anyhow!("此存储点中没有数据"))
        }
    }
}
