#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::c_void;
use std::fs;
use std::io::Write;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const APP_TITLE: &str = "qbee 游戏限速助手";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "QbeeGameSpeedLimiter";
const WM_STATUS: u32 = WM_APP + 1;
const WM_DETECTED: u32 = WM_APP + 2;
const WM_STOPPED: u32 = WM_APP + 3;
const WM_SCAN_DONE: u32 = WM_APP + 4;

const ID_URL: isize = 1001;
const ID_USER: isize = 1002;
const ID_PASSWORD: isize = 1003;
const ID_INTERVAL: isize = 1004;
const ID_STARTUP: isize = 1005;
const ID_AUTOSTART: isize = 1006;
const ID_FOLDERS: isize = 1007;
const ID_TEST: isize = 1008;
const ID_SCAN: isize = 1009;
const ID_ADD: isize = 1010;
const ID_REMOVE: isize = 1011;
const ID_OPEN_CONFIG: isize = 1012;
const ID_SAVE: isize = 1013;
const ID_START: isize = 1014;
const ID_STOP: isize = 1015;

const COLOR_BG: u32 = rgb(8, 10, 15);
const COLOR_CARD: u32 = rgb(17, 19, 28);
const COLOR_INPUT: u32 = rgb(11, 13, 20);
const COLOR_BORDER: u32 = rgb(39, 45, 61);
const COLOR_TEXT: u32 = rgb(229, 231, 235);
const COLOR_MUTED: u32 = rgb(156, 163, 175);
const COLOR_PRIMARY: u32 = rgb(99, 102, 241);
const COLOR_PRIMARY_HOT: u32 = rgb(129, 140, 248);
const COLOR_BUTTON: u32 = rgb(28, 32, 44);
const COLOR_BUTTON_DISABLED: u32 = rgb(31, 35, 46);
const UI_WIDTH: i32 = 960;
const UI_HEIGHT: i32 = 740;

#[derive(Clone, Serialize, Deserialize)]
struct AppConfig {
    qbee_url: String,
    username: String,
    password: String,
    game_folders: Vec<String>,
    game_processes: Vec<String>,
    exclude_processes: Vec<String>,
    exclude_path_keywords: Vec<String>,
    exclude_steam_app_keywords: Vec<String>,
    check_interval_seconds: u64,
    restore_on_exit: bool,
    start_with_windows: bool,
    auto_start_monitor: bool,
    log_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            qbee_url: "http://127.0.0.1:8080".into(),
            username: "admin".into(),
            password: String::new(),
            game_folders: vec![
                r"C:\Program Files (x86)\Steam\steamapps".into(),
                r"D:\SteamLibrary\steamapps".into(),
            ],
            game_processes: vec![],
            exclude_processes: vec![
                "steam.exe".into(),
                "steamservice.exe".into(),
                "steamwebhelper.exe".into(),
                "wallpaper32.exe".into(),
                "wallpaper64.exe".into(),
                "wallpaper_engine.exe".into(),
                "epicgameslauncher.exe".into(),
                "goggalaxy.exe".into(),
                "wegame.exe".into(),
                "battle.net.exe".into(),
            ],
            exclude_path_keywords: vec![
                r"\steamapps\common\steamworks shared\".into(),
                r"\steamapps\common\proton ".into(),
                r"\steamapps\common\steam linux runtime".into(),
                r"\_commonredist\".into(),
                r"\redist\".into(),
                r"\redistributable\".into(),
                r"\installer\".into(),
                r"\uninstall\".into(),
                r"\launcher\".into(),
                r"\wallpaper_engine\".into(),
            ],
            exclude_steam_app_keywords: vec![
                "wallpaper".into(),
                "dedicated server".into(),
                "server tool".into(),
                "server dedicated".into(),
                "sdk".into(),
                "tool".into(),
                "tools".into(),
                "benchmark".into(),
                "editor".into(),
                "modding".into(),
                "workshop".into(),
                "proton".into(),
                "redistributable".into(),
                "runtime".into(),
            ],
            check_interval_seconds: 5,
            restore_on_exit: true,
            start_with_windows: false,
            auto_start_monitor: false,
            log_file: "qbee_game_speed_limiter.log".into(),
        }
    }
}

struct Handles {
    hwnd: HWND,
    url: HWND,
    user: HWND,
    password: HWND,
    interval: HWND,
    startup: HWND,
    autostart: HWND,
    folders: HWND,
    status: HWND,
    detected: HWND,
    save: HWND,
    start: HWND,
    stop: HWND,
    font: HFONT,
    title_font: HFONT,
    bg: HBRUSH,
    card: HBRUSH,
    input: HBRUSH,
    button: HBRUSH,
    primary: HBRUSH,
    disabled: HBRUSH,
}

struct App {
    config: AppConfig,
    handles: Handles,
    monitor: Monitor,
    closing_after_stop: bool,
}

#[derive(Clone)]
struct Monitor {
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            stopping: Arc::new(AtomicBool::new(false)),
        }
    }
}

