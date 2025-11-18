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

// info宏 仅在debug模式下打印
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("prefix: {}", format_args!($($arg)*));
    };
}

// error宏 仅在debug模式下打印
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("prefix: {}", format_args!($($arg)*));
    };
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
            get_all_passwords_from_storage,
            decrypt_password,
            generate_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct AppState {
    password_manager: Arc<RwLock<Option<PasswordManager>>>,
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

        info!("配置路径：{}", config_path.to_str().unwrap_or("空"));

        Config::load_from_file(&config_path).unwrap_or_default()
    };

    info!("配置：\n{:?}", &config);

    // Resolve data path using Tauri's cross-platform path resolution
    if let Err(e) = config.resolve_data_path(&app).await {
        error!("解析 data path 失败: {}", e);

        return Err(ErrorInfo {
            code: 500,
            info: e.to_string(),
        });
    }

    let password_manager = PasswordManager::new(config.clone())
        .await
        .map_err(ErrorInfo::from)?;

    info!("密码管理器初始化完成");

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
    info!("添加密码请求：{:?}", &request);

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
