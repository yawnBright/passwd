// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// App 启动
// 加载配置
//      默认配置文件路径
//          不存在 --> 新建默认配置
//          存在   --> 读取并反序列化
fn main() {
    passwd_lib::run_tauri_app();
}