thread_local! {
    static APP: RefCell<Option<Rc<RefCell<App>>>> = const { RefCell::new(None) };
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn from_wide(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

fn lower(value: &str) -> String {
    value.to_lowercase()
}

fn app_dir() -> PathBuf {
    let mut buffer = vec![0u16; 32768];
    let len = unsafe { GetModuleFileNameW(null_mut(), buffer.as_mut_ptr(), buffer.len() as u32) }
        as usize;
    PathBuf::from(from_wide(&buffer[..len]))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn config_path() -> PathBuf {
    app_dir().join("qbee_game_speed_limiter.json")
}

fn load_config() -> AppConfig {
    let path = config_path();
    match fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<AppConfig>(&text).ok())
    {
        Some(mut config) => {
            if config.qbee_url.trim().is_empty() {
                config.qbee_url = AppConfig::default().qbee_url;
            }
            if config.check_interval_seconds == 0 {
                config.check_interval_seconds = 5;
            }
            config
        }
        None => {
            let config = AppConfig::default();
            let _ = save_config(&config);
            config
        }
    }
}

fn save_config(config: &AppConfig) -> std::io::Result<()> {
    fs::write(config_path(), serde_json::to_string_pretty(config).unwrap())
}

fn log_line(config: &AppConfig, message: &str) {
    let path = app_dir().join(if config.log_file.is_empty() {
        "qbee_game_speed_limiter.log"
    } else {
        &config.log_file
    });
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", message);
    }
}

fn get_text(hwnd: HWND) -> String {
    let len = unsafe { GetWindowTextLengthW(hwnd) } as usize;
    let mut buffer = vec![0u16; len + 1];
    unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    from_wide(&buffer)
}

fn set_text(hwnd: HWND, text: &str) {
    unsafe {
        SetWindowTextW(hwnd, wide(text).as_ptr());
    }
}

fn set_status(handles: &Handles, text: &str) {
    set_text(handles.status, &format!("状态：{text}"));
}

fn post_string(hwnd: HWND, msg: u32, text: String) {
    unsafe {
        PostMessageW(hwnd, msg, 0, Box::into_raw(Box::new(text)) as isize);
    }
}

fn normalize_path(value: &str) -> String {
    fs::canonicalize(value)
        .unwrap_or_else(|_| PathBuf::from(value))
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

fn is_under_folder(file: &str, folder: &str) -> bool {
    let file = normalize_path(file);
    let folder = normalize_path(folder);
    file == folder || file.starts_with(&(folder + "\\"))
}

struct QbeeClient {
    base_url: String,
    username: String,
    password: String,
    agent: ureq::Agent,
    cookie: Option<String>,
    logged_in: bool,
}

impl QbeeClient {
    fn new(config: &AppConfig) -> Self {
        Self {
            base_url: config.qbee_url.trim_end_matches('/').to_string(),
            username: config.username.clone(),
            password: config.password.clone(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            cookie: None,
            logged_in: false,
        }
    }

    fn speed_limits_enabled(&mut self) -> Result<bool, String> {
        self.ensure_login()?;
        Ok(self
            .request("GET", "/api/v2/transfer/speedLimitsMode", None)?
            .trim()
            == "1")
    }

    fn set_speed_limits(&mut self, enabled: bool) -> Result<bool, String> {
        let current = self.speed_limits_enabled()?;
        if current == enabled {
            return Ok(false);
        }
        self.request("POST", "/api/v2/transfer/toggleSpeedLimitsMode", Some(""))?;
        Ok(true)
    }

    fn ensure_login(&mut self) -> Result<(), String> {
        if self.logged_in {
            return Ok(());
        }
        self.validate_server()?;
        if self.request("GET", "/api/v2/app/version", None).is_ok() {
            self.logged_in = true;
            return Ok(());
        }
        let body = format!(
            "username={}&password={}",
            url_encode(&self.username),
            url_encode(&self.password)
        );
        let result = self.request("POST", "/api/v2/auth/login", Some(&body))?;
        if result.trim() != "Ok." {
            return Err("qbee 登录失败，请检查 Web UI 用户名和密码。".into());
        }
        self.logged_in = true;
        Ok(())
    }

    fn validate_server(&mut self) -> Result<(), String> {
        let root = self.request("GET", "/", None)?;
        if root.contains("CEF remote debugging") && !self.try_ipv6_loopback() {
            return Err("当前地址打开的是 CEF remote debugging，不是 qBittorrent Web UI。请尝试 http://[::1]:8080 或更换 qB Web UI 端口。".into());
        }
        Ok(())
    }

    fn try_ipv6_loopback(&mut self) -> bool {
        if !(self.base_url.contains("127.0.0.1") || self.base_url.contains("localhost")) {
            return false;
        }
        let original = self.base_url.clone();
        self.base_url = self
            .base_url
            .replace("127.0.0.1", "[::1]")
            .replace("localhost", "[::1]");
        if self.request("GET", "/api/v2/app/version", None).is_ok() {
            true
        } else {
            self.base_url = original;
            false
        }
    }

    fn request(&mut self, method: &str, path: &str, body: Option<&str>) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = match method {
            "POST" => self.agent.post(&url),
            _ => self.agent.get(&url),
        }
        .set("User-Agent", "qbee-game-speed-limiter/5.0")
        .set("Referer", &self.base_url);

        if let Some(cookie) = &self.cookie {
            req = req.set("Cookie", cookie);
        }

        let response = match body {
            Some(body) => req
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_string(body),
            None => req.call(),
        }
        .map_err(|err| err.to_string())?;

        if let Some(cookie) = response.header("set-cookie") {
            self.cookie = cookie.split(';').next().map(str::to_string);
        }
        response.into_string().map_err(|err| err.to_string())
    }
}

fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            byte => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

struct GameDetector {
    folders: Vec<String>,
    process_names: HashSet<String>,
    excluded_names: HashSet<String>,
    excluded_path_keywords: Vec<String>,
    path_cache: HashMap<u32, (String, String)>,
}

impl GameDetector {
    fn new(config: &AppConfig) -> Self {
        Self {
            folders: build_detection_folders(config),
            process_names: config.game_processes.iter().map(|v| lower(v)).collect(),
            excluded_names: config.exclude_processes.iter().map(|v| lower(v)).collect(),
            excluded_path_keywords: config
                .exclude_path_keywords
                .iter()
                .map(|v| lower(v))
                .collect(),
            path_cache: HashMap::new(),
        }
    }

    fn detect(&mut self) -> Option<String> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return None;
            }
            let mut entry: PROCESSENTRY32W = zeroed();
            entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
            let mut live = HashSet::new();
            let mut result = None;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    live.insert(pid);
                    let exe = lower(&from_wide(&entry.szExeFile));
                    let stem = exe.strip_suffix(".exe").unwrap_or(&exe).to_string();
                    if self.excluded_names.contains(&exe) || self.excluded_names.contains(&stem) {
                        if Process32NextW(snapshot, &mut entry) == 0 {
                            break;
                        }
                        continue;
                    }
                    if self.process_names.contains(&exe) || self.process_names.contains(&stem) {
                        result = Some(exe);
                        break;
                    }
                    if let Some(path) = self.process_path(pid, &exe) {
                        let normalized = normalize_path(&path);
                        if self
                            .excluded_path_keywords
                            .iter()
                            .any(|keyword| normalized.contains(keyword))
                        {
                            if Process32NextW(snapshot, &mut entry) == 0 {
                                break;
                            }
                            continue;
                        }
                        if self
                            .folders
                            .iter()
                            .any(|folder| is_under_folder(&path, folder))
                        {
                            result = Some(path);
                            break;
                        }
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
            if self.path_cache.len() > 256 {
                self.path_cache.retain(|pid, _| live.contains(pid));
            }
            result
        }
    }

    unsafe fn process_path(&mut self, pid: u32, exe: &str) -> Option<String> {
        if let Some((cached_exe, cached_path)) = self.path_cache.get(&pid) {
            if cached_exe == exe {
                return (!cached_path.is_empty()).then(|| cached_path.clone());
            }
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process == null_mut() {
            self.path_cache
                .insert(pid, (exe.to_string(), String::new()));
            return None;
        }
        let mut buffer = vec![0u16; 32768];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
        CloseHandle(process);
        let path = if ok != 0 {
            from_wide(&buffer[..size as usize])
        } else {
            String::new()
        };
        self.path_cache.insert(pid, (exe.to_string(), path.clone()));
        (!path.is_empty()).then_some(path)
    }
}

fn build_detection_folders(config: &AppConfig) -> Vec<String> {
    let mut folders = BTreeSet::new();
    for folder in &config.game_folders {
        if normalize_path(folder).ends_with("\\steamapps") {
            for app_folder in
                steam_installed_app_folders(folder, &config.exclude_steam_app_keywords)
            {
                folders.insert(app_folder);
            }
        } else {
            folders.insert(folder.clone());
        }
    }
    folders.into_iter().collect()
}

fn steam_installed_app_folders(steamapps: &str, excluded_keywords: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir(steamapps) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let app_name = acf_value(&text, "name");
        let install_dir = acf_value(&text, "installdir");
        if install_dir.is_empty() {
            continue;
        }
        let haystack = lower(&(app_name + " " + &install_dir));
        if excluded_keywords
            .iter()
            .any(|keyword| haystack.contains(&lower(keyword)))
        {
            continue;
        }
        let folder = Path::new(steamapps).join("common").join(install_dir);
        if folder.is_dir() {
            result.push(folder.to_string_lossy().to_string());
        }
    }
    result
}

