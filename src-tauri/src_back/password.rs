use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::simple_crypto::RobustEncryptedData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub id: String,
    pub title: String,
    pub description: String, // 明文描述，不再加密
    pub tags: Vec<String>,
    pub username: String, // 明文用户名，不再加密
    pub encrypted_password: RobustEncryptedData, // 仅加密密码字段
    pub url: Option<String>, // 明文URL，不再加密
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordCreateRequest {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub username: String,
    pub password: String, // 明文密码
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PasswordUpdateRequest {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub username: Option<String>,
    pub password: Option<String>, // 明文密码，可选更新
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PasswordSearchQuery {
    pub keyword: String,
    pub tags: Option<Vec<String>>,
}

impl Password {
    pub fn new(request: PasswordCreateRequest, encrypted_password: RobustEncryptedData) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: request.title,
            description: request.description,
            tags: request.tags,
            username: request.username,
            encrypted_password,
            url: request.url,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, request: PasswordUpdateRequest, encrypted_password: Option<RobustEncryptedData>) {
        if let Some(title) = request.title {
            self.title = title;
        }
        if let Some(description) = request.description {
            self.description = description;
        }
        if let Some(tags) = request.tags {
            self.tags = tags;
        }
        if let Some(username) = request.username {
            self.username = username;
        }
        if let Some(password) = encrypted_password {
            self.encrypted_password = password;
        }
        if let Some(url) = request.url {
            self.url = Some(url);
        }
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordGeneratorConfig {
    pub length: usize,
    pub exclude_chars: Option<String>,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
}

impl Default for PasswordGeneratorConfig {
    fn default() -> Self {
        Self {
            length: 16,
            exclude_chars: None,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: true,
        }
    }
}