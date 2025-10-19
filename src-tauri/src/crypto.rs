use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{rand_core::RngCore, SaltString};

pub fn encrypt(data: &str, master_key: &str) -> Result<Vec<u8>> {
    // 使用 Argon2 从主密钥派生加密密钥
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(master_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("派生密钥失败: {}", e))?;

    // 使用派生的密钥进行 AES-GCM 加密
    let hash_binding = password_hash.hash.unwrap();
    let hash_bytes = hash_binding.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
    let cipher = Aes256Gcm::new(key);

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密数据
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("加密失败: {}", e))?;

    // 组合 nonce 和密文
    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt(encrypted_data: &[u8], master_key: &str) -> Result<String> {
    if encrypted_data.len() < 12 {
        anyhow::bail!("加密数据格式错误");
    }

    // 提取 nonce 和密文
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 使用 Argon2 从主密钥派生解密密钥
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(master_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("派生密钥失败: {}", e))?;

    // 使用派生的密钥进行 AES-GCM 解密
    let hash_binding = password_hash.hash.unwrap();
    let hash_bytes = hash_binding.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
    let cipher = Aes256Gcm::new(key);

    // 解密数据
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("解密失败: {}", e))?;

    String::from_utf8(plaintext).context("解密后的数据不是有效的 UTF-8 字符串")
}

pub fn verify_master_key(master_key: &str, hash: &str) -> Result<bool> {
    let argon2 = Argon2::default();
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;

    Ok(argon2
        .verify_password(master_key.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn hash_master_key(master_key: &str) -> Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(master_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("哈希主密钥失败: {}", e))?;

    Ok(password_hash.to_string())
}