fn acf_value(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\"");
    for line in text.lines() {
        if line.contains(&needle) {
            let parts: Vec<_> = line.split('"').collect();
            if parts.len() >= 4 {
                return parts[3].replace("\\\\", "\\");
            }
        }
    }
    String::new()
}

fn scan_game_libraries() -> Vec<String> {
    let mut folders = BTreeSet::new();
    for drive in b'A'..=b'Z' {
        let root = format!("{}:\\", drive as char);
        if !Path::new(&root).exists() {
            continue;
        }
        for relative in [
            r"SteamLibrary\steamapps",
            "Games",
            "Epic Games",
            "GOG Games",
            "WeGameApps",
            "XboxGames",
        ] {
            let folder = Path::new(&root).join(relative);
            if folder.is_dir() {
                folders.insert(folder.to_string_lossy().to_string());
            }
        }
        let library = Path::new(&root).join(r"SteamLibrary\steamapps\libraryfolders.vdf");
        read_steam_libraries(&library, &mut folders);
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        read_steam_libraries(
            &Path::new(&program_files_x86).join(r"Steam\steamapps\libraryfolders.vdf"),
            &mut folders,
        );
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        read_steam_libraries(
            &Path::new(&program_files).join(r"Steam\steamapps\libraryfolders.vdf"),
            &mut folders,
        );
    }
    folders.into_iter().collect()
}

