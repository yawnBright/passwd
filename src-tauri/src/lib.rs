mod config;
mod crypto;
mod log;
mod manager;
mod password;
mod store;

use config::Config;
use crypto::EncryptedData;
use manager::PasswordManager;
use password::{Password, PasswordCreateRequest, PasswordGeneratorConfig};
use std::path::PathBuf;
use std::sync::OnceLock;
use store::StorageData;
use store::StorageTarget;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            password_manager: OnceLock::new(),
            // config: Arc::new(RwLock::new(Config::default())),
        })
        .setup(|app| {
            init(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize_manager,
            add_password,
            delete_password,
            search_passwords,
            get_all_passwords_from_storage,
            decrypt_password,
            generate_password,
            update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

static CONF_PATH: OnceLock<PathBuf> = OnceLock::new();
static DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

fn init(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let conf_path = Config::get_config_path(app)?;

    CONF_PATH
        .set(conf_path)
        .map_err(|_| anyhow::anyhow!("CONF_PATH已初始化"))?;

    let data_path = Config::get_data_path(app)?;
    DATA_PATH
        .set(data_path)
        .map_err(|_| anyhow::anyhow!("DATA_PATH已初始化"))?;

    info!(
        "**配置路径**：{}",
        CONF_PATH.get().unwrap().to_str().unwrap_or("空")
    );

    info!(
        "**数据路径**：{}",
        DATA_PATH.get().unwrap().to_str().unwrap_or("空")
    );

    Ok(())
}

// 为什么这里需要一个OnceLock呢
// 因为password_manager这个变量需要延迟初始化
// 或至少等到app实例创建之后才能初始化
//
// 可以在setup里面初始化，但是这个初始化又是个异步的
// 后面可以考虑使用同步块来解决
//
// 或者使用unsafe代码
struct AppState {
    password_manager: OnceLock<PasswordManager>,
}

#[derive(serde::Serialize)]
struct ErrorInfo {
    code: isize,
    info: String,
}

#[derive(serde::Serialize)]
struct InitializeResult {
    is_first_setup: bool,
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
    state: tauri::State<'_, AppState>,
) -> Result<InitializeResult, ErrorInfo> {
    let conf_path = CONF_PATH.get().expect("[内部错误] sys init error");

    let mut config = Config::default();

    if conf_path.exists() {
        info!("配置文件存在，加载配置");
        config = Config::load_from_file(conf_path)?;
    } else {
        info!("配置文件不存在，创建默认配置");
        config.save_to_file(conf_path)?;
    }

    info!("配置：{:?}", &config);

    let is_first_setup = config.is_first_setup;

    let password_manager = PasswordManager::new(config).await?;

    info!("密码管理器初始化完成");

    // let is_first_setup = password_manager
    //     .get_config_ref()
    //     .read()
    //     .await
    //     .is_first_setup;

    // 更新状态
    if state.password_manager.set(password_manager).is_err() {
        panic!("[内部错误] sys init error");
    }

    Ok(InitializeResult { is_first_setup })
}

#[tauri::command]
async fn add_password(
    request: PasswordCreateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<(), ErrorInfo> {
    info!("添加密码请求：{:?}", &request);

    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
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

// 更新配置
#[tauri::command]
async fn update_config(
    new_config: Config,
    state: tauri::State<'_, AppState>,
) -> Result<(), ErrorInfo> {
    let manager = state.password_manager.get().ok_or_else(|| ErrorInfo {
        code: 500,
        info: "Password manager not initialized".to_string(),
    })?;

    manager
        .update_config(new_config)
        .await
        .map_err(ErrorInfo::from)
}
