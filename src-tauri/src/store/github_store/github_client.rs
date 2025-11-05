use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubFileContent {
    pub content: String,
    pub encoding: String,
    pub sha: String,
    pub size: i32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubCreateUpdateRequest {
    pub message: String,
    pub content: String,
    pub sha: Option<String>, // 更新时需要
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubCreateUpdateResponse {
    pub content: GithubFileContent,
    pub commit: serde_json::Value,
}

pub struct GithubClient {
    pub owner: String,
    pub repo: String,
    pub token: String,
    pub branch: String,
    pub client: reqwest::Client,
}

impl GithubClient {
    pub fn new(owner: String, repo: String, token: String, branch: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("password-manager")
            .build()
            .unwrap();

        Self {
            owner,
            repo,
            token,
            branch,
            client,
        }
    }

    pub async fn get_file(&self, path: &str) -> Result<GithubFileContent> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .query(&[("ref", &self.branch)])
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get file: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error ({}): {}", status, text));
        }

        let file_content: GithubFileContent = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        Ok(file_content)
    }

    pub async fn create_or_update_file(
        &self,
        path: &str,
        content: &str,
        message: &str,
        sha: Option<&str>,
    ) -> Result<GithubCreateUpdateResponse> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        );

        let encoded_content = general_purpose::STANDARD.encode(content);

        let request_body = GithubCreateUpdateRequest {
            message: message.to_string(),
            content: encoded_content,
            sha: sha.map(|s| s.to_string()),
            branch: self.branch.clone(),
        };

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to create/update file: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error ({}): {}", status, text));
        }

        let response_data: GithubCreateUpdateResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        Ok(response_data)
    }

    pub async fn delete_file(&self, path: &str, message: &str, sha: &str) -> Result<()> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        );

        #[derive(Serialize)]
        struct DeleteRequest {
            message: String,
            sha: String,
            branch: String,
        }

        let request_body = DeleteRequest {
            message: message.to_string(),
            sha: sha.to_string(),
            branch: self.branch.clone(),
        };

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to delete file: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error ({}): {}", status, text));
        }

        Ok(())
    }

    pub fn decode_file_content(&self, file_content: &GithubFileContent) -> Result<String> {
        if file_content.encoding != "base64" {
            return Err(anyhow!("Unsupported encoding: {}", file_content.encoding));
        }

        let decoded = general_purpose::STANDARD
            .decode(&file_content.content.replace("\n", ""))
            .map_err(|e| anyhow!("Failed to decode base64: {}", e))?;

        String::from_utf8(decoded).map_err(|e| anyhow!("Invalid UTF-8 content: {}", e))
    }
}
