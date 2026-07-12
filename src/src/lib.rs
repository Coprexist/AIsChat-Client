use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct IOBridgeRequest {
  target: serde_json::Value,
  method: String,
  args: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SafeAreaInsets {
  top: f64,
  bottom: f64,
  left: f64,
  right: f64,
}

/// 通用桥接命令：按 method 分发到不同逻辑
#[tauri::command]
fn io_bridge_call(request: IOBridgeRequest) -> Result<serde_json::Value, String> {
  let method = request.method;
  let _args = request.args.unwrap_or_default();

  log::info!("IO Bridge call: {}", method);

  match method.as_str() {
    "getPlatform" => {
      #[cfg(target_os = "android")]
      return Ok(serde_json::json!({ "platform": "android" }));
      #[cfg(target_os = "ios")]
      return Ok(serde_json::json!({ "platform": "ios" }));
      #[cfg(not(any(target_os = "android", target_os = "ios")))]
      return Ok(serde_json::json!({ "platform": "desktop" }));
    }
    "getAppVersion" => {
      Ok(serde_json::json!({ "version": "0.0.1" }))
    }
    _ => {
      log::warn!("IO Bridge: unknown method {}", method);
      Ok(serde_json::json!({
        "result": "ok",
        "method": method,
        "args": _args,
        "note": "unimplemented method"
      }))
    }
  }
}

/// 获取状态栏高度（Android 通过 CSS env(safe-area-inset-top) 更可靠）
#[tauri::command]
fn get_status_bar_height() -> f64 {
  0.0
}

/// 获取安全区域插值
#[tauri::command]
fn get_safe_area_insets() -> Result<SafeAreaInsets, String> {
  Ok(SafeAreaInsets {
    top: 0.0,
    bottom: 0.0,
    left: 0.0,
    right: 0.0,
  })
}

/// 移动端注入脚本：适配键盘弹出 + 安全区域
const ADAPTATION_SCRIPT: &str = r#"(function(){
if(window.__aischatLoaded)return;
window.__aischatLoaded=true;
var m=document.querySelector('meta[name=viewport]');
if(!m){m=document.createElement('meta');m.name='viewport';document.head.appendChild(m);}
m.content='width=device-width,initial-scale=1.0,maximum-scale=1.0,user-scalable=no,viewport-fit=cover';
var s=document.createElement('style');
s.textContent='body{padding-top:env(safe-area-inset-top,0px)!important;padding-bottom:env(safe-area-inset-bottom,0px)!important}';
document.head.appendChild(s);
var fullHeight=window.innerHeight;
function onResize(){
if(window.innerHeight>fullHeight)fullHeight=window.innerHeight;
var kh=fullHeight-window.innerHeight;
if(kh>50)document.body.style.paddingBottom=kh+'px';
else document.body.style.paddingBottom='';
}
window.addEventListener('resize',onResize);
if(window.visualViewport)window.visualViewport.addEventListener('resize',onResize);
})();
"#;

// Tauri 2.0: 移动端入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      log::info!("AIsChat Client starting...");
      let _window = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
      )
      .title("AIsChat")
      .inner_size(1200.0, 800.0)
      .min_inner_size(800.0, 600.0)
      .resizable(true)
      .fullscreen(false)
      .center()
      .initialization_script(ADAPTATION_SCRIPT)
      .build()?;
      Ok(())
    })
    .on_navigation(|url| {
      let allowed = url.scheme() == "https"
        || url.scheme() == "http"
        || url.scheme() == "tauri";
      if !allowed {
        log::warn!("Blocked navigation to: {}", url);
      }
      allowed
    })
    .invoke_handler(tauri::generate_handler![
      io_bridge_call,
      get_status_bar_height,
      get_safe_area_insets,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
