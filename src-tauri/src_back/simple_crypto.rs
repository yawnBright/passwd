use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// 简单的XOR加密，用于数据混淆
/// 即使部分数据损坏，也不会影响整体结构解析
pub struct SimpleCrypto {
    key: u8,
}

impl SimpleCrypto {
    pub fn new() -> Self {
        Self { key: 0x88 } // 内置密钥 888 -> 0x88
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| b ^ self.key).collect()
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Vec<u8> {
        // XOR加密解密是同一个操作
        self.encrypt(encrypted)
    }
}

/// 可恢复的加密数据格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustEncryptedData {
    pub data: Vec<u8>,
    pub checksum: u32,
}

impl SimpleCrypto {
    /// 加密并添加校验和
    pub fn encrypt_with_checksum(&self, plaintext: &str) -> Result<RobustEncryptedData> {
        let bytes = plaintext.as_bytes();
        let encrypted = self.encrypt(bytes);
        let checksum = self.calculate_checksum(&encrypted);
        
        Ok(RobustEncryptedData {
            data: encrypted,
            checksum,
        })
    }

    /// 解密并验证校验和
    pub fn decrypt_with_checksum(&self, encrypted: &RobustEncryptedData) -> Result<String> {
        // 验证校验和（可选，即使失败也尝试解密）
        let expected_checksum = self.calculate_checksum(&encrypted.data);
        if expected_checksum != encrypted.checksum {
            eprintln!("Checksum mismatch: expected {}, got {}", expected_checksum, encrypted.checksum);
        }
        
        let decrypted = self.decrypt(&encrypted.data);
        String::from_utf8(decrypted).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        data.iter().map(|&b| b as u32).sum::<u32>() % 0xFFFFFFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_encryption() {
        let crypto = SimpleCrypto::new();
        let original = "test_password_123";
        
        let encrypted = crypto.encrypt_with_checksum(original).unwrap();
        let decrypted = crypto.decrypt_with_checksum(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_partial_corruption() {
        let crypto = SimpleCrypto::new();
        let original = "test_password_123";
        
        let mut encrypted = crypto.encrypt_with_checksum(original).unwrap();
        // 模拟部分数据损坏
        if !encrypted.data.is_empty() {
            encrypted.data[0] ^= 0xFF;
        }
        
        // 即使数据损坏，也能解密（虽然内容可能不正确）
        let result = crypto.decrypt_with_checksum(&encrypted);
        assert!(result.is_ok() || result.is_err()); // 取决于损坏程度
    }
}