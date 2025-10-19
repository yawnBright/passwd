use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub key: String,
    pub value: String,
    pub label: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

impl Password {
    pub fn new(key: String, value: String, label: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            key,
            value,
            label,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn encrypt(&self, master_key: &str) -> Result<EncryptedPassword> {
        let serialized = serde_json::to_string(self).context("序列化密码失败")?;

        let encrypted = crate::crypto::encrypt(&serialized, master_key).context("加密密码失败")?;

        Ok(EncryptedPassword {
            key: self.key.clone(),
            encrypted_data: STANDARD.encode(encrypted),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPassword {
    pub key: String,
    pub encrypted_data: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EncryptedPassword {
    pub fn decrypt(&self, master_key: &str) -> Result<Password> {
        let encrypted_data = STANDARD
            .decode(&self.encrypted_data)
            .context("Base64解码失败")?;

        let decrypted =
            crate::crypto::decrypt(&encrypted_data, master_key).context("解密密码失败")?;

        serde_json::from_str(&decrypted).context("反序列化密码失败")
    }
}

pub struct PasswordGenerator {
    length: usize,
    exclude_chars: String,
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self {
            length: 16,
            exclude_chars: String::new(),
        }
    }
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            exclude_chars: String::new(),
        }
    }

    pub fn exclude_chars(mut self, chars: &str) -> Self {
        self.exclude_chars = chars.to_string();
        self
    }

    pub fn generate(&self) -> Result<String> {
        use rand::Rng;

        let mut charset = String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?");

        // 移除排除的字符
        for exclude_char in self.exclude_chars.chars() {
            charset.retain(|c| c != exclude_char);
        }

        if charset.is_empty() {
            anyhow::bail!("字符集为空，无法生成密码");
        }

        let charset_chars: Vec<char> = charset.chars().collect();
        let mut rng = rand::thread_rng();

        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset_chars.len());
                charset_chars[idx]
            })
            .collect();

        Ok(password)
    }
}
