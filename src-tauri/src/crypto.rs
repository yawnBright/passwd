use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// 将用户密码确定性转换为32字节密钥
/// 使用SHA-256哈希，不需要任何盐值或存储
fn password_to_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 使用密码加密数据
///
/// 特点：
/// - 用户密码通过SHA-256转换为32字节密钥
/// - 每次加密生成随机nonce，保证语义安全
///
/// # 参数
/// * `plaintext` - 要加密的明文数据
/// * `password` - 用户设置的密码
///
/// # 返回
/// * `Result<EncryptedData>` - 成功返回加密数据，失败返回错误
///
/// # 错误
/// * 加密过程中的任何错误都会返回
pub fn encrypt_with_password(plaintext: &str, password: &str) -> Result<EncryptedData> {
    // 确定性密钥派生：密码 → SHA-256 → 32字节密钥
    let key_bytes = password_to_key(password);
    let key = Key::<Aes256Gcm>::from(key_bytes);

    // 创建AES-256-GCM加密器
    let cipher = Aes256Gcm::new(&key);

    // 生成随机nonce（保证语义安全）
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    // 加密数据
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

/// 使用密码解密数据
///
/// # 参数
/// * `encrypted_data` - 加密的数据结构
/// * `password` - 用户设置的密码
///
/// # 返回
/// * `Result<String>` - 成功返回解密后的明文，失败返回错误
///
/// # 错误
/// * 解密过程中的任何错误都会返回，包括密码错误
pub fn decrypt_with_password(encrypted_data: &EncryptedData, password: &str) -> Result<String> {
    // 确定性密钥派生：密码 → SHA-256 → 32字节密钥
    let key_bytes = password_to_key(password);
    let key = Key::<Aes256Gcm>::from(key_bytes);

    // 创建AES-256-GCM解密器
    let cipher = Aes256Gcm::new(&key);

    // 使用存储的nonce
    let nonce_bytes: [u8; 12] = encrypted_data.nonce.as_slice().try_into()?;
    let nonce = Nonce::from(nonce_bytes);

    // 解密数据
    let plaintext = cipher
        .decrypt(&nonce, encrypted_data.ciphertext.as_ref())
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use crate::crypto::*;
    #[test]
    fn main() {
        let passwd = "hello world";

        let text = "你好，世界";

        let encrypted_text = encrypt_with_password(text, passwd).unwrap();

        println!("{:#?}", encrypted_text);

        let t = decrypt_with_password(&encrypted_text, passwd).unwrap();

        println!("{}", t);

        assert!(t.eq(text))
    }
}
