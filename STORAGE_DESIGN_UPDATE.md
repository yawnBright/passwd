# 存储点设计重构总结

## 概述
本次重构实现了用户提出的存储点平等地位设计，使本地存储和GitHub存储地位相同，配置开启即生效，各存储点独立操作。

## 主要变更

### 1. 存储点平等地位设计
- **问题**: 原有设计中GitHub存储作为本地存储的附属，地位不平等
- **解决方案**: 
  - 引入 `StorageTarget` 枚举（Local、GitHub、All）
  - 使用 `HashMap<StorageTarget, Arc<dyn Storage>>` 统一管理存储点
  - 移除 `local_storage` 和 `github_storage` 字段，改为统一的 `storages` 管理

### 2. 存储点独立操作
- **新增功能**:
  - `get_storage(target)` - 获取指定存储点实例
  - `get_enabled_storages()` - 获取所有启用存储点
  - `load_from_storage(target)` - 从指定存储点加载数据
  - `save_to_storage(target, data)` - 保存数据到指定存储点
  - `search_passwords_in_storage(query, target)` - 在指定存储点搜索
  - `get_all_passwords_from_storage(target)` - 获取指定存储点所有密码
  - `get_password_by_id_from_storage(id, target)` - 从指定存储点获取密码

### 3. 存储点同步功能
- **新增功能**:
  - `sync_storages(from, to)` - 同步两个存储点数据
  - `get_storage_status()` - 获取存储点状态信息

### 4. 数据加载策略
- **优化**: 按优先级加载（本地优先于GitHub），数据合并时后面的覆盖前面的
- **缓存机制**: 所有存储点数据合并后统一缓存

### 5. 前端接口扩展
- **新增Tauri命令**:
  - `search_passwords_in_storage` - 指定存储点搜索
  - `get_all_passwords_from_storage` - 指定存储点获取所有密码
  - `get_password_by_id_from_storage` - 指定存储点获取密码
  - `get_storage_status` - 获取存储点状态
  - `sync_storages` - 同步存储点

## 技术实现

### 核心结构变更
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageTarget {
    Local,
    GitHub,
    All, // 查询时使用，表示查询所有存储点
}

pub struct PasswordManager {
    storages: HashMap<StorageTarget, Arc<dyn Storage>>,
    // ... 其他字段
}
```

### 存储状态结构
```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageStatus {
    pub enabled: bool,
    pub connected: bool,
    pub password_count: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}
```

## 测试覆盖
- **集成测试**: 验证存储点平等设计和同步功能
- **单元测试**: 确保现有功能不受影响

## 优势
1. **平等地位**: 本地和GitHub存储地位相同，配置开启即生效
2. **独立操作**: 可以单独查询、保存、同步指定存储点
3. **灵活查询**: 支持指定存储点查询，未指定时默认合并所有存储点
4. **状态监控**: 提供详细的存储点状态信息
5. **数据同步**: 支持存储点间的数据迁移和同步
6. **向后兼容**: 保持现有API接口不变

## 使用示例

### 查询指定存储点
```rust
// 查询本地存储
let local_passwords = manager.get_all_passwords_from_storage(StorageTarget::Local).await?;

// 查询所有存储点（合并数据）
let all_passwords = manager.get_all_passwords_from_storage(StorageTarget::All).await?;
```

### 同步存储点
```rust
// 从本地同步到GitHub
manager.sync_storages(StorageTarget::Local, StorageTarget::GitHub).await?;
```

### 获取存储状态
```rust
let status = manager.get_storage_status().await;
for (target, info) in status {
    println!("{:?}: enabled={}, connected={}, count={}", 
        target, info.enabled, info.connected, info.password_count);
}
```

## 后续扩展
该设计为后续功能扩展提供了良好基础：
- 支持更多存储后端（如AWS S3、Google Drive等）
- 实现更复杂的同步策略（增量同步、冲突解决等）
- 添加存储点健康检查和自动恢复功能