fn read_steam_libraries(file: &Path, folders: &mut BTreeSet<String>) {
    if !file.is_file() {
        return;
    }
    if let Some(parent) = file.parent() {
        folders.insert(parent.to_string_lossy().to_string());
    }
    let Ok(text) = fs::read_to_string(file) else {
        return;
    };
    for line in text.lines() {
        if line.contains("\"path\"") {
            let parts: Vec<_> = line.split('"').collect();
            if parts.len() >= 4 {
                folders.insert(format!("{}\\steamapps", parts[3].replace("\\\\", "\\")));
            }
        }
    }
}

impl Monitor {
    fn start(&self, hwnd: HWND, config: AppConfig) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        self.stopping.store(false, Ordering::SeqCst);
        let hwnd_value = hwnd as isize;
        let running = self.running.clone();
        let stopping = self.stopping.clone();
        thread::spawn(move || {
            let hwnd = hwnd_value as HWND;
            let mut app_enabled_speed_limits = false;
            let mut had_state = false;
            let mut last_game_running = false;
            let mut last_detected = String::new();
            let result = (|| -> Result<(), String> {
                let mut client = QbeeClient::new(&config);
                let mut detector = GameDetector::new(&config);
                log_line(&config, "Monitor started.");
                post_string(hwnd, WM_STATUS, "监控中".into());
                while !stopping.load(Ordering::SeqCst) {
                    let detected = detector.detect().unwrap_or_default();
                    let game_running = !detected.is_empty();
                    if detected != last_detected {
                        last_detected = detected.clone();
                        post_string(hwnd, WM_DETECTED, detected.clone());
                    }
                    if !had_state || game_running != last_game_running {
                        if game_running {
                            let already_enabled = client.speed_limits_enabled()?;
                            if !already_enabled {
                                client.set_speed_limits(true)?;
                                app_enabled_speed_limits = true;
                            } else {
                                app_enabled_speed_limits = false;
                            }
                            post_string(
                                hwnd,
                                WM_STATUS,
                                if already_enabled {
                                    "检测到游戏运行，备用速度限制原本已打开。"
                                } else {
                                    "检测到游戏运行，已打开备用速度限制。"
                                }
                                .into(),
                            );
                        } else if had_state {
                            if app_enabled_speed_limits {
                                let _ = client.set_speed_limits(false);
                                post_string(
                                    hwnd,
                                    WM_STATUS,
                                    "检测到游戏退出，已关闭备用速度限制。".into(),
                                );
                            } else {
                                post_string(
                                    hwnd,
                                    WM_STATUS,
                                    "检测到游戏退出，保留原本已打开的备用速度限制。".into(),
                                );
                            }
                            app_enabled_speed_limits = false;
                        }
                        had_state = true;
                        last_game_running = game_running;
                    }
                    for _ in 0..config.check_interval_seconds.max(1) * 10 {
                        if stopping.load(Ordering::SeqCst) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                if config.restore_on_exit && app_enabled_speed_limits {
                    let _ = client.set_speed_limits(false);
                }
                Ok(())
            })();
            if let Err(error) = result {
                post_string(hwnd, WM_STATUS, format!("监控出错：{error}"));
                log_line(&config, &format!("Monitor error: {error}"));
            }
            running.store(false, Ordering::SeqCst);
            stopping.store(false, Ordering::SeqCst);
            post_string(hwnd, WM_DETECTED, String::new());
            unsafe { PostMessageW(hwnd, WM_STOPPED, 0, 0) };
        });
    }

    fn stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
    }
}

fn startup_enabled() -> bool {
    unsafe {
        let mut key: HKEY = null_mut();
        let path = wide(RUN_KEY);
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return false;
        }
        let value = wide(RUN_VALUE);
        let ok = RegQueryValueExW(
            key,
            value.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        ) == ERROR_SUCCESS;
        RegCloseKey(key);
        ok
    }
}

fn set_startup_enabled(enabled: bool) {
    unsafe {
        let mut key: HKEY = null_mut();
        let path = wide(RUN_KEY);
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            null_mut(),
            0,
            KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        ) != ERROR_SUCCESS
        {
            return;
        }
        let value = wide(RUN_VALUE);
        if enabled {
            let exe = std::env::current_exe().unwrap_or_default();
            let command = wide(&format!("\"{}\"", exe.to_string_lossy()));
            RegSetValueExW(
                key,
                value.as_ptr(),
                0,
                REG_SZ,
                command.as_ptr() as *const u8,
                (command.len() * 2) as u32,
            );
        } else {
            RegDeleteValueW(key, value.as_ptr());
        }
        RegCloseKey(key);
    }
}

unsafe fn create_window_text(
    class: &str,
    text: &str,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: isize,
    parent: HWND,
    font: HFONT,
) -> HWND {
    let hwnd = CreateWindowExW(
        0,
        wide(class).as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE | style,
        ui_scale(x),
        ui_scale(y),
        ui_scale(w),
        ui_scale(h),
        parent,
        id as usize as HMENU,
        GetModuleHandleW(null()),
        null_mut(),
    );
    SendMessageW(hwnd, WM_SETFONT, font as usize, 1);
    hwnd
}

unsafe fn make_font(size: i32, weight: i32) -> HFONT {
    CreateFontW(
        -ui_scale(size),
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        CLEARTYPE_QUALITY as u32,
        DEFAULT_PITCH as u32,
        wide("Segoe UI").as_ptr(),
    )
}

unsafe fn ui_scale(value: i32) -> i32 {
    value
}

