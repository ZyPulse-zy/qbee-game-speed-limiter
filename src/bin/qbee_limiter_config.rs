#![windows_subsystem = "windows"]

#[path = "../common.rs"]
mod common;

use common::*;
use serde::Serialize;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    message: String,
    data: T,
}

fn main() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind config server");
    let port = listener.local_addr().map(|addr| addr.port()).unwrap_or(0);
    let url = format!("http://127.0.0.1:{port}/");
    open_browser(&url);

    listener.set_nonblocking(true).ok();
    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let shutdown = shutdown.clone();
                thread::spawn(move || handle_client(stream, shutdown));
            }
            Err(_) => thread::sleep(std::time::Duration::from_millis(60)),
        }
    }
}

fn open_browser(url: &str) {
    unsafe {
        ShellExecuteW(
            null_mut(),
            wide("open").as_ptr(),
            wide(url).as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
    }
}

fn handle_client(mut stream: TcpStream, shutdown: Arc<AtomicBool>) {
    let Ok((method, path, body)) = read_request(&mut stream) else {
        return;
    };
    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/") => html_response(APP_HTML),
        ("GET", "/api/config") => json_response(&ApiResponse {
            ok: true,
            message: "ok".into(),
            data: serde_json::json!({
                "config": load_config(),
                "status": load_status(),
            }),
        }),
        ("GET", "/api/status") => json_response(&ApiResponse {
            ok: true,
            message: "ok".into(),
            data: load_status(),
        }),
        ("POST", "/api/save") => match serde_json::from_slice::<AppConfig>(&body) {
            Ok(config) => {
                let save = save_config(&config);
                set_startup_enabled(config.start_with_windows);
                let start = if config.auto_start_monitor {
                    start_monitor_process_checked().map(|_| ())
                } else {
                    Ok(())
                };
                match save.and_then(|_| start.map_err(std::io::Error::other)) {
                    Ok(_) => json_response(&ApiResponse {
                        ok: true,
                        message: if config.auto_start_monitor {
                            "配置已保存，后台监控已启动。"
                        } else {
                            "配置已保存。"
                        }
                        .into(),
                        data: load_status(),
                    }),
                    Err(err) => error_response(&err.to_string()),
                }
            }
            Err(err) => error_response(&format!("配置格式无效：{err}")),
        },
        ("POST", "/api/start") => match serde_json::from_slice::<AppConfig>(&body) {
            Ok(config) => {
                let _ = save_config(&config);
                set_startup_enabled(config.start_with_windows);
                match start_monitor_process_checked() {
                    Ok(status) => json_response(&ApiResponse {
                        ok: true,
                        message: "后台监控已启动。".into(),
                        data: status,
                    }),
                    Err(err) => error_response(&err),
                }
            }
            Err(err) => error_response(&format!("配置格式无效：{err}")),
        },
        ("POST", "/api/stop") => {
            request_stop_monitor();
            json_response(&ApiResponse {
                ok: true,
                message: "已发送停止指令。".into(),
                data: load_status(),
            })
        }
        ("POST", "/api/test") => match serde_json::from_slice::<AppConfig>(&body) {
            Ok(config) => match test_connection(&config) {
                Ok(message) => json_response(&ApiResponse {
                    ok: true,
                    message,
                    data: serde_json::json!({}),
                }),
                Err(err) => error_response(&err),
            },
            Err(err) => error_response(&format!("配置格式无效：{err}")),
        },
        ("POST", "/api/diagnostics") => match serde_json::from_slice::<AppConfig>(&body) {
            Ok(config) => json_response(&ApiResponse {
                ok: true,
                message: "自检完成。".into(),
                data: run_diagnostics(&config),
            }),
            Err(err) => error_response(&format!("配置格式无效：{err}")),
        },
        ("POST", "/api/scan") => json_response(&ApiResponse {
            ok: true,
            message: "扫描完成。".into(),
            data: scan_game_libraries(),
        }),
        ("POST", "/api/desktop-shortcut") => match create_desktop_shortcut() {
            Ok(path) => json_response(&ApiResponse {
                ok: true,
                message: format!("桌面入口已创建：{}", path.display()),
                data: serde_json::json!({}),
            }),
            Err(err) => error_response(&err),
        },
        ("POST", "/api/quit") => {
            shutdown.store(true, Ordering::SeqCst);
            json_response(&ApiResponse {
                ok: true,
                message: "配置器已关闭。".into(),
                data: serde_json::json!({}),
            })
        }
        _ => not_found_response(),
    };
    let _ = stream.write_all(response.as_bytes());
}

