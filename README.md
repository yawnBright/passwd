# 密码管理器

一个基于Tauri的跨平台密码管理器，支持本地存储和GitHub同步。

## 功能特性

- 🔐 **安全加密**：使用Argon2密钥派生和AES-256-GCM加密
- 📱 **跨平台**：基于Tauri，支持Windows、macOS、Linux
- 🔄 **GitHub同步**：可选的GitHub存储同步功能
- 🔍 **快速搜索**：支持标题、描述、标签搜索
- ⚙️ **密码生成器**：可配置的密码生成规则
- 📝 **双重加密**：可选的描述字段双重加密

## 技术栈

- **后端**：Rust + Tauri
- **前端**：Vue.js 3 + TypeScript
- **加密**：Argon2 + AES-256-GCM
- **存储**：本地JSON文件 + GitHub API

## 开发环境

### 前置要求

- Rust 1.70+
- Node.js 18+
- npm 或 pnpm

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装Tauri CLI（可选）
cargo install tauri-cli
```

### 开发运行

```bash
# 运行前端开发服务器
npm run dev

# 构建前端
npm run build

# 运行Tauri应用（需要安装Tauri CLI）
cargo tauri dev
```

### 构建发布版本

```bash
# 构建前端
npm run build

# 构建Tauri应用
cargo tauri build
```

## 项目结构

```
src-tauri/
├── src/
│   ├── main.rs          # Tauri应用入口
│   ├── config.rs        # 配置管理
│   ├── crypto.rs        # 加密服务
│   ├── manager.rs       # 密码管理器核心
│   ├── password.rs      # 密码模型
│   ├── store.rs         # 存储抽象
│   ├── github_store.rs  # GitHub存储实现
│   ├── github_client.rs # GitHub API客户端
│   └── local_store.rs   # 本地存储实现
├── Cargo.toml           # Rust依赖配置
└── tauri.conf.json      # Tauri配置

src/                     # Vue.js前端代码
├── components/          # Vue组件
├── views/              # 页面视图
├── services/           # API服务
└── types/              # TypeScript类型定义
```

## 安全特性

1. **主密码保护**：使用Argon2进行密钥派生
2. **AES-256-GCM加密**：所有敏感数据都经过加密
3. **双重加密**：可选的描述字段双重加密
4. **安全随机数**：使用操作系统提供的随机数生成器
5. **无云端依赖**：本地优先，GitHub同步可选

## 配置说明

应用支持以下配置选项：

- **安全设置**：双重加密、密码复杂度要求
- **存储设置**：本地存储路径、GitHub仓库配置
- **同步设置**：自动同步、冲突解决策略

## 许可证

MIT License
