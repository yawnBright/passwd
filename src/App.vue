<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SearchBar from "./components/searchbar.vue";


// 数据类型定义
interface EncryptedData {
  ciphertext: string;
  nonce: string;
  salt: string;
}

interface Password {
  id: string;
  title: string;
  description: string;
  tags: string[];
  username: string;
  encrypted_password: EncryptedData; // 仅加密密码字段
  url?: string;
  created_at: string;
  updated_at: string;
}

interface StorageMetadata {
  version: string;
  last_sync: string;
  password_count: number;
}

interface StorageData {
  metadata: StorageMetadata;
  passwords: Password[];
}

// 存储目标类型
// enum StorageTarget {
//   Local = "Local",
//   GitHub = "GitHub"
// }

// 错误信息接口
interface ErrorInfo {
  code: number;
  info: string;
}

// 密码更新请求接口
// interface PasswordUpdateRequest {
//   id: string;
//   title?: string;
//   description?: string;
//   tags?: string[];
//   username?: string;
//   password?: string;
//   url?: string;
// }

// 密码搜索查询接口
// interface PasswordSearchQuery {
//   keyword: string;
//   tags?: string[];
// }

interface PasswordCreateRequest {
  title: string;
  description: string;
  tags: string[];
  username: string;
  password: string;
  url?: string;
  key: string; // 用于加密的密码
}

// 日志显示
const logMessages = ref<string[]>([]);

// 状态管理
const passwords = ref<Password[]>([]);
const searchQuery = ref("");
const isLoading = ref(false);
const error = ref("");

// 添加密码的表单数据
const showAddForm = ref(false);
const newPassword = ref<PasswordCreateRequest>({
  title: "",
  description: "",
  tags: [],
  username: "",
  password: "",
  url: "",
  key: ""
});

// 解密密码的对话框
const showDecryptDialog = ref(false);
const decryptPassword = ref({ id: "", key: "", decrypted: "", encryptedData: null as EncryptedData | null });


function addLog(msg: string) {
  logMessages.value.push(msg);
}


// 密码生成器配置接口
interface PasswordGeneratorConfig {
  length: number;
  exclude_chars?: string;
  require_uppercase: boolean;
  require_lowercase: boolean;
  require_numbers: boolean;
  require_symbols: boolean;
}

// 密码生成器
const showGenerator = ref(false);
const generatorConfig = ref<PasswordGeneratorConfig>({
  length: 16,
  exclude_chars: undefined,
  require_uppercase: true,
  require_lowercase: true,
  require_numbers: true,
  require_symbols: true
});

// 初始化应用
async function initializeApp() {
  addLog("开始初始化应用...");
  try {
    isLoading.value = true;
    error.value = "";
    await invoke("initialize_manager");
    addLog("初始化管理器成功");
    await loadPasswords();
    addLog("初始加载密码成功");
  } catch (e) {
    let _e = e as ErrorInfo;
    error.value = `初始化失败: ${_e.info}`;
    console.error("初始化错误:", _e);
  } finally {
    isLoading.value = false;
  }
}

// 加载所有密码
async function loadPasswords() {
  try {
    const storageData: StorageData = await invoke("get_all_passwords_from_storage", { storageTarget: "local" });
    passwords.value = storageData.passwords;
  } catch (e) {
    error.value = `加载密码失败: ${e}`;
    console.error("加载密码错误:", e);
    passwords.value = [];
  }
}

// 搜索密码
async function handleSearch(query: string) {
  searchQuery.value = query;
  if (!query.trim()) {
    await loadPasswords();
    return;
  }
  
  try {
    addLog(`搜索密码: ${query}`);
    const results: Password[] = await invoke("search_passwords", { query });
    passwords.value = results;
    addLog(`搜索到 ${results.length} 条密码`);
  } catch (e) {
    error.value = `搜索失败: ${e}`;
    console.error("搜索错误:", e);
  }
}

// 添加新密码
async function addPassword() {
  if (!newPassword.value.title || !newPassword.value.username || !newPassword.value.password || !newPassword.value.key) {
    error.value = "请填写所有必填字段";
    addLog("添加密码失败: 缺少必填字段");
    return;
  }
  
  try {
    addLog(`添加密码: ${newPassword.value.title} - ${newPassword.value.username}`);
    await invoke("add_password", { request: newPassword.value });
    addLog("here")
    showAddForm.value = false;
    resetForm();
    addLog("添加密码成功");
    await loadPasswords();
  } catch (e: any) {
    const errorInfo = e as ErrorInfo;
    error.value = `添加密码失败: ${errorInfo.info}`;
    console.error("添加密码错误:", e);
  }
}

