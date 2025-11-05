# 贡献指南

感谢您对本项目的兴趣！本指南将帮助您了解如何为这个项目做出贡献。

## 开发环境设置

### 前置要求

- Rust 1.70+
- Node.js 18+
- npm 或 pnpm
- Git

### 环境配置

1. **克隆项目**
   ```bash
   git clone https://github.com/your-username/passwd.git
   cd passwd
   ```

2. **安装前端依赖**
   ```bash
   npm install
   ```

3. **安装Tauri CLI（可选）**
   ```bash
   cargo install tauri-cli
   ```

4. **验证环境**
   ```bash
   # 检查Rust编译
   cd src-tauri && cargo check
   
   # 检查前端构建
   cd .. && npm run build
   ```

## 开发流程

### 1. 创建功能分支

```bash
git checkout -b feature/your-feature-name
```

### 2. 开发规范

#### Rust代码规范

- 遵循Rust官方编码规范
- 使用`cargo fmt`格式化代码
- 使用`cargo clippy`检查代码质量
- 为所有公共函数编写文档注释

```rust
/// 加密给定的密码
/// 
/// # Arguments
/// * `password` - 要加密的密码
/// 
/// # Returns
/// 返回加密后的数据或错误
/// 
/// # Examples
/// ```
/// let encrypted = encrypt_password("my_password")?;
/// ```
pub fn encrypt_password(password: &str) -> Result<EncryptedData> {
    // 实现代码
}
```

#### 前端代码规范

- 遵循Vue.js风格指南
- 使用TypeScript严格模式
- 组件使用组合式API
- 为复杂逻辑编写单元测试

```typescript
// 推荐使用组合式API
<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  title: string
  count?: number
}

const props = withDefaults(defineProps<Props>(), {
  count: 0
})

const doubled = computed(() => props.count * 2)
</script>
```

### 3. 测试要求

#### Rust测试

- 为所有核心业务逻辑编写单元测试
- 使用`cargo test`运行测试
- 确保测试覆盖率达到80%以上

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = MasterKey::from_password("test", b"salt").unwrap();
        let crypto = CryptoService::new(key);
        
        let original = "test_password";
        let encrypted = crypto.encrypt_with_master_key(original).unwrap();
        let decrypted = crypto.decrypt_with_master_key(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
    }
}
```

#### 前端测试

- 为关键组件编写测试
- 使用Vue Test Utils进行组件测试
- 确保UI交互逻辑正确

### 4. 提交规范

使用规范的提交信息格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型说明：
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

示例：
```
feat(crypto): 添加ChaCha20-Poly1305加密支持

- 新增ChaCha20CryptoService实现
- 支持在AES-256-GCM和ChaCha20-Poly1305之间切换
- 更新配置结构支持算法选择

Closes #123
```

## 代码审查

### 1. 自审查清单

在提交PR前，请检查：

- [ ] 代码通过所有测试
- [ ] 没有编译警告
- [ ] 遵循编码规范
- [ ] 更新了相关文档
- [ ] 添加了必要的注释

### 2. 审查标准

审查人员会关注：

- **安全性**：是否引入安全风险
- **性能**：是否影响系统性能
- **可维护性**：代码是否易于理解和维护
- **测试覆盖**：是否有足够的测试
- **文档**：是否更新了相关文档

## 报告问题

### 1. Bug报告

报告bug时请提供：

- 问题描述
- 复现步骤
- 期望行为
- 实际行为
- 环境信息（操作系统、版本等）
- 相关日志或错误信息

### 2. 功能请求

请求新功能时请说明：

- 功能描述
- 使用场景
- 预期效果
- 可能的实现方案

## 安全报告

如发现安全漏洞，请：

1. **不要公开报告**，请私信维护者
2. 提供详细的问题描述
3. 如可能，提供修复建议
4. 给予合理时间修复后再公开

## 文档贡献

### 1. 代码文档

- 为所有公共API编写文档
- 包含使用示例
- 说明参数和返回值
- 标注可能的错误情况

### 2. 用户文档

- 更新README.md
- 添加使用教程
- 更新配置说明
- 提供故障排除指南

## 性能优化

### 1. 基准测试

为性能关键代码添加基准测试：

```rust
#[bench]
fn bench_encryption(b: &mut Bencher) {
    let key = MasterKey::from_password("test", b"salt").unwrap();
    let crypto = CryptoService::new(key);
    
    b.iter(|| {
        crypto.encrypt_with_master_key("test_password").unwrap()
    });
}
```

### 2. 性能分析

使用工具进行性能分析：

```bash
# Rust性能分析
cargo bench
cargo flamegraph

# 前端性能分析
npm run build -- --profile
```

## 发布流程

### 1. 版本管理

遵循语义化版本（Semantic Versioning）：

- MAJOR：不兼容的API变更
- MINOR：向下兼容的功能添加
- PATCH：向下兼容的bug修复

### 2. 发布准备

- 更新版本号
- 更新CHANGELOG.md
- 确保所有测试通过
- 准备发布说明

### 3. 构建发布

```bash
# 构建前端
npm run build

# 构建Tauri应用
cargo tauri build

# 创建发布包
# 构建产物在 src-tauri/target/release/bundle/
```

## 社区准则

### 1. 行为准则

- 保持友善和尊重
- 欢迎新贡献者
- 建设性反馈
- 避免人身攻击

### 2. 沟通方式

- 使用清晰、简洁的语言
- 提供具体的例子
- 耐心解答问题
- 承认错误并学习

## 获取帮助

如有疑问，可以：

1. 查看项目文档
2. 搜索已有issue
3. 在讨论区提问
4. 联系维护者

感谢您对项目的贡献！ 🎉