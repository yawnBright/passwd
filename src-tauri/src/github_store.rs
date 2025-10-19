use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    github_client::{Author, Committer, GitHubClient},
    password::EncryptedPassword,
    store::StorePoint,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubStoreConfig {
    pub owner: String,
    pub repo: String,
    pub file_path: String,
    pub branch: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

pub struct GitHubStore {
    client: GitHubClient,
    config: GitHubStoreConfig,
}

impl GitHubStore {
    pub fn new(token: String, config: GitHubStoreConfig) -> Self {
        Self {
            client: GitHubClient::new(token),
            config,
        }
    }

    fn get_author(&self) -> Option<Author> {
        if let (Some(name), Some(email)) = (&self.config.author_name, &self.config.author_email) {
            Some(Author {
                name: name.clone(),
                email: email.clone(),
            })
        } else {
            None
        }
    }

    fn get_committer(&self) -> Option<Committer> {
        self.get_author().as_ref().map(|author| Committer {
            name: author.name.clone(),
            email: author.email.clone(),
        })
    }

    async fn load_store_data(
        &self,
        master_key: &str,
    ) -> Result<HashMap<String, EncryptedPassword>> {
        match self
            .client
            .get_file_content(
                &self.config.owner,
                &self.config.repo,
                &self.config.file_path,
                self.config.branch.as_deref(),
            )
            .await
        {
            Ok(file_content) => {
                let decoded_content = STANDARD
                    .decode(&file_content.content)
                    .context("Base64解码失败")?;

                let decrypted_content = crate::crypto::decrypt(&decoded_content, master_key)
                    .context("解密存储数据失败")?;

                serde_json::from_str(&decrypted_content).context("解析存储数据失败")
            }
            Err(_) => {
                // 文件不存在，返回空的密码映射
                Ok(HashMap::new())
            }
        }
    }

    async fn save_store_data(
        &self,
        data: &HashMap<String, EncryptedPassword>,
        master_key: &str,
        message: &str,
    ) -> Result<()> {
        let serialized = serde_json::to_string(data).context("序列化存储数据失败")?;

        let encrypted =
            crate::crypto::encrypt(&serialized, master_key).context("加密存储数据失败")?;

        let encoded_content = STANDARD.encode(&encrypted);

        // 检查文件是否存在，决定是创建还是更新
        match self
            .client
            .get_file_content(
                &self.config.owner,
                &self.config.repo,
                &self.config.file_path,
                self.config.branch.as_deref(),
            )
            .await
        {
            Ok(_) => {
                // 文件存在，更新它
                self.client
                    .update_file(
                        &self.config.owner,
                        &self.config.repo,
                        &self.config.file_path,
                        &encoded_content,
                        message,
                        self.config.branch.as_deref(),
                        self.get_author(),
                        self.get_committer(),
                    )
                    .await
                    .context("更新GitHub文件失败")?;
            }
            Err(_) => {
                // 文件不存在，创建它
                self.client
                    .create_file(
                        &self.config.owner,
                        &self.config.repo,
                        &self.config.file_path,
                        &encoded_content,
                        message,
                        self.config.branch.as_deref(),
                        self.get_author(),
                        self.get_committer(),
                    )
                    .await
                    .context("创建GitHub文件失败")?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl StorePoint for GitHubStore {
    async fn save(&self, password: &EncryptedPassword, master_key: &str) -> Result<()> {
        let mut passwords = self.load_store_data(master_key).await?;
        passwords.insert(password.key.clone(), password.clone());

        self.save_store_data(
            &passwords,
            master_key,
            &format!("保存密码: {}", password.key),
        )
        .await
    }

    async fn load(&self, key: &str, master_key: &str) -> Result<Option<EncryptedPassword>> {
        let passwords = self.load_store_data(master_key).await?;
        Ok(passwords.get(key).cloned())
    }

    async fn delete(&self, key: &str, master_key: &str) -> Result<()> {
        let mut passwords = self.load_store_data(master_key).await?;
        passwords.remove(key);

        self.save_store_data(&passwords, master_key, &format!("删除密码: {}", key))
            .await
    }

    async fn list_keys(&self, master_key: &str) -> Result<Vec<String>> {
        let passwords = self.load_store_data(master_key).await?;
        Ok(passwords.keys().cloned().collect())
    }

    async fn exists(&self, key: &str, master_key: &str) -> Result<bool> {
        let passwords = self.load_store_data(master_key).await?;
        Ok(passwords.contains_key(key))
    }
}