unsafe fn ui_rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left: ui_scale(left),
        top: ui_scale(top),
        right: ui_scale(right),
        bottom: ui_scale(bottom),
    }
}

unsafe fn draw_panel(hdc: HDC, rect: RECT, brush: HBRUSH) {
    let pen = CreatePen(PS_SOLID, 1, COLOR_BORDER);
    let old_pen = SelectObject(hdc, pen as _);
    let old_brush = SelectObject(hdc, brush as _);
    RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 18, 18);
    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    DeleteObject(pen as _);
}

unsafe fn draw_owner_button(item: &DRAWITEMSTRUCT, handles: &Handles) {
    let id = item.CtlID as isize;
    let disabled = (item.itemState & ODS_DISABLED as u32) != 0;
    let pressed = (item.itemState & ODS_SELECTED as u32) != 0;
    let focused = (item.itemState & ODS_FOCUS as u32) != 0;
    let is_primary = id == ID_START || id == ID_TEST;
    let brush = if disabled {
        handles.disabled
    } else if is_primary {
        handles.primary
    } else {
        handles.button
    };
    let border = if focused || (pressed && is_primary) {
        COLOR_PRIMARY_HOT
    } else {
        COLOR_BORDER
    };
    let text_color = if disabled {
        COLOR_MUTED
    } else if is_primary {
        rgb(255, 255, 255)
    } else {
        COLOR_TEXT
    };

    let pen = CreatePen(PS_SOLID, 1, border);
    let old_pen = SelectObject(item.hDC, pen as _);
    let old_brush = SelectObject(item.hDC, brush as _);
    RoundRect(
        item.hDC,
        item.rcItem.left,
        item.rcItem.top,
        item.rcItem.right,
        item.rcItem.bottom,
        10,
        10,
    );
    SelectObject(item.hDC, old_brush);
    SelectObject(item.hDC, old_pen);
    DeleteObject(pen as _);

    let len = GetWindowTextLengthW(item.hwndItem) as usize;
    let mut text = vec![0u16; len + 1];
    GetWindowTextW(item.hwndItem, text.as_mut_ptr(), text.len() as i32);
    let mut text_rect = item.rcItem;
    SetBkMode(item.hDC, TRANSPARENT as i32);
    SetTextColor(item.hDC, text_color);
    DrawTextW(
        item.hDC,
        text.as_mut_ptr(),
        -1,
        &mut text_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
}

fn save_from_ui(app: &mut App, show_status: bool) -> bool {
    let handles = &app.handles;
    let url = get_text(handles.url).trim().to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        unsafe {
            MessageBoxW(
                handles.hwnd,
                wide("请输入有效的 qB Web UI 地址。").as_ptr(),
                wide(APP_TITLE).as_ptr(),
                MB_ICONWARNING,
            );
        }
        return false;
    }
    let count = unsafe { SendMessageW(handles.folders, LB_GETCOUNT, 0, 0) as i32 };
    if count <= 0 {
        unsafe {
            MessageBoxW(
                handles.hwnd,
                wide("请至少添加一个游戏库文件夹，或点击“自动扫描”。").as_ptr(),
                wide(APP_TITLE).as_ptr(),
                MB_ICONWARNING,
            );
        }
        return false;
    }
    app.config.qbee_url = url;
    app.config.username = get_text(handles.user);
    app.config.password = get_text(handles.password);
    app.config.check_interval_seconds = get_text(handles.interval).parse().unwrap_or(5).max(1);
    app.config.start_with_windows =
        unsafe { SendMessageW(handles.startup, BM_GETCHECK, 0, 0) == BST_CHECKED as isize };
    app.config.auto_start_monitor =
        unsafe { SendMessageW(handles.autostart, BM_GETCHECK, 0, 0) == BST_CHECKED as isize };
    app.config.game_folders.clear();
    for i in 0..count {
        let len = unsafe { SendMessageW(handles.folders, LB_GETTEXTLEN, i as usize, 0) as usize };
        let mut buffer = vec![0u16; len + 1];
        unsafe {
            SendMessageW(
                handles.folders,
                LB_GETTEXT,
                i as usize,
                buffer.as_mut_ptr() as isize,
            )
        };
        app.config.game_folders.push(from_wide(&buffer));
    }
    let _ = save_config(&app.config);
    set_startup_enabled(app.config.start_with_windows);
    if show_status {
        set_status(handles, "已保存");
    }
    true
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let bg = CreateSolidBrush(COLOR_BG);
            let card = CreateSolidBrush(COLOR_CARD);
            let input = CreateSolidBrush(COLOR_INPUT);
            let button = CreateSolidBrush(COLOR_BUTTON);
            let primary = CreateSolidBrush(COLOR_PRIMARY);
            let disabled = CreateSolidBrush(COLOR_BUTTON_DISABLED);
            let font = make_font(15, FW_NORMAL as i32);
            let title_font = make_font(24, FW_SEMIBOLD as i32);

            create_window_text(
                "STATIC", APP_TITLE, SS_LEFT, 32, 28, 420, 38, 0, hwnd, title_font,
            );
            create_window_text(
                "STATIC",
                "Rust 原生低占用版 · Tailwind 风格简洁布局",
                SS_LEFT,
                34,
                70,
                620,
                24,
                0,
                hwnd,
                font,
            );
            let status = create_window_text(
                "STATIC",
                "状态：就绪",
                SS_LEFT | SS_CENTERIMAGE,
                34,
                96,
                880,
                28,
                0,
                hwnd,
                font,
            );

            create_window_text(
                "STATIC",
                "连接设置",
                SS_LEFT,
                48,
                134,
                180,
                24,
                0,
                hwnd,
                font,
            );
            create_window_text("STATIC", "地址", SS_LEFT, 58, 178, 56, 24, 0, hwnd, font);
            let url = create_window_text(
                "EDIT",
                "",
                WS_BORDER | ES_AUTOHSCROLL as u32,
                126,
                170,
                770,
                34,
                ID_URL,
                hwnd,
                font,
            );
            create_window_text("STATIC", "用户名", SS_LEFT, 58, 226, 58, 24, 0, hwnd, font);
            let user = create_window_text(
                "EDIT",
                "",
                WS_BORDER | ES_AUTOHSCROLL as u32,
                126,
                218,
                300,
                34,
                ID_USER,
                hwnd,
                font,
            );
            create_window_text("STATIC", "密码", SS_LEFT, 462, 226, 56, 24, 0, hwnd, font);
            let password = create_window_text(
                "EDIT",
                "",
                WS_BORDER | ES_AUTOHSCROLL as u32 | ES_PASSWORD as u32,
                514,
                218,
                300,
                34,
                ID_PASSWORD,
                hwnd,
                font,
            );
            create_window_text("STATIC", "间隔", SS_LEFT, 58, 274, 56, 24, 0, hwnd, font);
            let interval = create_window_text(
                "EDIT",
                "",
                WS_BORDER | ES_NUMBER as u32,
                126,
                266,
                82,
                34,
                ID_INTERVAL,
                hwnd,
                font,
            );
            create_window_text(
                "BUTTON",
                "测试连接",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                238,
                264,
                122,
                38,
                ID_TEST,
                hwnd,
                font,
            );
            let startup = create_window_text(
                "BUTTON",
                "开机自启动",
                BS_AUTOCHECKBOX as u32,
                400,
                270,
                150,
                28,
                ID_STARTUP,
                hwnd,
                font,
            );
            let autostart = create_window_text(
                "BUTTON",
                "启动后自动开始监控",
                BS_AUTOCHECKBOX as u32,
                590,
                270,
                260,
                28,
                ID_AUTOSTART,
                hwnd,
                font,
            );

            create_window_text("STATIC", "游戏库", SS_LEFT, 48, 374, 180, 24, 0, hwnd, font);
            let folders = create_window_text(
                "LISTBOX",
                "",
                WS_BORDER | LBS_NOTIFY as u32 | WS_VSCROLL | WS_HSCROLL,
                58,
                410,
                700,
                180,
                ID_FOLDERS,
                hwnd,
                font,
            );
            create_window_text(
                "BUTTON",
                "自动扫描",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                786,
                410,
                120,
                38,
                ID_SCAN,
                hwnd,
                font,
            );
            create_window_text(
                "BUTTON",
                "添加",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                786,
                460,
                120,
                38,
                ID_ADD,
                hwnd,
                font,
            );
            create_window_text(
                "BUTTON",
                "删除",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                786,
                510,
                120,
                38,
                ID_REMOVE,
                hwnd,
                font,
            );
            create_window_text(
                "BUTTON",
                "打开配置",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                786,
                560,
                120,
                38,
                ID_OPEN_CONFIG,
                hwnd,
                font,
            );

            let detected = create_window_text(
                "STATIC",
                "当前检测到：无",
                SS_LEFT,
                58,
                618,
                820,
                26,
                0,
                hwnd,
                font,
            );
            let save = create_window_text(
                "BUTTON",
                "保存",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                550,
                664,
                110,
                40,
                ID_SAVE,
                hwnd,
                font,
            );
            let start = create_window_text(
                "BUTTON",
                "开始监控",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                674,
                664,
                122,
                40,
                ID_START,
                hwnd,
                font,
            );
            let stop = create_window_text(
                "BUTTON",
                "停止监控",
                BS_PUSHBUTTON as u32 | BS_OWNERDRAW as u32,
                810,
                664,
                122,
                40,
                ID_STOP,
                hwnd,
                font,
            );
            EnableWindow(stop, 0);

            let config = load_config();
            let handles = Handles {
                hwnd,
                url,
                user,
                password,
                interval,
                startup,
                autostart,
                folders,
                status,
                detected,
                save,
                start,
                stop,
                font,
                title_font,
                bg,
                card,
                input,
                button,
                primary,
                disabled,
            };
            let app = Rc::new(RefCell::new(App {
                config,
                handles,
                monitor: Monitor::default(),
                closing_after_stop: false,
            }));
            load_to_ui(&app.borrow());
            APP.with(|slot| *slot.borrow_mut() = Some(app.clone()));
            if app.borrow().config.auto_start_monitor {
                PostMessageW(hwnd, WM_COMMAND, ID_START as usize, 0);
            }
            0
        }
        WM_ERASEBKGND => {
            APP.with(|slot| {
                if let Some(app) = slot.borrow().as_ref() {
                    let Ok(app) = app.try_borrow() else {
                        return;
                    };
                    let hdc = wparam as HDC;
                    let handles = &app.handles;
                    let mut rect = ui_rect(0, 0, UI_WIDTH, UI_HEIGHT);
                    FillRect(hdc, &rect, handles.bg);
                    rect = ui_rect(30, 112, 930, 328);
                    draw_panel(hdc, rect, handles.card);
                    rect = ui_rect(30, 350, 930, 604);
                    draw_panel(hdc, rect, handles.card);
                }
            });
            1
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, COLOR_TEXT);
            GetStockObject(NULL_BRUSH as i32) as isize
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as HDC;
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, COLOR_TEXT);
            GetStockObject(NULL_BRUSH as i32) as isize
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
            let hdc = wparam as HDC;
            SetBkColor(hdc, COLOR_INPUT);
            SetTextColor(hdc, COLOR_TEXT);
            APP.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .and_then(|app| app.try_borrow().ok().map(|app| app.handles.input as isize))
                    .unwrap_or(0)
            })
        }
        WM_DRAWITEM => APP.with(|slot| {
            if let Some(app) = slot.borrow().as_ref() {
                if let Ok(app) = app.try_borrow() {
                    draw_owner_button(&*(lparam as *const DRAWITEMSTRUCT), &app.handles);
                }
            }
            1
        }),
        WM_COMMAND => {
            let id = (wparam & 0xffff) as isize;
            APP.with(|slot| {
                if let Some(app_rc) = slot.borrow().as_ref() {
                    let mut app = app_rc.borrow_mut();
                    handle_command(&mut app, id);
                }
            });
            0
        }
        WM_STATUS => {
            let text = Box::from_raw(lparam as *mut String);
            APP.with(|slot| {
                if let Some(app) = slot.borrow().as_ref() {
                    let app = app.borrow();
                    set_status(&app.handles, &text);
                }
            });
            0
        }
        WM_DETECTED => {
            let text = Box::from_raw(lparam as *mut String);
            APP.with(|slot| {
                if let Some(app) = slot.borrow().as_ref() {
                    let label = if text.is_empty() {
                        "当前检测到：无".into()
                    } else {
                        format!("当前检测到：{}", text)
                    };
                    set_text(app.borrow().handles.detected, &label);
                }
            });
            0
        }
        WM_STOPPED => {
            APP.with(|slot| {
                if let Some(app_rc) = slot.borrow().as_ref() {
                    let app = app_rc.borrow_mut();
                    EnableWindow(app.handles.start, 1);
                    EnableWindow(app.handles.stop, 0);
                    set_text(app.handles.start, "开始监控");
                    set_text(app.handles.stop, "停止监控");
                    set_status(&app.handles, "已停止监控");
                    if app.closing_after_stop {
                        DestroyWindow(app.handles.hwnd);
                    }
                }
            });
            0
        }
        WM_SCAN_DONE => {
            let found = Box::from_raw(lparam as *mut Vec<String>);
            APP.with(|slot| {
                if let Some(app) = slot.borrow().as_ref() {
                    let app = app.borrow();
                    let mut existing = HashSet::new();
                    let count = SendMessageW(app.handles.folders, LB_GETCOUNT, 0, 0) as i32;
                    for i in 0..count {
                        let len = SendMessageW(app.handles.folders, LB_GETTEXTLEN, i as usize, 0)
                            as usize;
                        let mut buffer = vec![0u16; len + 1];
                        SendMessageW(
                            app.handles.folders,
                            LB_GETTEXT,
                            i as usize,
                            buffer.as_mut_ptr() as isize,
                        );
                        existing.insert(lower(&from_wide(&buffer)));
                    }
                    let mut added = 0;
                    for folder in found.iter() {
                        if existing.insert(lower(folder)) {
                            SendMessageW(
                                app.handles.folders,
                                LB_ADDSTRING,
                                0,
                                wide(folder).as_ptr() as isize,
                            );
                            added += 1;
                        }
                    }
                    set_status(
                        &app.handles,
                        &format!("自动扫描完成，新增 {added} 个游戏库。"),
                    );
                }
            });
            0
        }
        WM_CLOSE => {
            APP.with(|slot| {
                if let Some(app_rc) = slot.borrow().as_ref() {
                    let mut app = app_rc.borrow_mut();
                    if app.monitor.running.load(Ordering::SeqCst) {
                        let choice = MessageBoxW(
                            hwnd,
                            wide("监控仍在运行。要停止监控并退出吗？").as_ptr(),
                            wide(APP_TITLE).as_ptr(),
                            MB_YESNO | MB_ICONQUESTION,
                        );
                        if choice == IDYES {
                            app.closing_after_stop = true;
                            app.monitor.stop();
                            EnableWindow(app.handles.start, 0);
                            EnableWindow(app.handles.stop, 0);
                            set_text(app.handles.stop, "停止中");
                            set_status(&app.handles, "正在停止监控，完成后退出...");
                        }
                        return;
                    }
                }
                DestroyWindow(hwnd);
            });
            0
        }
        WM_DESTROY => {
            APP.with(|slot| {
                if let Some(app) = slot.borrow_mut().take() {
                    let handles = &app.borrow().handles;
                    DeleteObject(handles.font as _);
                    DeleteObject(handles.title_font as _);
                    DeleteObject(handles.bg as _);
                    DeleteObject(handles.card as _);
                    DeleteObject(handles.input as _);
                    DeleteObject(handles.button as _);
                    DeleteObject(handles.primary as _);
                    DeleteObject(handles.disabled as _);
                }
            });
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    r as u32 | ((g as u32) << 8) | ((b as u32) << 16)
}

