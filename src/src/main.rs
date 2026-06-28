#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// 桌面端入口点
fn main() {
  aischat_client_lib::run();
}
