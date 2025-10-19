// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_fs::FsExt;

mod config;
mod crypto;
mod github_client;
mod github_store;
mod manager;
mod password;
mod store;

fn main() {
    init_password_manager();
    run_tauri_app();
}

fn init_password_manager() {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![add_password])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize)]
struct ErrorInfo {
    code: isize,
    info: String,
}

#[tauri::command]
fn add_password(pswd: password::Password) -> Result<(), ErrorInfo> {
    Ok(())
}

#[tauri::command]
fn delete_password(pswd: password::Password) -> Result<(), ErrorInfo> {
    Ok(())
}

#[tauri::command]
fn search_password() -> Result<Vec<password::Password>, ErrorInfo> {
    Ok(vec![])
}

#[tauri::command]
fn get_current_config() -> Result<config::Config, ErrorInfo> {
    Ok(())
}

#[tauri::command]
fn save_config() -> Result<(), ErrorInfo> {
    Ok(())
}