// 删除密码
async function deletePassword(id: string) {
  if (!confirm("确定要删除这个密码吗？")) return;
  
  try {
    await invoke("delete_password", { passwordId: id });
    await loadPasswords();
  } catch (e) {
    error.value = `删除密码失败: ${e}`;
    console.error("删除密码错误:", e);
  }
}

// 解密密码
async function decryptPasswordFunc(id: string, key: string) {
  try {
    const password = passwords.value.find(p => p.id === id);
    if (!password) return;
    
    const decrypted: string = await invoke("decrypt_password", { 
      password: password.encrypted_password, 
      user_password: key 
    });
    decryptPassword.value = { 
      id, 
      key, 
      decrypted,
      encryptedData: password.encrypted_password 
    };
    showDecryptDialog.value = true;
  } catch (e) {
    error.value = `解密密码失败: ${e}`;
    console.error("解密密码错误:", e);
  }
}

// 生成密码
async function generatePassword() {
  try {
    const generated: string = await invoke("generate_password", { config: generatorConfig.value });
    newPassword.value.password = generated;
    showGenerator.value = false;
  } catch (e) {
    error.value = `生成密码失败: ${e}`;
    console.error("生成密码错误:", e);
  }
}

// 重置表单
function resetForm() {
  addLog("重置添加密码表单");
  newPassword.value = {
    title: "",
    description: "",
    tags: [],
    username: "",
    password: "",
    url: "",
    key: ""
  };
}

// 标签处理
function addTag(tag: string) {
  if (tag && !newPassword.value.tags.includes(tag)) {
    newPassword.value.tags.push(tag);
  }
}

function removeTag(tag: string) {
  const index = newPassword.value.tags.indexOf(tag);
  if (index > -1) {
    newPassword.value.tags.splice(index, 1);
  }
}

// 组件挂载时初始化
onMounted(() => {
  initializeApp();
});
</script>

<template>
  <main class="container">
    <h1>密码管理器</h1>
    
    <!-- 日志显示 -->
    <!-- <div v-for="msg in logMessages" :key="msg" class="log-message">
      {{ msg }}
    </div> -->

    <!-- 错误显示 -->
    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="loading">
      正在加载...
    </div>

    <!-- 顶部搜索栏和操作按钮 -->
    <div class="header-section">
      <SearchBar 
        v-model="searchQuery" 
        placeholder="搜索密码..."
        @search="handleSearch"
      />
      <div class="action-buttons">
        <button @click="showAddForm = true" class="btn btn-primary">
          添加密码
        </button>
        <button @click="showGenerator = true" class="btn btn-secondary">
          生成密码
        </button>
      </div>
    </div>
    
    <!-- 密码列表 -->
    <div class="password-list">
      <div v-for="password in passwords" :key="password.id" class="password-card">
        <div class="password-header">
          <h3>{{ password.title }}</h3>
          <div class="password-actions">
            <button @click="decryptPasswordFunc(password.id, 'your-key')" class="btn btn-small">
              查看
            </button>
            <button @click="deletePassword(password.id)" class="btn btn-danger btn-small">
              删除
            </button>
          </div>
        </div>
        <p class="description">{{ password.description }}</p>
        <p class="username">用户名: {{ password.username }}</p>
        <div class="tags">
          <span v-for="tag in password.tags" :key="tag" class="tag">{{ tag }}</span>
        </div>
        <p class="url" v-if="password.url">网址: {{ password.url }}</p>
        <p class="date">创建时间: {{ new Date(password.created_at).toLocaleString() }}</p>
      </div>
      
      <div v-if="passwords.length === 0 && !isLoading" class="empty-state">
        暂无密码记录
      </div>
    </div>

    <!-- 添加密码对话框 -->
    <div v-if="showAddForm" class="modal">
      <div class="modal-content">
        <h2>添加新密码</h2>
        <form @submit.prevent="addPassword">
          <div class="form-group">
            <label>标题 *</label>
            <input v-model="newPassword.title" type="text" required />
          </div>
          
          <div class="form-group">
            <label>描述</label>
            <textarea v-model="newPassword.description"></textarea>
          </div>
          
          <div class="form-group">
            <label>用户名 *</label>
            <input v-model="newPassword.username" type="text" required />
          </div>
          
          <div class="form-group">
            <label>密码 *</label>
            <input v-model="newPassword.password" type="password" required />
          </div>
          
          <div class="form-group">
            <label>加密密钥 *</label>
            <input v-model="newPassword.key" type="password" required />
            <small>用于加密密码的密钥</small>
          </div>
          
          <div class="form-group">
            <label>网址</label>
            <input v-model="newPassword.url" type="url" />
          </div>
          
          <div class="form-group">
            <label>标签</label>
            <input 
              type="text" 
              placeholder="输入标签后按回车"
              @keydown.enter.prevent="addTag(($event.target as HTMLInputElement).value); ($event.target as HTMLInputElement).value = ''"
            />
            <div class="tags">
              <span v-for="tag in newPassword.tags" :key="tag" class="tag">
                {{ tag }}
                <button type="button" @click="removeTag(tag)" class="remove-tag">×</button>
              </span>
            </div>
          </div>
          
          <div class="form-actions">
            <button type="submit" class="btn btn-primary">保存</button>
            <button type="button" @click="showAddForm = false" class="btn btn-secondary">取消</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 密码生成器对话框 -->
    <div v-if="showGenerator" class="modal">
      <div class="modal-content">
        <h2>密码生成器</h2>
        <div class="form-group">
          <label>长度: {{ generatorConfig.length }}</label>
          <input v-model="generatorConfig.length" type="range" min="8" max="32" />
        </div>
        
        <div class="form-group">
          <label><input v-model="generatorConfig.require_uppercase" type="checkbox" /> 包含大写字母</label>
          <label><input v-model="generatorConfig.require_lowercase" type="checkbox" /> 包含小写字母</label>
          <label><input v-model="generatorConfig.require_numbers" type="checkbox" /> 包含数字</label>
          <label><input v-model="generatorConfig.require_symbols" type="checkbox" /> 包含特殊字符</label>
        </div>
        
        <div class="form-group">
          <label>排除字符 (可选):</label>
          <input v-model="generatorConfig.exclude_chars" type="text" placeholder="例如: 0O1lI" />
          <small>输入要从密码中排除的字符</small>
        </div>
        
        <div class="form-actions">
          <button @click="generatePassword" class="btn btn-primary">生成密码</button>
          <button @click="showGenerator = false" class="btn btn-secondary">关闭</button>
        </div>
      </div>
    </div>

    <!-- 解密密码对话框 -->
    <div v-if="showDecryptDialog" class="modal">
      <div class="modal-content">
        <h2>密码详情</h2>
        <div class="form-group">
          <label>解密后的密码:</label>
          <div class="password-display">{{ decryptPassword.decrypted }}</div>
        </div>
        <div class="form-actions">
          <button @click="showDecryptDialog = false" class="btn btn-primary">关闭</button>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  background-color: #f5f5f5;
  min-height: 100vh;
}

