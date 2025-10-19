use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{
    Client, Response,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub encoding: String,
    pub size: i64,
    pub name: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Committer {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUpdateRequest {
    pub message: String,
    pub content: String,
    pub sha: Option<String>,
    pub branch: Option<String>,
    pub author: Option<Author>,
    pub committer: Option<Committer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUpdateResponse {
    pub content: Option<FileContent>,
    pub commit: CommitInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub url: String,
    pub html_url: String,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub date: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub message: String,
    pub sha: String,
    pub branch: Option<String>,
    pub author: Option<Author>,
    pub committer: Option<Committer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub content: Option<serde_json::Value>,
    pub commit: CommitInfo,
}

pub struct GitHubClient {
    client: Client,
    token: String,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://api.github.com".to_string(),
        }
    }

    fn build_auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers
    }

    pub async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
    ) -> Result<FileContent> {
        let mut url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.base_url, owner, repo, path
        );

        if let Some(branch_name) = branch {
            url.push_str(&format!("?ref={}", branch_name));
        }

        let response = self
            .client
            .get(&url)
            .headers(self.build_auth_headers())
            .send()
            .await
            .context("Failed to send request to get file content")?;

        self.handle_response(response).await
    }

    pub async fn create_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        message: &str,
        branch: Option<&str>,
        author: Option<Author>,
        committer: Option<Committer>,
    ) -> Result<CreateUpdateResponse> {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.base_url, owner, repo, path
        );

        let encoded_content = STANDARD.encode(content.as_bytes());

        let request_body = CreateUpdateRequest {
            message: message.to_string(),
            content: encoded_content,
            sha: None,
            branch: branch.map(|s| s.to_string()),
            author,
            committer,
        };

        let response = self
            .client
            .put(&url)
            .headers(self.build_auth_headers())
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to create file")?;

        self.handle_response(response).await
    }

    pub async fn update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        message: &str,
        branch: Option<&str>,
        author: Option<Author>,
        committer: Option<Committer>,
    ) -> Result<CreateUpdateResponse> {
        let file_info = self.get_file_content(owner, repo, path, branch).await?;

        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.base_url, owner, repo, path
        );

        let encoded_content = STANDARD.encode(content.as_bytes());

        let request_body = CreateUpdateRequest {
            message: message.to_string(),
            content: encoded_content,
            sha: Some(file_info.sha),
            branch: branch.map(|s| s.to_string()),
            author,
            committer,
        };

        let response = self
            .client
            .put(&url)
            .headers(self.build_auth_headers())
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to update file")?;

        self.handle_response(response).await
    }

    pub async fn delete_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        message: &str,
        branch: Option<&str>,
        author: Option<Author>,
        committer: Option<Committer>,
    ) -> Result<DeleteResponse> {
        let file_info = self.get_file_content(owner, repo, path, branch).await?;

        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.base_url, owner, repo, path
        );

        let request_body = DeleteRequest {
            message: message.to_string(),
            sha: file_info.sha,
            branch: branch.map(|s| s.to_string()),
            author,
            committer,
        };

        let response = self
            .client
            .delete(&url)
            .headers(self.build_auth_headers())
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to delete file")?;

        self.handle_response(response).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read response text")?;

        if status.is_success() {
            serde_json::from_str(&text).context("Failed to parse response JSON")
        } else {
            Err(anyhow::anyhow!("GitHub API error ({}): {}", status, text))
        }
    }
}