fn read_request(stream: &mut TcpStream) -> Result<(String, String, Vec<u8>), String> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = stream.read(&mut chunk).map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 1024 * 1024 {
            return Err("request too large".into());
        }
    }
    let header_end = buffer
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|pos| pos + 4)
        .ok_or_else(|| "bad request".to_string())?;
    let header = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = header.lines();
    let first = lines.next().ok_or_else(|| "bad request".to_string())?;
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts
        .next()
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let content_length = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk).map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
    Ok((method, path, buffer[header_end..].to_vec()))
}

fn json_response<T: Serialize>(value: &T) -> String {
    let body = serde_json::to_string(value).unwrap();
    response("200 OK", "application/json; charset=utf-8", &body)
}

fn html_response(body: &str) -> String {
    response("200 OK", "text/html; charset=utf-8", body)
}

fn error_response(message: &str) -> String {
    json_response(&ApiResponse {
        ok: false,
        message: message.to_string(),
        data: serde_json::json!({}),
    })
}

fn not_found_response() -> String {
    response("404 Not Found", "text/plain; charset=utf-8", "Not found")
}

fn response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.as_bytes().len()
    )
}

const APP_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>qbee 游戏限速助手</title>
<style>
:root{color-scheme:dark;--bg:#08090a;--panel:#16171c;--panel2:#1e1f25;--line:rgba(255,255,255,.08);--text:#f7f8f8;--muted:#9ca3af;--dim:#6b7280;--accent:#5e6ad2;--ok:#32d583;--warn:#fdb022;--bad:#f97066}
*{box-sizing:border-box}body{margin:0;background:radial-gradient(circle at 25% -10%,rgba(94,106,210,.18),transparent 34%),var(--bg);color:var(--text);font:14px/1.55 Inter,"Segoe UI",system-ui,sans-serif}
.shell{max-width:1180px;margin:0 auto;padding:36px 28px 48px}.top{display:flex;align-items:flex-start;justify-content:space-between;gap:24px;margin-bottom:24px}.title h1{margin:0;font-size:30px;letter-spacing:-.02em}.title p{margin:6px 0 0;color:var(--muted)}
.status{min-width:360px;border:1px solid var(--line);background:rgba(255,255,255,.035);border-radius:14px;padding:14px 16px}.status b{display:block;font-size:13px}.status span{display:block;color:var(--muted);font-size:12px;margin-top:2px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.grid{display:grid;grid-template-columns:minmax(0,1.1fr) 340px;gap:18px}.card{border:1px solid var(--line);background:linear-gradient(180deg,rgba(255,255,255,.035),rgba(255,255,255,.02));border-radius:14px;padding:18px;transition:border-color .18s ease,transform .18s ease}.card:hover{border-color:rgba(255,255,255,.14);transform:translateY(-1px)}.card h2{font-size:15px;margin:0 0 16px}.field{display:grid;gap:6px;margin-bottom:14px}.field label{font-size:12px;color:var(--muted)}input,textarea,select{width:100%;border:1px solid var(--line);background:#0b0c10;color:var(--text);border-radius:9px;padding:10px 12px;outline:none;transition:border-color .16s ease,box-shadow .16s ease}input:focus,textarea:focus,select:focus{border-color:rgba(94,106,210,.65);box-shadow:0 0 0 3px rgba(94,106,210,.16)}textarea{min-height:168px;resize:vertical;font-family:"JetBrains Mono","Consolas",monospace;font-size:12px}
.row{display:grid;grid-template-columns:1fr 1fr;gap:12px}.checks{display:flex;gap:16px;flex-wrap:wrap;margin:8px 0 12px}.check{display:flex;align-items:center;gap:8px;color:var(--muted)}.check input{width:auto}
.actions{display:flex;gap:10px;flex-wrap:wrap;margin-top:16px}.btn{border:1px solid var(--line);background:var(--panel2);color:var(--text);border-radius:9px;padding:10px 14px;cursor:pointer;transition:transform .14s ease,background .14s ease,border-color .14s ease}.btn:hover{transform:translateY(-1px);border-color:rgba(255,255,255,.18)}.btn:active{transform:translateY(0)}.btn.primary{background:var(--accent);border-color:rgba(255,255,255,.12)}.btn.good{background:rgba(50,213,131,.12);border-color:rgba(50,213,131,.35)}.btn:disabled{opacity:.55;cursor:not-allowed}
.pill{display:inline-flex;align-items:center;gap:8px;border:1px solid var(--line);background:var(--panel2);border-radius:999px;padding:6px 10px;color:var(--muted);font-size:12px}.dot{width:8px;height:8px;border-radius:50%;background:var(--dim)}.dot.on{background:var(--ok)}.dot.busy{background:var(--warn)}.dot.bad{background:var(--bad)}.dot.busy{animation:pulse 1s infinite}@keyframes pulse{0%,100%{box-shadow:0 0 0 0 rgba(253,176,34,.4)}50%{box-shadow:0 0 0 6px rgba(253,176,34,0)}}.hint{margin:-6px 0 14px;color:var(--muted);font-size:12px}
.list{display:grid;gap:8px;max-height:260px;overflow:auto}.item{display:flex;justify-content:space-between;gap:10px;align-items:center;border:1px solid var(--line);border-radius:9px;background:#0b0c10;padding:9px 10px}.item code{font-family:"JetBrains Mono","Consolas",monospace;font-size:12px;color:#d8dcff;overflow:hidden;text-overflow:ellipsis}.item button{border:0;background:transparent;color:var(--muted);cursor:pointer}
.log{font-family:"JetBrains Mono","Consolas",monospace;font-size:12px;color:var(--muted);white-space:pre-wrap;min-height:80px}.footer{margin-top:18px;color:var(--dim);font-size:12px}
@media(max-width:900px){.grid{grid-template-columns:1fr}.top{display:block}.status{min-width:0;margin-top:16px}}
</style>
</head>
<body>
<main class="shell">
  <section class="top">
    <div class="title"><h1>qbee 游戏限速助手</h1><p>配置界面只在需要时打开，后台监控程序独立低占用运行。</p></div>
    <div class="status"><b id="headline">正在读取状态</b><span id="subline">请稍候...</span></div>
  </section>
  <section class="grid">
    <div class="card">
      <h2>连接设置</h2>
      <div class="field"><label>下载客户端</label><select id="client" onchange="updateClientHelp()"><option value="qbittorrent">qBittorrent / qBittorrent EE</option><option value="transmission">Transmission</option><option value="aria2">aria2 / Motrix</option><option value="utorrent">µTorrent / BitTorrent Classic</option><option value="deluge">Deluge</option><option value="bitcomet">BitComet / 比特彗星</option></select></div>
      <p class="hint" id="clientHelp">qB 使用备用速度限制开关，适合 qBittorrent 与 qBittorrent Enhanced Edition。</p>
      <div class="field"><label>qB Web UI 地址</label><input id="qbee_url"></div>
      <div class="field"><label>Transmission RPC 地址</label><input id="transmission_url"></div>
      <div class="field"><label>aria2 JSON-RPC 地址</label><input id="aria2_url"></div>
      <div class="field"><label>µTorrent / BitTorrent Web UI 地址</label><input id="utorrent_url"></div>
      <div class="field"><label>Deluge Web JSON 地址</label><input id="deluge_url"></div>
      <div class="row">
        <div class="field"><label>用户名</label><input id="username"></div>
        <div class="field"><label>密码</label><input id="password" type="password"></div>
      </div>
      <div class="row">
        <div class="field"><label>aria2 Secret（可选）</label><input id="aria2_secret" type="password"></div>
        <div class="field"><label>检测间隔（秒）</label><input id="interval" type="number" min="1"></div>
      </div>
      <div class="row">
        <div class="field"><label>游戏中下载限速（KiB/s）</label><input id="download_limit" type="number" min="1"></div>
        <div class="field"><label>游戏中上传限速（KiB/s）</label><input id="upload_limit" type="number" min="1"></div>
      </div>
      <div class="field"><label>手动进程名（逗号分隔，可选）</label><input id="processes"></div>
      <div class="checks">
        <label class="check"><input id="startup" type="checkbox">开机启动后台监控</label>
        <label class="check"><input id="autostart" type="checkbox">保存后自动启动监控</label>
      </div>
      <div class="actions">
        <button class="btn" onclick="testConnection()">测试连接</button><button class="btn" onclick="runDiagnostics()">运行自检</button>
        <button class="btn primary" onclick="saveConfig()">保存并应用</button>
        <button class="btn good" onclick="startMonitor()">启动监控</button>
        <button class="btn" onclick="stopMonitor()">停止监控</button>
        <button class="btn" onclick="createShortcut()">创建桌面入口</button><button class="btn" onclick="quitApp()">关闭配置器</button>
      </div>
    </div>
    <aside class="card">
      <h2>后台状态</h2>
      <p class="pill"><i id="dot" class="dot"></i><span id="state">未知</span></p>
      <div class="field" style="margin-top:16px"><label>当前检测到</label><textarea id="detected" readonly></textarea></div>
      <div class="log" id="log">等待操作...</div>
    </aside>
    <div class="card" style="grid-column:1/-1">
      <h2>游戏库文件夹</h2>
      <div class="actions" style="margin-top:0;margin-bottom:12px">
        <button class="btn" onclick="scanLibraries()">自动扫描</button>
        <button class="btn" onclick="addFolder()">添加当前输入</button>
      </div>
      <div class="field"><label>新增文件夹路径</label><input id="folderInput" placeholder="例如 D:\SteamLibrary\steamapps"></div>
      <div class="list" id="folders"></div>
    </div>
  </section>
  <div class="footer">配置会保存在本目录的 qbee_game_speed_limiter.json。保存配置后，后台监控程序会按你的设置自动启动。</div>
</main>
<script>
let config = null;
const $ = id => document.getElementById(id);
function log(text){ $('log').textContent = text; }
function setBusy(text){ $('headline').textContent = text; $('subline').textContent = '操作进行中'; $('dot').className='dot busy'; }
function updateClientHelp(){
  const value = $('client').value;
  const messages = {
    qbittorrent:'qB 使用备用速度限制开关，适合 qBittorrent 与 qBittorrent Enhanced Edition。',
    transmission:'Transmission 使用 RPC 的 alt-speed-enabled 开关，需要在 Transmission 中先配置好备用限速值。',
    aria2:'aria2 / Motrix 没有备用限速开关，本工具会在游戏中临时切换全局上下行限速，退出后恢复原值。',
    utorrent:'µTorrent / BitTorrent Classic 使用 Web UI API 临时修改全局上下行限速，退出游戏后恢复原值。',
    deluge:'Deluge 使用 Web JSON-RPC 临时修改全局上下行限速，退出游戏后恢复原值。Deluge Web 通常只需要密码。',
    bitcomet:'BitComet / 比特彗星已加入列表，但当前缺少稳定公开的远程限速 API，本版会明确提示而不会假装自动控制。'
  };
  $('clientHelp').textContent = messages[value] || messages.qbittorrent;
}
function configFromForm(){
  return {
    ...config,
    download_client:$('client').value,
    qbee_url:$('qbee_url').value.trim(),
    transmission_url:$('transmission_url').value.trim(),
    aria2_url:$('aria2_url').value.trim(),
    utorrent_url:$('utorrent_url').value.trim(),
    deluge_url:$('deluge_url').value.trim(),
    username:$('username').value,
    password:$('password').value,
    aria2_secret:$('aria2_secret').value,
    game_download_limit_kib:Math.max(1, Number($('download_limit').value || 512)),
    game_upload_limit_kib:Math.max(1, Number($('upload_limit').value || 128)),
    check_interval_seconds:Math.max(1, Number($('interval').value || 5)),
    game_processes:$('processes').value.split(',').map(s=>s.trim()).filter(Boolean),
    start_with_windows:$('startup').checked,
    auto_start_monitor:$('autostart').checked,
    game_folders:[...document.querySelectorAll('[data-folder]')].map(el=>el.dataset.folder)
  };
}
function renderFolders(items){
  $('folders').innerHTML = '';
  items.forEach(folder => {
    const row = document.createElement('div');
    row.className = 'item';
    row.dataset.folder = folder;
    row.innerHTML = `<code title="${folder}">${folder}</code><button>移除</button>`;
    row.querySelector('button').onclick = () => row.remove();
    $('folders').appendChild(row);
  });
}
function fillForm(data){
  config = data.config;
  $('client').value = config.download_client || 'qbittorrent';
  $('qbee_url').value = config.qbee_url || '';
  $('transmission_url').value = config.transmission_url || 'http://127.0.0.1:9091/transmission/rpc';
  $('aria2_url').value = config.aria2_url || 'http://127.0.0.1:6800/jsonrpc';
  $('utorrent_url').value = config.utorrent_url || 'http://127.0.0.1:8080/gui';
  $('deluge_url').value = config.deluge_url || 'http://127.0.0.1:8112/json';
  $('username').value = config.username || '';
  $('password').value = config.password || '';
  $('aria2_secret').value = config.aria2_secret || '';
  $('interval').value = config.check_interval_seconds || 5;
  $('download_limit').value = config.game_download_limit_kib || 512;
  $('upload_limit').value = config.game_upload_limit_kib || 128;
  $('processes').value = (config.game_processes || []).join(', ');
  $('startup').checked = !!config.start_with_windows;
  $('autostart').checked = !!config.auto_start_monitor;
  renderFolders(config.game_folders || []);
  updateClientHelp();
  renderStatus(data.status);
}
function renderStatus(status){
  const stale = !status.updated_at || Date.now()/1000 - status.updated_at > 20;
  $('headline').textContent = stale ? '后台监控未响应' : status.message;
  $('subline').textContent = status.running ? '后台监控正在运行' : '后台监控未运行';
  $('state').textContent = status.running ? (status.game_running ? '游戏运行中' : '监控中') : '已停止';
  $('detected').value = status.detected || '无';
  $('dot').className = 'dot ' + (status.running ? 'on' : '');
}
async function api(path, body){
  const res = await fetch(path, {method: body ? 'POST':'GET', body: body ? JSON.stringify(body): undefined});
  const json = await res.json();
  if(!json.ok) throw new Error(json.message);
  return json;
}
async function load(){
  try{ const json = await api('/api/config'); fillForm(json.data); }
  catch(e){ log('读取失败：' + e.message); }
}
async function saveConfig(){
  setBusy('正在保存配置');
  try{ const json = await api('/api/save', configFromForm()); config = configFromForm(); log(json.message); await refreshStatus(); }
  catch(e){ $('dot').className='dot bad'; log('保存失败：' + e.message); }
}
async function testConnection(){
  setBusy('正在测试连接');
  try{ const json = await api('/api/test', configFromForm()); log(json.message); await refreshStatus(); }
  catch(e){ $('dot').className='dot bad'; log('连接失败：' + e.message); }
}
async function runDiagnostics(){
  setBusy('正在运行自检');
  try{
    const json = await api('/api/diagnostics', configFromForm());
    const icon = {ok:'✓', warn:'!', error:'×', info:'i'};
    const lines = json.data.map(item => `${icon[item.level] || '-'} ${item.title}: ${item.message}`);
    log(lines.join('\n'));
    await refreshStatus();
  }catch(e){ $('dot').className='dot bad'; log('自检失败：' + e.message); }
}
async function scanLibraries(){
  setBusy('正在扫描游戏库');
  try{ const json = await api('/api/scan', {}); const folders = new Set([...document.querySelectorAll('[data-folder]')].map(el=>el.dataset.folder)); json.data.forEach(v=>folders.add(v)); renderFolders([...folders]); log(`扫描完成，找到 ${json.data.length} 个候选目录。`); await refreshStatus(); }
  catch(e){ $('dot').className='dot bad'; log('扫描失败：' + e.message); }
}
function addFolder(){ const v=$('folderInput').value.trim(); if(!v) return; const folders = new Set([...document.querySelectorAll('[data-folder]')].map(el=>el.dataset.folder)); folders.add(v); renderFolders([...folders]); $('folderInput').value=''; }
async function startMonitor(){ setBusy('正在启动监控'); try{ const json = await api('/api/start', configFromForm()); log(json.message); renderStatus(json.data); }catch(e){ $('headline').textContent='启动监控失败'; $('subline').textContent='请查看右侧错误信息'; $('state').textContent='启动失败'; $('dot').className='dot bad'; log('启动失败：' + e.message); } }
async function stopMonitor(){ setBusy('正在停止监控'); try{ const json = await api('/api/stop', {}); log(json.message); setTimeout(refreshStatus, 1200); }catch(e){ $('dot').className='dot bad'; log('停止失败：' + e.message); } }
async function createShortcut(){ setBusy('正在创建桌面入口'); try{ const json = await api('/api/desktop-shortcut', {}); log(json.message); await refreshStatus(); }catch(e){ $('dot').className='dot bad'; log('创建失败：' + e.message); } }
async function quitApp(){ try{ await api('/api/quit', {}); window.close(); document.body.innerHTML='<main class="shell"><h1>配置器已关闭</h1><p>可以关闭这个标签页。</p></main>'; }catch(e){} }
async function refreshStatus(){ try{ const json = await api('/api/status'); renderStatus(json.data); }catch(e){} }
load(); setInterval(refreshStatus, 3000);
</script>
</body>
</html>"#;
