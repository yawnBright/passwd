use anyhow::{Context, Result};
use std::sync::Arc;

use crate::{
    password::{Password, PasswordGenerator},
    store::StorePoint,
};

pub struct PasswordManager {
    store: Arc<dyn StorePoint>,
    master_key: String,
}

impl PasswordManager {
    pub fn new(store: Arc<dyn StorePoint>, master_key: String) -> Self {
        Self { store, master_key }
    }

    /// 新增密码
    pub async fn add_password(&self, key: String, value: String, label: String) -> Result<()> {
        if self.store.exists(&key, &self.master_key).await? {
            anyhow::bail!("密码键 '{}' 已存在", key);
        }

        let password = Password::new(key, value, label);
        let encrypted_password = password.encrypt(&self.master_key)?;

        self.store
            .save(&encrypted_password, &self.master_key)
            .await
            .context("保存密码失败")?;

        Ok(())
    }

    /// 删除密码
    pub async fn delete_password(&self, key: &str) -> Result<()> {
        if !self.store.exists(key, &self.master_key).await? {
            anyhow::bail!("密码键 '{}' 不存在", key);
        }

        self.store
            .delete(key, &self.master_key)
            .await
            .context("删除密码失败")?;

        Ok(())
    }

    /// 查询密码
    pub async fn get_password(&self, key: &str) -> Result<Option<Password>> {
        let encrypted_password = self.store.load(key, &self.master_key).await?;

        match encrypted_password {
            Some(encrypted) => {
                let password = encrypted.decrypt(&self.master_key)?;
                Ok(Some(password))
            }
            None => Ok(None),
        }
    }

    /// 更新密码
    pub async fn update_password(&self, key: String, value: String, label: String) -> Result<()> {
        if !self.store.exists(&key, &self.master_key).await? {
            anyhow::bail!("密码键 '{}' 不存在", key);
        }

        let password = Password::new(key, value, label);
        let encrypted_password = password.encrypt(&self.master_key)?;

        self.store
            .save(&encrypted_password, &self.master_key)
            .await
            .context("更新密码失败")?;

        Ok(())
    }

    /// 列出所有密码键
    pub async fn list_passwords(&self) -> Result<Vec<String>> {
        self.store
            .list_keys(&self.master_key)
            .await
            .context("获取密码列表失败")
    }

    /// 生成推荐密码
    pub fn generate_password(&self, length: usize, exclude_chars: Option<&str>) -> Result<String> {
        let mut generator = PasswordGenerator::new(length);

        if let Some(exclude) = exclude_chars {
            generator = generator.exclude_chars(exclude);
        }

        generator.generate()
    }

    /// 搜索密码（通过标签或键名）
    pub async fn search_passwords(&self, query: &str) -> Result<Vec<Password>> {
        let keys = self.store.list_keys(&self.master_key).await?;
        let mut results = Vec::new();

        for key in keys {
            if let Some(password) = self.get_password(&key).await? {
                if password.key.contains(query) || password.label.contains(query) {
                    results.push(password);
                }
            }
        }

        // 去重并排序结果
        results.sort_by(|a, b| a.key.cmp(&b.key));
        results.dedup_by(|a, b| a.key == b.key);

        Ok(results)
    }

    /// 验证主密钥
    pub fn verify_master_key(&self, key: &str) -> bool {
        // 这里可以实现更复杂的验证逻辑
        // 目前简单比较，实际应用中应该使用密钥派生函数
        self.master_key == key
    }

    /// 获取存储统计信息
    pub async fn get_stats(&self) -> Result<ManagerStats> {
        let keys = self.store.list_keys(&self.master_key).await?;
        let total_passwords = keys.len();

        let mut total_length = 0;
        let mut strongest_password = 0;
        let mut weakest_password = usize::MAX;

        for key in &keys {
            if let Some(password) = self.get_password(key).await? {
                let length = password.value.len();
                total_length += length;
                strongest_password = strongest_password.max(length);
                weakest_password = weakest_password.min(length);
            }
        }

        let average_length = if total_passwords > 0 {
            total_length / total_passwords
        } else {
            0
        };

        Ok(ManagerStats {
            total_passwords,
            average_length,
            strongest_password: if strongest_password == 0 {
                None
            } else {
                Some(strongest_password)
            },
            weakest_password: if weakest_password == usize::MAX {
                None
            } else {
                Some(weakest_password)
            },
        })
    }
}

#[derive(Debug)]
pub struct ManagerStats {
    pub total_passwords: usize,
    pub average_length: usize,
    pub strongest_password: Option<usize>,
    pub weakest_password: Option<usize>,
}
