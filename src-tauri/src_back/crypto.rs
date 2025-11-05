use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use password_hash::rand_core::RngCore as PasswordHashRngCore;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MasterKey {
    key: Vec<u8>,
}

impl MasterKey {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        // 使用HKDF风格的密钥派生
        let argon2 = Argon2::default();
        let salt_string =
            SaltString::encode_b64(salt).map_err(|e| anyhow!("Invalid salt: {}", e))?;
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        // 从哈希中提取密钥
        let binding = password_hash
            .hash
            .ok_or_else(|| anyhow!("No hash in password hash"))?;
        let hash_bytes = binding.as_bytes();
        if hash_bytes.len() < 32 {
            return Err(anyhow!("Hash too short for key derivation"));
        }

        let mut key = vec![0u8; 32];
        key.copy_from_slice(&hash_bytes[..32]);

        Ok(Self { key })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

pub struct CryptoService {
    master_key: MasterKey,
    simple_key: Key<Aes256Gcm>,
}

impl CryptoService {
    pub fn new(master_key: MasterKey) -> Self {
        // 生成简单的内置密钥（用于非敏感数据加密）
        let simple_key = Aes256Gcm::generate_key(&mut OsRng);

        Self {
            master_key,
            simple_key,
        }
    }

    // 使用主密钥加密（用于密码等敏感数据）
    pub fn encrypt_with_master_key(&self, plaintext: &str) -> Result<EncryptedData> {
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key.key);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = vec![0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt,
        })
    }

    // 使用主密钥解密
    pub fn decrypt_with_master_key(&self, encrypted: &EncryptedData) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    // 使用简单密钥加密（用于描述等非敏感数据）
    pub fn encrypt_with_simple_key(&self, plaintext: &str) -> Result<EncryptedData> {
        let cipher = Aes256Gcm::new(&self.simple_key);

        let mut nonce_bytes = vec![0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt: vec![], // 简单加密不需要salt
        })
    }

    // 使用简单密钥解密
    pub fn decrypt_with_simple_key(&self, encrypted: &EncryptedData) -> Result<String> {
        let cipher = Aes256Gcm::new(&self.simple_key);
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    // 生成随机密码
    pub fn generate_password(config: &crate::password::PasswordGeneratorConfig) -> Result<String> {
        let mut chars = String::new();

        if config.require_uppercase {
            chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        if config.require_lowercase {
            chars.push_str("abcdefghijklmnopqrstuvwxyz");
        }
        if config.require_numbers {
            chars.push_str("0123456789");
        }
        if config.require_symbols {
            chars.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if let Some(exclude) = &config.exclude_chars {
            for exclude_char in exclude.chars() {
                chars = chars.replace(exclude_char, "");
            }
        }

        if chars.is_empty() {
            return Err(anyhow!("No characters available for password generation"));
        }

        let char_vec: Vec<char> = chars.chars().collect();
        let mut password = String::new();

        for _ in 0..config.length {
            let mut rng = rand::rng();
            let idx = rng.next_u32() as usize % char_vec.len();
            password.push(char_vec[idx]);
        }

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let master_key = MasterKey::from_password("test_password", b"test_salt").unwrap();
        let crypto = CryptoService::new(master_key);

        let original = "test_password_123";
        let encrypted = crypto.encrypt_with_master_key(original).unwrap();
        let decrypted = crypto.decrypt_with_master_key(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }
}

