use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// use crate::simple_crypto::RobustEncryptedData;
use crate::crypto::EncryptedData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub id: String,
    /// 标题 用于展示和搜索
    pub title: String,
    /// 描述 用于展示和搜索
    pub description: String, // 明文描述，不再加密
    /// 标签 用于分类
    pub tags: Vec<String>,
    pub username: String,                  // 明文用户名，不再加密
    pub encrypted_password: EncryptedData, // 仅加密密码字段
    pub url: Option<String>,               // 明文URL，不再加密
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordCreateRequest {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub username: String,
    /// 明文密码
    pub password: String,
    pub url: Option<String>,
    pub key: String, // 用于加密的密码
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
    pub fn new(request: PasswordCreateRequest, encrypted_password: EncryptedData) -> Self {
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
    pub fn update(&mut self, request: PasswordUpdateRequest, encrypted_password: EncryptedData) {
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
        // if let Some(password) = encrypted_password {
        //     self.encrypted_password = password;
        // }
        self.encrypted_password = encrypted_password;
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

/// 根据配置生成复杂密码
///
/// # 参数
/// * `config` - 密码生成配置
///
/// # 返回
/// * `Result<String, String>` - 成功返回生成的密码，失败返回错误信息
///
/// # 示例
/// ```
/// let config = PasswordGeneratorConfig {
///     length: 12,
///     exclude_chars: Some("O0l1".to_string()),
///     require_uppercase: true,
///     require_lowercase: true,
///     require_numbers: true,
///     require_symbols: true,
/// };
/// let password = generate_password(config)?;
/// ```
pub fn generate_password(config: &PasswordGeneratorConfig) -> Result<String> {
    // 定义字符集
    const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
    const NUMBERS: &str = "0123456789";
    const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

    // 根据配置构建可用字符集
    let mut available_chars = String::new();
    let mut required_chars = Vec::new();

    // 添加小写字母
    if config.require_lowercase {
        available_chars.push_str(LOWERCASE);
        // 确保至少包含一个小写字母
        required_chars.push(get_random_char(LOWERCASE));
    }

    // 添加大写字母
    if config.require_uppercase {
        available_chars.push_str(UPPERCASE);
        // 确保至少包含一个大写字母
        required_chars.push(get_random_char(UPPERCASE));
    }

    // 添加数字
    if config.require_numbers {
        available_chars.push_str(NUMBERS);
        // 确保至少包含一个数字
        required_chars.push(get_random_char(NUMBERS));
    }

    // 添加特殊符号
    if config.require_symbols {
        available_chars.push_str(SYMBOLS);
        // 确保至少包含一个特殊符号
        required_chars.push(get_random_char(SYMBOLS));
    }

    // 如果没有选择任何字符类型，返回错误
    if available_chars.is_empty() {
        return Err(anyhow!("至少需要选择一种字符类型"));
    }

    // 处理排除字符
    let mut filtered_chars = available_chars.clone();
    if let Some(exclude) = &config.exclude_chars {
        for exclude_char in exclude.chars() {
            filtered_chars = filtered_chars.replace(exclude_char, "");
        }
    }

    // 如果过滤后没有可用字符，返回错误
    if filtered_chars.is_empty() {
        return Err(anyhow!("排除字符后没有可用字符"));
    }

    // 生成随机密码
    let mut password_chars = Vec::new();

    // 首先添加必需的字符
    password_chars.extend(&required_chars);

    // 计算还需要多少字符
    let remaining_length = config.length.saturating_sub(required_chars.len());

    // 添加剩余的随机字符
    for _ in 0..remaining_length {
        password_chars.push(get_random_char(&filtered_chars));
    }

    // 打乱字符顺序以增加随机性
    shuffle_chars(&mut password_chars);

    // 组合成最终密码
    let password: String = password_chars.into_iter().collect();

    Ok(password)
}

/// 从字符串中随机选择一个字符
fn get_random_char(chars: &str) -> char {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let rng = simple_rng(seed);
    let index = rng % chars.len() as u64;
    chars.chars().nth(index as usize).unwrap_or('a')
}

/// 简单的线性同余随机数生成器
fn simple_rng(mut seed: u64) -> u64 {
    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    seed % (1u64 << 31)
}

/// 打乱字符数组
fn shuffle_chars(chars: &mut [char]) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut rng = simple_rng(seed);

    // Fisher-Yates 洗牌算法
    for i in (1..chars.len()).rev() {
        rng = simple_rng(rng);
        let j = (rng % (i as u64 + 1)) as usize;
        chars.swap(i, j);
    }
}
