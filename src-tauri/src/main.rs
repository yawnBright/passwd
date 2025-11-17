// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use config::Config;
use crypto::EncryptedData;
use manager::PasswordManager;
use password::{Password, PasswordCreateRequest, PasswordGeneratorConfig};
use std::sync::Arc;
use store::StorageData;
use store::StorageTarget;
use tokio::sync::RwLock;

mod config;
mod crypto;
mod manager;
mod password;
mod store;

struct AppState {
    password_manager: Arc<RwLock<Option<PasswordManager>>>,
}
// App 启动
// 加载配置
//      默认配置文件路径
//          不存在 --> 新建默认配置
//          存在   --> 读取并反序列化
fn main() {
    run_tauri_app();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            password_manager: Arc::new(RwLock::new(None)),
            // config: Arc::new(RwLock::new(Config::default())),
        })
        .invoke_handler(tauri::generate_handler![
            initialize_manager,
            add_password,
            delete_password,
            search_passwords,
            // search_passwords_in_storage,
            // get_all_passwords,
            get_all_passwords_from_storage,
            // get_password_by_id,
            // get_password_by_id_from_storage,
            decrypt_password,
            generate_password,
            // get_current_config,
            // save_config,
            // get_storage_status,
            // sync_storages
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
    // is_first_setup: bool,
    // has_encrypted_data: bool,
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
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<InitializeResult, ErrorInfo> {
    // Try to load existing config, or create default
    let mut config = {
        let config_path = Config::get_full_config_path(&app)
            .await
            .unwrap_or_else(|_| {
                // Fallback to relative path if Tauri path resolution fails
                Config::get_config_path()
            });
        Config::load_from_file(&config_path).unwrap_or_default()
    };

    // Resolve data path using Tauri's cross-platform path resolution
    if let Err(e) = config.resolve_data_path(&app).await {
        eprintln!("Warning: Failed to resolve data path: {}", e);
    }

    let password_manager = PasswordManager::new(config.clone())
        .await
        .map_err(ErrorInfo::from)?;

    // 更新状态
    *state.password_manager.write().await = Some(password_manager);
    // *state.config.write().await = config.clone();

    Ok(InitializeResult {
        // is_first_setup,
        // has_encrypted_data,
    })
}

#[tauri::command]
async fn add_password(
    request: PasswordCreateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<(), ErrorInfo> {
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

// #[tauri::command]
// async fn get_all_passwords(state: tauri::State<'_, AppState>) -> Result<Vec<Password>, ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     manager.get_all_passwords().await.map_err(ErrorInfo::from)
// }

// #[tauri::command]
// async fn get_password_by_id(
//     password_id: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<Option<Password>, ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     manager
//         .get_password_by_id(&password_id)
//         .await
//         .map_err(ErrorInfo::from)
// }

#[tauri::command]
async fn decrypt_password(
    password: EncryptedData,
    user_password: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, ErrorInfo> {
    let manager = state.password_manager.read().await;
    let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .decrypt_password(&user_password, &password)
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

// #[tauri::command]
// async fn get_current_config(state: tauri::State<'_, AppState>) -> Result<Config, ErrorInfo> {
//     let config = state.config.read().await;
//     Ok(config.clone())
// }
//
// #[tauri::command]
// async fn save_config(config: Config, state: tauri::State<'_, AppState>) -> Result<(), ErrorInfo> {
//     let config_path = Config::get_config_path();
//     config.save_to_file(&config_path).map_err(ErrorInfo::from)?;
//
//     *state.config.write().await = config;
//     Ok(())
// }

// #[tauri::command]
// async fn search_passwords_in_storage(
//     query: String,
//     storage_target: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<Vec<Password>, ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     let target = match storage_target.as_str() {
//         "local" => StorageTarget::Local,
//         "github" => StorageTarget::GitHub,
//         _ => {
//             return Err(ErrorInfo {
//                 code: 400,
//                 info: "Invalid storage target".to_string(),
//             });
//         }
//     };
//
//     manager
//         .search_passwords_in_storage(&query, target)
//         .await
//         .map_err(ErrorInfo::from)
// }

#[tauri::command]
async fn get_all_passwords_from_storage(
    storage_target: String,
    state: tauri::State<'_, AppState>,
) -> Result<StorageData, ErrorInfo> {
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
            });
        }
    };

    manager
        .get_all_passwords_from_storage(target)
        .await
        .map_err(ErrorInfo::from)
}

// #[tauri::command]
// async fn get_password_by_id_from_storage(
//     password_id: String,
//     storage_target: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<Option<Password>, ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     let target = match storage_target.as_str() {
//         "local" => StorageTarget::Local,
//         "github" => StorageTarget::GitHub,
//         _ => {
//             return Err(ErrorInfo {
//                 code: 400,
//                 info: "Invalid storage target".to_string(),
//             });
//         }
//     };
//
//     manager
//         .get_password_by_id_from_storage(&password_id, target)
//         .await
//         .map_err(ErrorInfo::from)
// }

// #[tauri::command]
// async fn get_storage_status(
//     state: tauri::State<'_, AppState>,
// ) -> Result<serde_json::Value, ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     let status = manager.get_storage_status().await;
//     let mut result = HashMap::new();
//
//     for (target, status) in status {
//         let key = match target {
//             StorageTarget::Local => "local",
//             StorageTarget::GitHub => "github",
//             StorageTarget::All => "all",
//         };
//         result.insert(key.to_string(), status);
//     }
//
//     Ok(serde_json::to_value(result).map_err(|e| ErrorInfo {
//         code: 500,
//         info: format!("Failed to serialize storage status: {}", e),
//     })?)
// }

// #[tauri::command]
// async fn sync_storages(
//     from_storage: String,
//     to_storage: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), ErrorInfo> {
//     let manager = state.password_manager.read().await;
//     let manager = manager.as_ref().ok_or_else(|| ErrorInfo {
//         code: 500,
//         info: "Password manager not initialized".to_string(),
//     })?;
//
//     let from_target = match from_storage.as_str() {
//         "local" => StorageTarget::Local,
//         "github" => StorageTarget::GitHub,
//         _ => {
//             return Err(ErrorInfo {
//                 code: 400,
//                 info: "Invalid from storage target".to_string(),
//             });
//         }
//     };
//
//     let to_target = match to_storage.as_str() {
//         "local" => StorageTarget::Local,
//         "github" => StorageTarget::GitHub,
//         _ => {
//             return Err(ErrorInfo {
//                 code: 400,
//                 info: "Invalid to storage target".to_string(),
//             });
//         }
//     };
//
//     manager
//         .sync_storages(from_target, to_target)
//         .await
//         .map_err(ErrorInfo::from)
// }