fn load_to_ui(app: &App) {
    let handles = &app.handles;
    set_text(handles.url, &app.config.qbee_url);
    set_text(handles.user, &app.config.username);
    set_text(handles.password, &app.config.password);
    set_text(
        handles.interval,
        &app.config.check_interval_seconds.to_string(),
    );
    unsafe {
        SendMessageW(
            handles.startup,
            BM_SETCHECK,
            if startup_enabled() {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            } as usize,
            0,
        );
        SendMessageW(
            handles.autostart,
            BM_SETCHECK,
            if app.config.auto_start_monitor {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            } as usize,
            0,
        );
        SendMessageW(handles.folders, LB_RESETCONTENT, 0, 0);
        for folder in &app.config.game_folders {
            SendMessageW(
                handles.folders,
                LB_ADDSTRING,
                0,
                wide(folder).as_ptr() as isize,
            );
        }
    }
}

fn handle_command(app: &mut App, id: isize) {
    unsafe {
        match id {
            ID_TEST => {
                if !save_from_ui(app, false) {
                    return;
                }
                set_text(app.handles.save, "保存");
                set_status(&app.handles, "正在测试连接...");
                let hwnd_value = app.handles.hwnd as isize;
                let config = app.config.clone();
                thread::spawn(move || {
                    let hwnd = hwnd_value as HWND;
                    let mut client = QbeeClient::new(&config);
                    let status = match client.speed_limits_enabled() {
                        Ok(true) => "连接成功，备用速度限制当前已打开。".into(),
                        Ok(false) => "连接成功，备用速度限制当前已关闭。".into(),
                        Err(error) => format!("连接失败：{error}"),
                    };
                    post_string(hwnd, WM_STATUS, status);
                });
            }
            ID_SCAN => {
                set_text(app.handles.save, "保存");
                set_status(&app.handles, "正在扫描游戏库...");
                let hwnd_value = app.handles.hwnd as isize;
                thread::spawn(move || {
                    let hwnd = hwnd_value as HWND;
                    let found = scan_game_libraries();
                    PostMessageW(
                        hwnd,
                        WM_SCAN_DONE,
                        0,
                        Box::into_raw(Box::new(found)) as isize,
                    );
                });
            }
            ID_ADD => add_folder(app.handles.hwnd, app.handles.folders),
            ID_REMOVE => {
                let index = SendMessageW(app.handles.folders, LB_GETCURSEL, 0, 0);
                if index != LB_ERR as isize {
                    SendMessageW(app.handles.folders, LB_DELETESTRING, index as usize, 0);
                }
            }
            ID_OPEN_CONFIG => {
                ShellExecuteW(
                    app.handles.hwnd,
                    wide("open").as_ptr(),
                    wide("explorer.exe").as_ptr(),
                    wide(&app_dir().to_string_lossy()).as_ptr(),
                    null(),
                    SW_SHOWNORMAL,
                );
            }
            ID_SAVE => {
                if save_from_ui(app, true) {
                    set_text(app.handles.save, "已保存");
                }
            }
            ID_START => {
                if !save_from_ui(app, false) {
                    return;
                }
                EnableWindow(app.handles.start, 0);
                EnableWindow(app.handles.stop, 1);
                set_text(app.handles.save, "保存");
                set_text(app.handles.start, "监控中");
                set_text(app.handles.stop, "停止监控");
                set_status(&app.handles, "监控中，检测到游戏时会自动开启备用速度限制。");
                app.monitor.start(app.handles.hwnd, app.config.clone());
            }
            ID_STOP => {
                EnableWindow(app.handles.start, 0);
                EnableWindow(app.handles.stop, 0);
                set_text(app.handles.stop, "停止中");
                set_status(&app.handles, "正在停止监控...");
                app.monitor.stop();
            }
            _ => {}
        }
    }
}

