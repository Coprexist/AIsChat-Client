use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct IOBridgeRequest {
  target: serde_json::Value,
  method: String,
  args: Option<Vec<serde_json::Value>>,
}

#[tauri::command]
fn io_bridge_call(request: IOBridgeRequest) -> Result<serde_json::Value, String> {
  let method = request.method;
  let args = request.args.unwrap_or_default();
  log::info!("IO Bridge call: {} with {} args", method, args.len());
  Ok(serde_json::json!({
    "result": "ok",
    "method": method,
    "args": args
  }))
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      let _window = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
      )
      .title("AIsChat")
      .inner_size(1200.0, 800.0)
      .min_inner_size(800.0, 600.0)
      .resizable(true)
      .initialization_script(ADAPTATION_SCRIPT)
      .build()?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![io_bridge_call])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}