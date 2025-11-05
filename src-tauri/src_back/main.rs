// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use config::Config;
use github_client::GithubClient;
use github_store::GithubStorage;
use manager::{PasswordManager, StorageStatus, StorageTarget};
use password::{Password, PasswordCreateRequest, PasswordGeneratorConfig};
use std::collections::HashMap;
use std::sync::Arc;
use store::{LocalStorage, Storage};
use tokio::sync::RwLock;

mod config;
mod crypto;
mod github_client;
mod github_store;
mod manager;
mod password;
mod simple_crypto;
mod store;

struct AppState {
    password_manager: Arc<RwLock<Option<PasswordManager>>>,
    config: Arc<RwLock<Config>>,
}

fn main() {
    init_password_manager();
    run_tauri_app();
}

fn init_password_manager() {
    // 初始化配置
    let config_path = Config::get_config_path();
    let _config = match Config::load_from_file(&config_path) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config::default();
            if let Err(e) = default_config.save_to_file(&config_path) {
                eprintln!("Failed to save default config: {}", e);
            }
            default_config
        }
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            password_manager: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(Config::default())),
        })
        .invoke_handler(tauri::generate_handler![
            initialize_manager,
            add_password,
            delete_password,
            search_passwords,
            search_passwords_in_storage,
            get_all_passwords,
            get_all_passwords_from_storage,
            get_password_by_id,
            get_password_by_id_from_storage,
            decrypt_password,
            generate_password,
            get_current_config,
            save_config,
            get_storage_status,
            sync_storages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize)]
struct ErrorInfo {
    code: isize,
    info: String,
}

#[derive(serde::Serialize)]
struct InitializeResult {
    is_first_setup: bool,
    has_encrypted_data: bool,
}

impl From<anyhow::Error> for ErrorInfo {
    fn from(error: anyhow::Error) -> Self {
        ErrorInfo {
            code: -1,
            info: error.to_string(),
        }
    }
}

#[tauri::command]
async fn initialize_manager(
    state: tauri::State<'_, AppState>,
) -> Result<InitializeResult, ErrorInfo> {
    let config_path = Config::get_config_path();
    let config = Config::load_from_file(&config_path).unwrap_or_else(|_| Config::default());

    // 检查是否存在加密数据 - 检查所有启用的存储点
    let mut has_encrypted_data = false;

    // 检查本地存储
    if config.storage.local_storage.enabled {
        let local_storage = Arc::new(LocalStorage::new(
            config.storage.local_storage.data_path.clone(),
        ));
        if let Ok(result) = local_storage.has_encrypted_data().await {
            if result {
                has_encrypted_data = true;
            }
        }
    }

    // 如果没有本地数据，检查GitHub存储
    if !has_encrypted_data && config.storage.github_storage.is_some() {
        if let Some(github_config) = &config.storage.github_storage {
            if github_config.enabled {
                let client = GithubClient::new(
                    github_config.owner.clone(),
                    github_config.repo.clone(),
                    github_config.token.clone(),
                    github_config.branch.clone(),
                );
                let github_storage =
                    Arc::new(GithubStorage::new(client, github_config.file_path.clone()));
                if let Ok(result) = github_storage.has_encrypted_data().await {
                    if result {
                        has_encrypted_data = true;
                    }
                }
            }
        }
    }

    // 简化场景判断：只基于是否有加密数据
    let is_first_setup = !has_encrypted_data;

    // 创建管理器（不再需要主密码）
    let password_manager = PasswordManager::new(config.clone())
        .await
        .map_err(ErrorInfo::from)?;

    // 更新状态
    *state.password_manager.write().await = Some(password_manager);
    *state.config.write().await = config.clone();

    Ok(InitializeResult {
        is_first_setup,
        has_encrypted_data,
    })
}

#[tauri::command]
async fn add_password(
    request: PasswordCreateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<Password, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager.add_password(request).await.map_err(ErrorInfo::from)
}

#[tauri::command]
async fn delete_password(
    password_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .delete_password(&password_id)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn search_passwords(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .search_passwords(&query)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_all_passwords(state: tauri::State<'_, AppState>) -> Result<Vec<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager.get_all_passwords().await.map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_password_by_id(
    password_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .get_password_by_id(&password_id)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn decrypt_password(
    password: Password,
    user_password: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .decrypt_password_with_key(&password, &user_password)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn generate_password(
    config: PasswordGeneratorConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .generate_password(&config)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_current_config(state: tauri::State<'_, AppState>) -> Result<Config, ErrorInfo> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
async fn save_config(config: Config, state: tauri::State<'_, AppState>) -> Result<(), ErrorInfo> {
    let config_path = Config::get_config_path();
    config.save_to_file(&config_path).map_err(ErrorInfo::from)?;

    *state.config.write().await = config;
    Ok(())
}

#[tauri::command]
async fn search_passwords_in_storage(
    query: String,
    storage_target: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    let target = match storage_target.as_str() {
        "local" => StorageTarget::Local,
        "github" => StorageTarget::GitHub,
        _ => {
            return Err(ErrorInfo {
                code: 400,
                info: "Invalid storage target".to_string(),
            })
        }
    };

    manager
        .search_passwords_in_storage(&query, target)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_all_passwords_from_storage(
    storage_target: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    let target = match storage_target.as_str() {
        "local" => StorageTarget::Local,
        "github" => StorageTarget::GitHub,
        _ => {
            return Err(ErrorInfo {
                code: 400,
                info: "Invalid storage target".to_string(),
            })
        }
    };

    manager
        .get_all_passwords_from_storage(target)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_password_by_id_from_storage(
    password_id: String,
    storage_target: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<Password>, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    let target = match storage_target.as_str() {
        "local" => StorageTarget::Local,
        "github" => StorageTarget::GitHub,
        _ => {
            return Err(ErrorInfo {
                code: 400,
                info: "Invalid storage target".to_string(),
            })
        }
    };

    manager
        .get_password_by_id_from_storage(&password_id, target)
        .await
        .map_err(ErrorInfo::from)
}

#[tauri::command]
async fn get_storage_status(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    let status = manager.get_storage_status().await;
    let mut result = HashMap::new();

    for (target, status) in status {
        let key = match target {
            StorageTarget::Local => "local",
            StorageTarget::GitHub => "github",
            StorageTarget::All => "all",
        };
        result.insert(key.to_string(), status);
    }

    Ok(serde_json::to_value(result).map_err(|e| ErrorInfo {
        code: 500,
        info: format!("Failed to serialize storage status: {}", e),
    })?)
}

#[tauri::command]
async fn sync_storages(
    from_storage: String,
    to_storage: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    let from_target = match from_storage.as_str() {
        "local" => StorageTarget::Local,
        "github" => StorageTarget::GitHub,
        _ => {
            return Err(ErrorInfo {
                code: 400,
                info: "Invalid from storage target".to_string(),
            })
        }
    };

    let to_target = match to_storage.as_str() {
        "local" => StorageTarget::Local,
        "github" => StorageTarget::GitHub,
        _ => {
            return Err(ErrorInfo {
                code: 400,
                info: "Invalid to storage target".to_string(),
            })
        }
    };

    manager
        .sync_storages(from_target, to_target)
        .await
        .map_err(ErrorInfo::from)
}