unsafe fn add_folder(owner: HWND, list: HWND) {
    let mut browse: BROWSEINFOW = zeroed();
    let title = wide("选择游戏库文件夹");
    browse.hwndOwner = owner;
    browse.lpszTitle = title.as_ptr();
    browse.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;
    let pid = SHBrowseForFolderW(&mut browse);
    if !pid.is_null() {
        let mut buffer = vec![0u16; 32768];
        if SHGetPathFromIDListW(pid, buffer.as_mut_ptr()) != 0 {
            SendMessageW(list, LB_ADDSTRING, 0, buffer.as_ptr() as isize);
        }
        CoTaskMemFree(pid as *const c_void);
    }
}

fn main() {
    unsafe {
        SetProcessDPIAware();

        let mutex = CreateMutexW(
            null(),
            1,
            wide("QbeeGameSpeedLimiter.Rust.SingleInstance").as_ptr(),
        );
        if GetLastError() == ERROR_ALREADY_EXISTS {
            MessageBoxW(
                null_mut(),
                wide("qbee 游戏限速助手已经在运行。").as_ptr(),
                wide(APP_TITLE).as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
            return;
        }

        let mut icc = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&mut icc);

        let class_name = wide("QbeeGameSpeedLimiterRustWindow");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(null()),
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide(APP_TITLE).as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            ui_scale(UI_WIDTH),
            ui_scale(UI_HEIGHT),
            null_mut(),
            null_mut(),
            wc.hInstance,
            null_mut(),
        );
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if mutex != null_mut() {
            CloseHandle(mutex);
        }
    }
}