h1 {
  color: #333;
  margin-bottom: 20px;
}

.error-message {
  background-color: #f8d7da;
  color: #721c24;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 20px;
  border: 1px solid #f5c6cb;
}

.loading {
  text-align: center;
  padding: 20px;
  color: #666;
}

.header-section {
  display: flex;
  gap: 20px;
  align-items: center;
  margin-bottom: 20px;
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.action-buttons {
  display: flex;
  gap: 10px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.btn-primary {
  background-color: #007bff;
  color: white;
}

.btn-primary:hover {
  background-color: #0056b3;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background-color: #545b62;
}

.btn-danger {
  background-color: #dc3545;
  color: white;
}

.btn-danger:hover {
  background-color: #c82333;
}

.btn-small {
  padding: 4px 8px;
  font-size: 12px;
}

.password-list {
  display: grid;
  gap: 15px;
}

.password-card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.password-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.password-header h3 {
  margin: 0;
  color: #333;
}

.password-actions {
  display: flex;
  gap: 8px;
}

.description {
  color: #666;
  margin: 8px 0;
}

.username {
  color: #007bff;
  font-weight: bold;
  margin: 5px 0;
}

.tags {
  margin: 10px 0;
}

.tag {
  display: inline-block;
  background-color: #e9ecef;
  color: #495057;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  margin-right: 5px;
  margin-bottom: 5px;
}

.url {
  color: #28a745;
  margin: 5px 0;
}

.date {
  color: #999;
  font-size: 12px;
  margin-top: 10px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #666;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  padding: 30px;
  border-radius: 8px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-content h2 {
  margin-top: 0;
  color: #333;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  color: #333;
  font-weight: bold;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-group textarea {
  height: 60px;
  resize: vertical;
}

.form-group small {
  color: #666;
  font-size: 12px;
}

.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
}

.password-display {
  background-color: #f8f9fa;
  padding: 10px;
  border-radius: 4px;
  font-family: monospace;
  word-break: break-all;
  border: 1px solid #dee2e6;
}

.remove-tag {
  background: none;
  border: none;
  color: #dc3545;
  cursor: pointer;
  margin-left: 5px;
  font-weight: bold;
}
</style>
