mod github_client;

use crate::store::{Storage, StorageData, StorageMetadata};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use github_client::GithubClient;
use std::collections::HashMap;

pub struct GithubStorage {
    client: GithubClient,
    file_path: String,
}

impl GithubStorage {
    pub fn new(
        owner: String,
        repo: String,
        token: String,
        branch: String,
        file_path: String,
    ) -> Self {
        let client = GithubClient::new(owner, repo, token, branch);
        Self { client, file_path }
    }
}

#[async_trait]
impl Storage for GithubStorage {
    async fn load(&self) -> Result<StorageData> {
        match self.client.get_file(&self.file_path).await {
            Ok(file_content) => {
                let content = self.client.decode_file_content(&file_content)?;
                let data: StorageData = serde_json::from_str(&content)?;
                Ok(data)
            }
            Err(e) => {
                // 如果文件不存在，返回空数据
                if e.to_string().contains("404") {
                    Ok(StorageData {
                        metadata: StorageMetadata {
                            version: "1.0.0".to_string(),
                            last_sync: chrono::Utc::now(),
                            password_count: 0,
                        },
                        passwords: HashMap::new(),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn save(&self, data: &StorageData) -> Result<()> {
        let content = serde_json::to_string_pretty(data)?;

        // 尝试获取现有文件的SHA（如果存在）
        let sha = match self.client.get_file(&self.file_path).await {
            Ok(file_content) => Some(file_content.sha),
            Err(_) => None,
        };

        let message = format!("Update passwords - {} items", data.metadata.password_count);

        self.client
            .create_or_update_file(&self.file_path, &content, &message, sha.as_deref())
            .await?;

        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        // 尝试获取仓库信息来测试连接
        let url = format!(
            "https://api.github.com/repos/{}/{}",
            self.client.owner.as_str(),
            self.client.repo.as_str()
        );

        let response = self
            .client
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.client.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to GitHub: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error ({}): {}", status, text));
        }

        Ok(())
    }

    async fn has_encrypted_data(&self) -> Result<bool> {
        match self.load().await {
            Ok(data) => Ok(!data.passwords.is_empty()),
            Err(_) => Ok(false),
        }
    }
}
