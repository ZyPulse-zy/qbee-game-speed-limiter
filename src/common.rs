#![allow(dead_code)]

use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::Threading::*;

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "DownloadClientGameLimiterMonitor";
const LEGACY_RUN_VALUE: &str = "QbeeLimiterMonitor";
const MONITOR_MUTEX: &str = "DownloadClientGameLimiterMonitor.SingleInstance";
const CONFIG_FILE: &str = "download_client_game_speed_limiter.json";
const LEGACY_CONFIG_FILE: &str = "qbee_game_speed_limiter.json";
const STATUS_FILE: &str = "download_limiter_status.json";
const LEGACY_STATUS_FILE: &str = "qbee_limiter_status.json";
const STOP_FILE: &str = "download_limiter_monitor.stop";
const LEGACY_STOP_FILE: &str = "qbee_limiter_monitor.stop";
const MONITOR_EXE: &str = "download_limiter_monitor.exe";
const LEGACY_MONITOR_EXE: &str = "qbee_limiter_monitor.exe";
const CONFIG_EXE: &str = "download_limiter_config.exe";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub download_client: DownloadClientKind,
    pub qbee_url: String,
    pub transmission_url: String,
    pub aria2_url: String,
    pub utorrent_url: String,
    pub deluge_url: String,
    pub bitcomet_url: String,
    pub username: String,
    pub password: String,
    pub aria2_secret: String,
    pub game_download_limit_kib: u64,
    pub game_upload_limit_kib: u64,
    pub game_folders: Vec<String>,
    pub game_processes: Vec<String>,
    pub exclude_processes: Vec<String>,
    pub exclude_path_keywords: Vec<String>,
    pub exclude_steam_app_keywords: Vec<String>,
    pub check_interval_seconds: u64,
    pub restore_on_exit: bool,
    pub start_with_windows: bool,
    pub auto_start_monitor: bool,
    pub log_file: String,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadClientKind {
    QBittorrent,
    Transmission,
    Aria2,
    UTorrent,
    Deluge,
    BitComet,
}

impl Default for DownloadClientKind {
    fn default() -> Self {
        Self::QBittorrent
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            download_client: DownloadClientKind::QBittorrent,
            qbee_url: "http://127.0.0.1:8080".into(),
            transmission_url: "http://127.0.0.1:9091/transmission/rpc".into(),
            aria2_url: "http://127.0.0.1:6800/jsonrpc".into(),
            utorrent_url: "http://127.0.0.1:8080/gui".into(),
            deluge_url: "http://127.0.0.1:8112/json".into(),
            bitcomet_url: "http://127.0.0.1:80".into(),
            username: "admin".into(),
            password: String::new(),
            aria2_secret: String::new(),
            game_download_limit_kib: 512,
            game_upload_limit_kib: 128,
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
            auto_start_monitor: true,
            log_file: "download_client_game_speed_limiter.log".into(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub level: String,
    pub title: String,
    pub message: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub running: bool,
    pub game_running: bool,
    pub detected: String,
    pub message: String,
    pub updated_at: u64,
}

impl Default for MonitorStatus {
    fn default() -> Self {
        Self {
            running: false,
            game_running: false,
            detected: String::new(),
            message: "监控未运行".into(),
            updated_at: now_unix(),
        }
    }
}

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

pub fn from_wide(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

pub fn app_dir() -> PathBuf {
    let mut buffer = vec![0u16; 32768];
    let len = unsafe { GetModuleFileNameW(null_mut(), buffer.as_mut_ptr(), buffer.len() as u32) }
        as usize;
    PathBuf::from(from_wide(&buffer[..len]))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_path() -> PathBuf {
    app_dir().join(CONFIG_FILE)
}

pub fn legacy_config_path() -> PathBuf {
    app_dir().join(LEGACY_CONFIG_FILE)
}

pub fn status_path() -> PathBuf {
    app_dir().join(STATUS_FILE)
}

pub fn stop_path() -> PathBuf {
    app_dir().join(STOP_FILE)
}

pub fn legacy_stop_path() -> PathBuf {
    app_dir().join(LEGACY_STOP_FILE)
}

pub fn monitor_exe_path() -> PathBuf {
    app_dir().join(MONITOR_EXE)
}

pub fn legacy_monitor_exe_path() -> PathBuf {
    app_dir().join(LEGACY_MONITOR_EXE)
}

pub fn load_config() -> AppConfig {
    let path = if config_path().is_file() {
        config_path()
    } else {
        legacy_config_path()
    };
    match fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<AppConfig>(&text).ok())
    {
        Some(mut config) => {
            let defaults = AppConfig::default();
            if config.qbee_url.trim().is_empty() {
                config.qbee_url = defaults.qbee_url.clone();
            }
            if config.transmission_url.trim().is_empty() {
                config.transmission_url = defaults.transmission_url.clone();
            }
            if config.aria2_url.trim().is_empty() {
                config.aria2_url = defaults.aria2_url.clone();
            }
            if config.utorrent_url.trim().is_empty() {
                config.utorrent_url = defaults.utorrent_url.clone();
            }
            if config.deluge_url.trim().is_empty() {
                config.deluge_url = defaults.deluge_url.clone();
            }
            if config.bitcomet_url.trim().is_empty() {
                config.bitcomet_url = defaults.bitcomet_url.clone();
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

pub fn save_config(config: &AppConfig) -> std::io::Result<()> {
    fs::write(config_path(), serde_json::to_string_pretty(config).unwrap())
}

pub fn load_status() -> MonitorStatus {
    fs::read_to_string(status_path())
        .ok()
        .and_then(|text| serde_json::from_str::<MonitorStatus>(&text).ok())
        .unwrap_or_default()
}

pub fn save_status(status: &MonitorStatus) {
    let _ = fs::write(status_path(), serde_json::to_string_pretty(status).unwrap());
}

pub fn log_line(config: &AppConfig, message: &str) {
    let path = app_dir().join(if config.log_file.is_empty() {
        "download_client_game_speed_limiter.log"
    } else {
        &config.log_file
    });
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", message);
    }
}

pub fn lower(value: &str) -> String {
    value.to_lowercase()
}

pub fn normalize_path(value: &str) -> String {
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

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn request_stop_monitor() {
    let _ = fs::write(stop_path(), "stop");
    let _ = fs::write(legacy_stop_path(), "stop");
}

pub fn clear_stop_monitor() {
    let _ = fs::remove_file(stop_path());
    let _ = fs::remove_file(legacy_stop_path());
}

pub fn stop_monitor_requested() -> bool {
    stop_path().is_file() || legacy_stop_path().is_file()
}

pub fn start_monitor_process() -> Result<(), String> {
    clear_stop_monitor();
    let exe = monitor_exe_path();
    if !exe.is_file() {
        return Err(format!("找不到后台监控程序：{}", exe.display()));
    }
    std::process::Command::new(exe)
        .spawn()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

pub fn start_monitor_process_checked() -> Result<MonitorStatus, String> {
    start_monitor_process()?;
    let start = now_unix();
    for _ in 0..25 {
        thread::sleep(Duration::from_millis(120));
        let status = load_status();
        if status.running && status.updated_at >= start {
            return Ok(status);
        }
    }
    Err("后台监控启动后没有响应。可能已有旧版本监控程序正在运行，或 download_limiter_monitor.exe 启动后立即退出。请在任务管理器结束旧的 qbee_limiter_monitor.exe 后重试，或把新版本完整解压覆盖到原目录。".into())
}

pub fn create_desktop_shortcut() -> Result<PathBuf, String> {
    let profile =
        env::var("USERPROFILE").map_err(|_| "无法读取 USERPROFILE，不能定位桌面。".to_string())?;
    let candidates = [
        PathBuf::from(&profile).join("Desktop"),
        PathBuf::from(&profile).join("OneDrive").join("Desktop"),
        PathBuf::from(&profile).join("桌面"),
    ];
    let desktop = candidates
        .iter()
        .find(|path| path.is_dir())
        .cloned()
        .ok_or_else(|| "没有找到桌面目录。".to_string())?;
    let exe = app_dir().join(CONFIG_EXE);
    if !exe.is_file() {
        return Err(format!("找不到配置程序：{}", exe.display()));
    }
    let exe_url = format!("file:///{}", exe.to_string_lossy().replace('\\', "/"));
    let shortcut = desktop.join("下载器游戏限速助手.url");
    let content = format!(
        "[InternetShortcut]\r\nURL={exe_url}\r\nIconFile={}\r\nIconIndex=0\r\n",
        exe.to_string_lossy()
    );
    fs::write(&shortcut, content).map_err(|err| err.to_string())?;
    Ok(shortcut)
}
pub fn set_startup_enabled(enabled: bool) {
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
        ) != 0
        {
            return;
        }
        let value = wide(RUN_VALUE);
        let legacy_value = wide(LEGACY_RUN_VALUE);
        if enabled {
            let command = wide(&format!("\"{}\"", monitor_exe_path().to_string_lossy()));
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
        RegDeleteValueW(key, legacy_value.as_ptr());
        RegCloseKey(key);
    }
}

pub enum DownloadLimitSnapshot {
    QBittorrent {
        changed: bool,
    },
    Transmission {
        changed: bool,
    },
    Aria2 {
        previous_download: String,
        previous_upload: String,
    },
    UTorrent {
        previous_download: i64,
        previous_upload: i64,
    },
    Deluge {
        previous_download: f64,
        previous_upload: f64,
    },
    BitComet {
        previous_download: u64,
        previous_upload: u64,
    },
}

pub enum DownloadController {
    QBittorrent(QbeeClient),
    Transmission(TransmissionClient),
    Aria2(Aria2Client),
    UTorrent(UTorrentClient),
    Deluge(DelugeClient),
    BitComet(BitCometClient),
}

impl DownloadController {
    pub fn new(config: &AppConfig) -> Self {
        match config.download_client {
            DownloadClientKind::QBittorrent => Self::QBittorrent(QbeeClient::new(config)),
            DownloadClientKind::Transmission => Self::Transmission(TransmissionClient::new(config)),
            DownloadClientKind::Aria2 => Self::Aria2(Aria2Client::new(config)),
            DownloadClientKind::UTorrent => Self::UTorrent(UTorrentClient::new(config)),
            DownloadClientKind::Deluge => Self::Deluge(DelugeClient::new(config)),
            DownloadClientKind::BitComet => Self::BitComet(BitCometClient::new(config)),
        }
    }

    pub fn test_connection(&mut self, config: &AppConfig) -> Result<String, String> {
        match self {
            Self::QBittorrent(client) => match client.speed_limits_enabled()? {
                true => Ok("qB 连接成功，备用速度限制当前已开启。".into()),
                false => Ok("qB 连接成功，备用速度限制当前已关闭。".into()),
            },
            Self::Transmission(client) => match client.alt_speed_enabled()? {
                true => Ok("Transmission 连接成功，备用限速当前已开启。".into()),
                false => Ok("Transmission 连接成功，备用限速当前已关闭。".into()),
            },
            Self::Aria2(client) => {
                let (down, up) = client.current_limits()?;
                Ok(format!(
                    "aria2 连接成功，当前全局限速：下载 {down}，上传 {up}。游戏中将切换为 {}K / {}K。",
                    config.game_download_limit_kib, config.game_upload_limit_kib
                ))
            }
            Self::UTorrent(client) => {
                let (down, up) = client.current_limits()?;
                Ok(format!(
                    "µTorrent/BitTorrent 连接成功，当前全局限速：下载 {}，上传 {}。游戏中将切换为 {}K / {}K。",
                    format_int_limit(down),
                    format_int_limit(up),
                    config.game_download_limit_kib,
                    config.game_upload_limit_kib
                ))
            }
            Self::Deluge(client) => {
                let (down, up) = client.current_limits()?;
                Ok(format!(
                    "Deluge 连接成功，当前全局限速：下载 {}，上传 {}。游戏中将切换为 {}K / {}K。",
                    format_float_limit(down),
                    format_float_limit(up),
                    config.game_download_limit_kib,
                    config.game_upload_limit_kib
                ))
            }
            Self::BitComet(client) => {
                let (down, up) = client.current_limits()?;
                Ok(format!(
                    "BitComet 连接成功，当前全局限速：下载 {}，上传 {}。游戏中将切换为 {}K / {}K。",
                    format_byte_limit(down),
                    format_byte_limit(up),
                    config.game_download_limit_kib,
                    config.game_upload_limit_kib
                ))
            }
        }
    }

    pub fn enable_game_mode(
        &mut self,
        config: &AppConfig,
    ) -> Result<(DownloadLimitSnapshot, String), String> {
        match self {
            Self::QBittorrent(client) => {
                let already_enabled = client.speed_limits_enabled()?;
                if already_enabled {
                    Ok((
                        DownloadLimitSnapshot::QBittorrent { changed: false },
                        "检测到游戏运行，qB 备用速度限制原本已打开。".into(),
                    ))
                } else {
                    client.set_speed_limits(true)?;
                    Ok((
                        DownloadLimitSnapshot::QBittorrent { changed: true },
                        "检测到游戏运行，已打开 qB 备用速度限制。".into(),
                    ))
                }
            }
            Self::Transmission(client) => {
                let already_enabled = client.alt_speed_enabled()?;
                if already_enabled {
                    Ok((
                        DownloadLimitSnapshot::Transmission { changed: false },
                        "检测到游戏运行，Transmission 备用限速原本已打开。".into(),
                    ))
                } else {
                    client.set_alt_speed(true)?;
                    Ok((
                        DownloadLimitSnapshot::Transmission { changed: true },
                        "检测到游戏运行，已打开 Transmission 备用限速。".into(),
                    ))
                }
            }
            Self::Aria2(client) => {
                let (previous_download, previous_upload) = client.current_limits()?;
                client.set_limits(config.game_download_limit_kib, config.game_upload_limit_kib)?;
                Ok((
                    DownloadLimitSnapshot::Aria2 {
                        previous_download,
                        previous_upload,
                    },
                    format!(
                        "检测到游戏运行，已把 aria2 全局限速切换为下载 {}K / 上传 {}K。",
                        config.game_download_limit_kib, config.game_upload_limit_kib
                    ),
                ))
            }
            Self::UTorrent(client) => {
                let (previous_download, previous_upload) = client.current_limits()?;
                client.set_limits(config.game_download_limit_kib, config.game_upload_limit_kib)?;
                Ok((
                    DownloadLimitSnapshot::UTorrent {
                        previous_download,
                        previous_upload,
                    },
                    format!(
                        "检测到游戏运行，已把 µTorrent/BitTorrent 全局限速切换为下载 {}K / 上传 {}K。",
                        config.game_download_limit_kib, config.game_upload_limit_kib
                    ),
                ))
            }
            Self::Deluge(client) => {
                let (previous_download, previous_upload) = client.current_limits()?;
                client.set_limits(config.game_download_limit_kib, config.game_upload_limit_kib)?;
                Ok((
                    DownloadLimitSnapshot::Deluge {
                        previous_download,
                        previous_upload,
                    },
                    format!(
                        "检测到游戏运行，已把 Deluge 全局限速切换为下载 {}K / 上传 {}K。",
                        config.game_download_limit_kib, config.game_upload_limit_kib
                    ),
                ))
            }
            Self::BitComet(client) => {
                let (previous_download, previous_upload) = client.current_limits()?;
                client.set_limits(config.game_download_limit_kib, config.game_upload_limit_kib)?;
                Ok((
                    DownloadLimitSnapshot::BitComet {
                        previous_download,
                        previous_upload,
                    },
                    format!(
                        "检测到游戏运行，已把 BitComet 全局限速切换为下载 {}K / 上传 {}K。",
                        config.game_download_limit_kib, config.game_upload_limit_kib
                    ),
                ))
            }
        }
    }

    pub fn restore(&mut self, snapshot: DownloadLimitSnapshot) -> Result<String, String> {
        match (self, snapshot) {
            (Self::QBittorrent(client), DownloadLimitSnapshot::QBittorrent { changed }) => {
                if changed {
                    client.set_speed_limits(false)?;
                    Ok("检测到游戏退出，已关闭 qB 备用速度限制。".into())
                } else {
                    Ok("检测到游戏退出，保留原本已打开的 qB 备用速度限制。".into())
                }
            }
            (Self::Transmission(client), DownloadLimitSnapshot::Transmission { changed }) => {
                if changed {
                    client.set_alt_speed(false)?;
                    Ok("检测到游戏退出，已关闭 Transmission 备用限速。".into())
                } else {
                    Ok("检测到游戏退出，保留原本已打开的 Transmission 备用限速。".into())
                }
            }
            (
                Self::Aria2(client),
                DownloadLimitSnapshot::Aria2 {
                    previous_download,
                    previous_upload,
                },
            ) => {
                client.restore_limits(&previous_download, &previous_upload)?;
                Ok("检测到游戏退出，已恢复 aria2 原来的全局限速。".into())
            }
            (
                Self::UTorrent(client),
                DownloadLimitSnapshot::UTorrent {
                    previous_download,
                    previous_upload,
                },
            ) => {
                client.restore_limits(previous_download, previous_upload)?;
                Ok("检测到游戏退出，已恢复 µTorrent/BitTorrent 原来的全局限速。".into())
            }
            (
                Self::Deluge(client),
                DownloadLimitSnapshot::Deluge {
                    previous_download,
                    previous_upload,
                },
            ) => {
                client.restore_limits(previous_download, previous_upload)?;
                Ok("检测到游戏退出，已恢复 Deluge 原来的全局限速。".into())
            }
            (
                Self::BitComet(client),
                DownloadLimitSnapshot::BitComet {
                    previous_download,
                    previous_upload,
                },
            ) => {
                client.restore_limits(previous_download, previous_upload)?;
                Ok("检测到游戏退出，已恢复 BitComet 原来的全局限速。".into())
            }
            _ => Ok("检测到游戏退出，当前下载器无需恢复。".into()),
        }
    }
}

pub struct QbeeClient {
    base_url: String,
    username: String,
    password: String,
    agent: ureq::Agent,
    cookie: Option<String>,
    logged_in: bool,
}

impl QbeeClient {
    pub fn new(config: &AppConfig) -> Self {
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

    pub fn speed_limits_enabled(&mut self) -> Result<bool, String> {
        self.ensure_login()?;
        Ok(self
            .request("GET", "/api/v2/transfer/speedLimitsMode", None)?
            .trim()
            == "1")
    }

    pub fn set_speed_limits(&mut self, enabled: bool) -> Result<bool, String> {
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
            return Err("qB 登录失败，请检查 Web UI 用户名和密码。".into());
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
        .set("User-Agent", "download-client-game-limiter/7.0")
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

pub struct TransmissionClient {
    url: String,
    username: String,
    password: String,
    agent: ureq::Agent,
    session_id: Option<String>,
}

impl TransmissionClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            url: config.transmission_url.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            session_id: None,
        }
    }

    pub fn alt_speed_enabled(&mut self) -> Result<bool, String> {
        let value = self.rpc(
            "session-get",
            serde_json::json!({ "fields": ["alt-speed-enabled"] }),
        )?;
        value["arguments"]["alt-speed-enabled"]
            .as_bool()
            .ok_or_else(|| "Transmission 返回中没有 alt-speed-enabled 字段。".to_string())
    }

    pub fn set_alt_speed(&mut self, enabled: bool) -> Result<(), String> {
        self.rpc(
            "session-set",
            serde_json::json!({ "alt-speed-enabled": enabled }),
        )?;
        Ok(())
    }

    fn rpc(
        &mut self,
        method: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({ "method": method, "arguments": arguments }).to_string();
        self.send_rpc(&body, true)
    }

    fn send_rpc(&mut self, body: &str, allow_retry: bool) -> Result<serde_json::Value, String> {
        let mut req = self
            .agent
            .post(&self.url)
            .set("User-Agent", "download-client-game-limiter/7.0")
            .set("Content-Type", "application/json");
        if let Some(session_id) = &self.session_id {
            req = req.set("X-Transmission-Session-Id", session_id);
        }
        if !self.username.is_empty() || !self.password.is_empty() {
            req = req.set(
                "Authorization",
                &format!("Basic {}", base64_basic(&self.username, &self.password)),
            );
        }
        match req.send_string(body) {
            Ok(response) => parse_transmission_response(response),
            Err(ureq::Error::Status(409, response)) if allow_retry => {
                if let Some(session_id) = response.header("X-Transmission-Session-Id") {
                    self.session_id = Some(session_id.to_string());
                    self.send_rpc(body, false)
                } else {
                    Err("Transmission 要求 Session ID，但响应中没有返回。".into())
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

fn parse_transmission_response(response: ureq::Response) -> Result<serde_json::Value, String> {
    let text = response.into_string().map_err(|err| err.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    if value["result"].as_str() == Some("success") {
        Ok(value)
    } else {
        Err(format!(
            "Transmission RPC 失败：{}",
            value["result"].as_str().unwrap_or("unknown")
        ))
    }
}

pub struct Aria2Client {
    url: String,
    secret: String,
    agent: ureq::Agent,
}

impl Aria2Client {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            url: config.aria2_url.clone(),
            secret: config.aria2_secret.clone(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
        }
    }

    pub fn current_limits(&mut self) -> Result<(String, String), String> {
        let value = self.rpc("aria2.getGlobalOption", vec![])?;
        let download = value["max-overall-download-limit"]
            .as_str()
            .unwrap_or("0")
            .to_string();
        let upload = value["max-overall-upload-limit"]
            .as_str()
            .unwrap_or("0")
            .to_string();
        Ok((download, upload))
    }

    pub fn set_limits(&mut self, download_kib: u64, upload_kib: u64) -> Result<(), String> {
        let options = serde_json::json!({
            "max-overall-download-limit": format!("{}K", download_kib),
            "max-overall-upload-limit": format!("{}K", upload_kib),
        });
        self.rpc("aria2.changeGlobalOption", vec![options])?;
        Ok(())
    }

    pub fn restore_limits(&mut self, download: &str, upload: &str) -> Result<(), String> {
        let options = serde_json::json!({
            "max-overall-download-limit": download,
            "max-overall-upload-limit": upload,
        });
        self.rpc("aria2.changeGlobalOption", vec![options])?;
        Ok(())
    }

    fn rpc(
        &mut self,
        method: &str,
        mut params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        if !self.secret.is_empty() {
            params.insert(0, serde_json::json!(format!("token:{}", self.secret)));
        }
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "download-client-game-limiter",
            "method": method,
            "params": params,
        })
        .to_string();
        let response = self
            .agent
            .post(&self.url)
            .set("User-Agent", "download-client-game-limiter/7.0")
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|err| err.to_string())?;
        let text = response.into_string().map_err(|err| err.to_string())?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| err.to_string())?;
        if !value["error"].is_null() {
            return Err(format!(
                "aria2 RPC 失败：{}",
                value["error"]["message"].as_str().unwrap_or("unknown")
            ));
        }
        Ok(value["result"].clone())
    }
}

pub struct UTorrentClient {
    base_url: String,
    username: String,
    password: String,
    agent: ureq::Agent,
    token: Option<String>,
    cookie: Option<String>,
}

impl UTorrentClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            base_url: normalize_utorrent_url(&config.utorrent_url),
            username: config.username.clone(),
            password: config.password.clone(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            token: None,
            cookie: None,
        }
    }

    pub fn current_limits(&mut self) -> Result<(i64, i64), String> {
        let value = self.api("action=getsettings")?;
        let settings = value["settings"]
            .as_array()
            .ok_or_else(|| "µTorrent/BitTorrent 返回中没有 settings 字段。".to_string())?;
        let download = utorrent_setting_i64(settings, "max_dl_rate").unwrap_or(0);
        let upload = utorrent_setting_i64(settings, "max_ul_rate").unwrap_or(0);
        Ok((download, upload))
    }

    pub fn set_limits(&mut self, download_kib: u64, upload_kib: u64) -> Result<(), String> {
        self.set_setting("max_dl_rate", &download_kib.to_string())?;
        self.set_setting("max_ul_rate", &upload_kib.to_string())?;
        Ok(())
    }

    pub fn restore_limits(&mut self, download_kib: i64, upload_kib: i64) -> Result<(), String> {
        self.set_setting("max_dl_rate", &download_kib.to_string())?;
        self.set_setting("max_ul_rate", &upload_kib.to_string())?;
        Ok(())
    }

    fn set_setting(&mut self, name: &str, value: &str) -> Result<(), String> {
        self.api(&format!(
            "action=setsetting&s={}&v={}",
            url_encode(name),
            url_encode(value)
        ))?;
        Ok(())
    }

    fn api(&mut self, query: &str) -> Result<serde_json::Value, String> {
        self.ensure_token()?;
        let token = self.token.clone().unwrap_or_default();
        let url = format!("{}/?token={}&{}", self.base_url, url_encode(&token), query);
        let response = self
            .with_common_headers(self.agent.get(&url))
            .call()
            .map_err(|err| format!("µTorrent/BitTorrent Web API 失败：{err}"))?;
        self.capture_cookie(&response);
        let text = response.into_string().map_err(|err| err.to_string())?;
        serde_json::from_str(&text)
            .map_err(|err| format!("µTorrent/BitTorrent 返回不是有效 JSON：{err}"))
    }

    fn ensure_token(&mut self) -> Result<(), String> {
        if self.token.is_some() {
            return Ok(());
        }
        let url = format!("{}/token.html", self.base_url);
        let response = self
            .with_common_headers(self.agent.get(&url))
            .call()
            .map_err(|err| format!("无法获取 µTorrent/BitTorrent Token：{err}"))?;
        self.capture_cookie(&response);
        let text = response.into_string().map_err(|err| err.to_string())?;
        let token = extract_utorrent_token(&text).ok_or_else(|| {
            "没有在 token.html 中找到 µTorrent/BitTorrent Token，请检查 Web UI 地址和账号密码。"
                .to_string()
        })?;
        self.token = Some(token);
        Ok(())
    }

    fn with_common_headers(&self, req: ureq::Request) -> ureq::Request {
        let mut req = req
            .set("User-Agent", "download-client-game-limiter/7.0")
            .set("Referer", &self.base_url);
        if !self.username.is_empty() || !self.password.is_empty() {
            req = req.set(
                "Authorization",
                &format!("Basic {}", base64_basic(&self.username, &self.password)),
            );
        }
        if let Some(cookie) = &self.cookie {
            req = req.set("Cookie", cookie);
        }
        req
    }

    fn capture_cookie(&mut self, response: &ureq::Response) {
        if let Some(cookie) = response.header("set-cookie") {
            self.cookie = cookie.split(';').next().map(str::to_string);
        }
    }
}

fn normalize_utorrent_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.ends_with("/gui") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/gui")
    }
}

fn extract_utorrent_token(text: &str) -> Option<String> {
    for marker in ["id='token'", "id=\"token\""] {
        if let Some(marker_index) = text.find(marker) {
            let after_marker = &text[marker_index..];
            let start = after_marker.find('>')? + 1;
            let after_start = &after_marker[start..];
            let end = after_start.find('<')?;
            let token = after_start[..end].trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    None
}

fn utorrent_setting_i64(settings: &[serde_json::Value], name: &str) -> Option<i64> {
    settings.iter().find_map(|row| {
        let row = row.as_array()?;
        if row.first()?.as_str()? != name {
            return None;
        }
        let value = row.get(2)?;
        value
            .as_i64()
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    })
}

pub struct DelugeClient {
    url: String,
    password: String,
    agent: ureq::Agent,
    cookie: Option<String>,
    logged_in: bool,
}

impl DelugeClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            url: normalize_deluge_url(&config.deluge_url),
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

    pub fn current_limits(&mut self) -> Result<(f64, f64), String> {
        let value = self.rpc("core.get_config", serde_json::json!([]))?;
        let download = json_number(&value["max_download_speed"]).unwrap_or(-1.0);
        let upload = json_number(&value["max_upload_speed"]).unwrap_or(-1.0);
        Ok((download, upload))
    }

    pub fn set_limits(&mut self, download_kib: u64, upload_kib: u64) -> Result<(), String> {
        self.rpc(
            "core.set_config",
            serde_json::json!([{
                "max_download_speed": download_kib as f64,
                "max_upload_speed": upload_kib as f64,
            }]),
        )?;
        Ok(())
    }

    pub fn restore_limits(&mut self, download_kib: f64, upload_kib: f64) -> Result<(), String> {
        self.rpc(
            "core.set_config",
            serde_json::json!([{
                "max_download_speed": download_kib,
                "max_upload_speed": upload_kib,
            }]),
        )?;
        Ok(())
    }

    fn ensure_login(&mut self) -> Result<(), String> {
        if self.logged_in {
            return Ok(());
        }
        let value = self.rpc_raw("auth.login", serde_json::json!([self.password.clone()]))?;
        if value.as_bool() == Some(true) {
            self.logged_in = true;
            Ok(())
        } else {
            Err("Deluge Web 登录失败，请检查 Web UI 密码。".into())
        }
    }

    fn rpc(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.ensure_login()?;
        self.rpc_raw(method, params)
    }

    fn rpc_raw(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({
            "method": method,
            "params": params,
            "id": 1,
        })
        .to_string();
        let mut req = self
            .agent
            .post(&self.url)
            .set("User-Agent", "download-client-game-limiter/7.0")
            .set("Content-Type", "application/json");
        if let Some(cookie) = &self.cookie {
            req = req.set("Cookie", cookie);
        }
        let response = req
            .send_string(&body)
            .map_err(|err| format!("Deluge Web API 失败：{err}"))?;
        if let Some(cookie) = response.header("set-cookie") {
            self.cookie = cookie.split(';').next().map(str::to_string);
        }
        let text = response.into_string().map_err(|err| err.to_string())?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("Deluge 返回不是有效 JSON：{err}"))?;
        if !value["error"].is_null() {
            return Err(format!(
                "Deluge RPC 失败：{}",
                value["error"]["message"].as_str().unwrap_or("unknown")
            ));
        }
        Ok(value["result"].clone())
    }
}

fn normalize_deluge_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.ends_with("/json") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/json")
    }
}

fn json_number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|number| number as f64))
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

fn format_int_limit(value: i64) -> String {
    if value <= 0 {
        "不限速".to_string()
    } else {
        format!("{value}K")
    }
}

fn format_float_limit(value: f64) -> String {
    if value < 0.0 {
        "不限速".to_string()
    } else {
        format!("{value:.0}K")
    }
}

fn format_byte_limit(value: u64) -> String {
    if value == 0 {
        "不限速".to_string()
    } else if value % (1024 * 1024) == 0 {
        format!("{}M", value / 1024 / 1024)
    } else {
        format!("{}K", value / 1024)
    }
}

pub struct BitCometClient {
    base_url: String,
    username: String,
    password: String,
    agent: ureq::Agent,
    token: Option<String>,
    client_id: String,
}

impl BitCometClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            base_url: normalize_bitcomet_url(&config.bitcomet_url),
            username: config.username.clone(),
            password: config.password.clone(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            token: None,
            client_id: bitcomet_client_id(),
        }
    }

    pub fn current_limits(&mut self) -> Result<(u64, u64), String> {
        let config = self.connection_config()?;
        let download = json_u64(&config["max_download_speed"]).unwrap_or(0);
        let upload = json_u64(&config["max_upload_speed"]).unwrap_or(0);
        Ok((download, upload))
    }

    pub fn set_limits(&mut self, download_kib: u64, upload_kib: u64) -> Result<(), String> {
        self.set_limits_bytes(
            download_kib.saturating_mul(1024),
            upload_kib.saturating_mul(1024),
        )
    }

    pub fn restore_limits(&mut self, download_bps: u64, upload_bps: u64) -> Result<(), String> {
        self.set_limits_bytes(download_bps, upload_bps)
    }

    fn set_limits_bytes(&mut self, download_bps: u64, upload_bps: u64) -> Result<(), String> {
        let mut config = self.connection_config()?;
        if let Some(object) = config.as_object_mut() {
            object.insert(
                "max_download_speed".to_string(),
                serde_json::json!(download_bps),
            );
            object.insert(
                "max_upload_speed".to_string(),
                serde_json::json!(upload_bps),
            );
        } else {
            config = serde_json::json!({
                "max_download_speed": download_bps,
                "max_upload_speed": upload_bps,
            });
        }
        let value = self.api_post(
            "/api/config/connection/set",
            serde_json::json!({ "connection_config": config }),
        )?;
        ensure_bitcomet_ok(&value)
    }

    fn connection_config(&mut self) -> Result<serde_json::Value, String> {
        let value = self.api_post("/api/config/connection/get", serde_json::json!({}))?;
        if let Some(error) = bitcomet_error_message(&value) {
            return Err(error);
        }
        let config = value["connection_config"].clone();
        if config.is_null() {
            Err("BitComet 返回中没有 connection_config 字段，请确认使用 2.16 或更新版本并启用 WebUI。".into())
        } else {
            Ok(config)
        }
    }

    fn api_post(
        &mut self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.ensure_token()?;
        let token = self.token.clone().unwrap_or_default();
        self.post_json(path, body, Some(&token))
    }

    fn ensure_token(&mut self) -> Result<(), String> {
        if self.token.is_some() {
            return Ok(());
        }
        if self.username.trim().is_empty() {
            return Err("BitComet WebUI 需要用户名和密码。请在 BitComet 的“远程访问 / WebUI”里启用 Web 远程访问并填写账号密码。".into());
        }
        let login_body = serde_json::json!({
            "username": self.username.clone(),
            "password": self.password.clone(),
        })
        .to_string();
        let authentication = bitcomet_encrypt(&login_body, &self.client_id)?;
        let login_value = self.post_json(
            "/api/webui/login",
            serde_json::json!({
                "client_id": self.client_id.clone(),
                "authentication": authentication,
            }),
            None,
        )?;
        if let Some(error) = bitcomet_error_message(&login_value) {
            return Err(error);
        }
        let invite_token = login_value["invite_token"]
            .as_str()
            .ok_or_else(|| "BitComet 登录成功响应中没有 invite_token。".to_string())?
            .to_string();
        let token_value = self.post_json(
            "/api/device_token/get",
            serde_json::json!({
                "invite_token": invite_token,
                "device_id": self.client_id.clone(),
                "device_name": "Download Client Game Speed Limiter",
                "platform": "webui",
            }),
            Some(&invite_token),
        )?;
        let token = token_value["device_token"]
            .as_str()
            .ok_or_else(|| "BitComet 没有返回 device_token，请检查 WebUI 账号密码。".to_string())?
            .to_string();
        if token.is_empty() {
            Err("BitComet 返回了空 device_token，请检查 WebUI 账号密码。".into())
        } else {
            self.token = Some(token);
            Ok(())
        }
    }

    fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .agent
            .post(&url)
            .set("User-Agent", "download-client-game-limiter/7.0")
            .set("Client-Type", "BitComet WebUI")
            .set("Content-Type", "application/json");
        if let Some(token) = bearer {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        let response = req
            .send_string(&body.to_string())
            .map_err(|err| format!("BitComet WebUI API 失败：{err}"))?;
        let text = response.into_string().map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| format!("BitComet 返回不是有效 JSON：{err}"))
    }
}

fn normalize_bitcomet_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

fn json_u64(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

fn ensure_bitcomet_ok(value: &serde_json::Value) -> Result<(), String> {
    match bitcomet_error_message(value) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn bitcomet_error_message(value: &serde_json::Value) -> Option<String> {
    let code = value["error_code"].as_str()?;
    if code.eq_ignore_ascii_case("OK") {
        None
    } else {
        Some(format!(
            "BitComet WebUI 返回错误：{}",
            value["error_message"].as_str().unwrap_or(code)
        ))
    }
}

fn bitcomet_client_id() -> String {
    let seed = format!(
        "download-client-game-speed-limiter:{}:{}",
        env::var("COMPUTERNAME").unwrap_or_default(),
        app_dir().display()
    );
    let digest = Sha256::digest(seed.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn bitcomet_encrypt(plain_text: &str, password: &str) -> Result<String, String> {
    let mut salt_key = [0u8; 8];
    let mut salt_hmac = [0u8; 8];
    let mut iv = [0u8; 16];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt_key);
    rng.fill_bytes(&mut salt_hmac);
    rng.fill_bytes(&mut iv);

    let mut key = [0u8; 32];
    let mut hmac_key = [0u8; 32];
    pbkdf2_hmac::<Sha1>(password.as_bytes(), &salt_key, 10_000, &mut key);
    pbkdf2_hmac::<Sha1>(password.as_bytes(), &salt_hmac, 10_000, &mut hmac_key);

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plain_text.as_bytes());

    let mut packet = Vec::with_capacity(2 + 8 + 8 + 16 + ciphertext.len() + 32);
    packet.extend_from_slice(&[3, 1]);
    packet.extend_from_slice(&salt_key);
    packet.extend_from_slice(&salt_hmac);
    packet.extend_from_slice(&iv);
    packet.extend_from_slice(&ciphertext);
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&hmac_key).map_err(|err| err.to_string())?;
    mac.update(&packet);
    packet.extend_from_slice(&mac.finalize().into_bytes());
    Ok(BASE64_STANDARD.encode(packet))
}

pub fn test_connection(config: &AppConfig) -> Result<String, String> {
    let mut controller = DownloadController::new(config);
    controller.test_connection(config)
}
pub fn run_diagnostics(config: &AppConfig) -> Vec<DiagnosticItem> {
    let mut items = Vec::new();
    push_diag(
        &mut items,
        "ok",
        "程序目录",
        &format!("当前程序目录：{}", app_dir().display()),
    );

    if monitor_exe_path().is_file() {
        push_diag(
            &mut items,
            "ok",
            "后台监控程序",
            "已找到 download_limiter_monitor.exe。",
        )
    } else {
        push_diag(
            &mut items,
            "error",
            "后台监控程序缺失",
            &format!(
                "没有找到：{}。请把两个 exe 放在同一文件夹。",
                monitor_exe_path().display()
            ),
        );
    }

    if config_path().is_file() {
        push_diag(
            &mut items,
            "ok",
            "配置文件",
            "已找到 download_client_game_speed_limiter.json。",
        )
    } else {
        push_diag(
            &mut items,
            "warn",
            "配置文件未创建",
            "保存配置后会自动创建 download_client_game_speed_limiter.json。",
        );
    }

    match config.download_client {
        DownloadClientKind::QBittorrent => {
            check_url(&mut items, "qB Web UI 地址", &config.qbee_url)
        }
        DownloadClientKind::Transmission => check_url(
            &mut items,
            "Transmission RPC 地址",
            &config.transmission_url,
        ),
        DownloadClientKind::Aria2 => {
            check_url(&mut items, "aria2 JSON-RPC 地址", &config.aria2_url);
            push_game_limit_diag(&mut items, config);
        }
        DownloadClientKind::UTorrent => {
            check_url(
                &mut items,
                "µTorrent/BitTorrent Web UI 地址",
                &config.utorrent_url,
            );
            push_game_limit_diag(&mut items, config);
        }
        DownloadClientKind::Deluge => {
            check_url(&mut items, "Deluge Web JSON 地址", &config.deluge_url);
            push_game_limit_diag(&mut items, config);
        }
        DownloadClientKind::BitComet => {
            check_url(&mut items, "BitComet WebUI 地址", &config.bitcomet_url);
            push_game_limit_diag(&mut items, config);
        }
    }

    let folders: Vec<_> = config
        .game_folders
        .iter()
        .filter(|folder| !folder.trim().is_empty())
        .collect();
    let existing = folders
        .iter()
        .filter(|folder| Path::new(folder.as_str()).is_dir())
        .count();
    if folders.is_empty() {
        push_diag(
            &mut items,
            "warn",
            "游戏库目录",
            "还没有添加游戏库目录。请先自动扫描或手动添加。",
        );
    } else if existing == 0 {
        push_diag(
            &mut items,
            "warn",
            "游戏库目录",
            "已添加目录，但当前没有一个路径存在。请检查盘符和目录层级。Steam 建议添加 steamapps。",
        );
    } else {
        push_diag(
            &mut items,
            "ok",
            "游戏库目录",
            &format!(
                "已添加 {} 个目录，其中 {} 个存在。",
                folders.len(),
                existing
            ),
        );
    }

    let status = load_status();
    if status.running {
        let age = now_unix().saturating_sub(status.updated_at);
        if age > config.check_interval_seconds.max(1) * 4 + 10 {
            push_diag(
                &mut items,
                "warn",
                "后台监控状态",
                "状态文件较久没有更新，后台监控可能卡住或已经退出。",
            );
        } else {
            push_diag(&mut items, "ok", "后台监控状态", "后台监控正在运行。")
        }
    } else {
        push_diag(
            &mut items,
            "warn",
            "后台监控状态",
            "后台监控未运行。点击“启动监控”或勾选保存后自动启动。",
        );
    }

    if config.start_with_windows {
        push_diag(
            &mut items,
            "ok",
            "开机自启动",
            "配置已勾选开机启动；保存后会写入当前用户启动项。",
        );
    } else {
        push_diag(
            &mut items,
            "info",
            "开机自启动",
            "未启用。需要长期使用时建议打开。",
        );
    }

    items
}

fn push_diag(items: &mut Vec<DiagnosticItem>, level: &str, title: &str, message: &str) {
    items.push(DiagnosticItem {
        level: level.to_string(),
        title: title.to_string(),
        message: message.to_string(),
    });
}

fn check_url(items: &mut Vec<DiagnosticItem>, title: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        push_diag(items, "ok", title, trimmed);
    } else {
        push_diag(items, "error", title, "地址应以 http:// 或 https:// 开头。");
    }
}

fn push_game_limit_diag(items: &mut Vec<DiagnosticItem>, config: &AppConfig) {
    if config.game_download_limit_kib == 0 || config.game_upload_limit_kib == 0 {
        push_diag(
            items,
            "warn",
            "游戏中限速值",
            "下载/上传限速建议大于 0 KiB/s。",
        );
    } else {
        push_diag(
            items,
            "ok",
            "游戏中限速值",
            &format!(
                "游戏中会切换为下载 {} KiB/s、上传 {} KiB/s。",
                config.game_download_limit_kib, config.game_upload_limit_kib
            ),
        );
    }
}

fn base64_basic(username: &str, password: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = format!("{username}:{password}");
    let bytes = input.as_bytes();
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let b0 = bytes[index];
        let b1 = if index + 1 < bytes.len() {
            bytes[index + 1]
        } else {
            0
        };
        let b2 = if index + 2 < bytes.len() {
            bytes[index + 2]
        } else {
            0
        };
        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if index + 1 < bytes.len() {
            output.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if index + 2 < bytes.len() {
            output.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
        index += 3;
    }
    output
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

pub struct GameDetector {
    folders: Vec<String>,
    process_names: HashSet<String>,
    excluded_names: HashSet<String>,
    excluded_path_keywords: Vec<String>,
    path_cache: HashMap<u32, (String, String)>,
}

impl GameDetector {
    pub fn new(config: &AppConfig) -> Self {
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

    pub fn detect(&mut self) -> Option<String> {
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

pub fn build_detection_folders(config: &AppConfig) -> Vec<String> {
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

pub fn scan_game_libraries() -> Vec<String> {
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
            "Battle.net",
            "EA Games",
            "Ubisoft Game Launcher\\games",
        ] {
            let folder = Path::new(&root).join(relative);
            if folder.is_dir() {
                folders.insert(folder.to_string_lossy().to_string());
            }
        }
        read_steam_libraries(
            &Path::new(&root).join(r"SteamLibrary\steamapps\libraryfolders.vdf"),
            &mut folders,
        );
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

pub fn run_monitor_forever() {
    let mutex = unsafe { CreateMutexW(null(), 1, wide(MONITOR_MUTEX).as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return;
    }

    clear_stop_monitor();
    let mut config = load_config();
    let mut detector = GameDetector::new(&config);
    let mut controller = DownloadController::new(&config);
    let mut active_limit: Option<DownloadLimitSnapshot> = None;
    let mut had_state = false;
    let mut last_game_running = false;
    let mut last_detected = String::new();
    log_line(&config, "Monitor process started.");
    save_status(&MonitorStatus {
        running: true,
        game_running: false,
        detected: String::new(),
        message: "后台监控已启动。".into(),
        updated_at: now_unix(),
    });

    while !stop_monitor_requested() {
        let latest_config = load_config();
        if serde_json::to_string(&latest_config).ok() != serde_json::to_string(&config).ok() {
            config = latest_config;
            detector = GameDetector::new(&config);
            controller = DownloadController::new(&config);
            active_limit = None;
            log_line(&config, "Monitor reloaded config.");
        }

        let detected = detector.detect().unwrap_or_default();
        let game_running = !detected.is_empty();
        let mut message = if game_running {
            "检测到游戏运行。".to_string()
        } else {
            "未检测到游戏运行。".to_string()
        };

        if !had_state || game_running != last_game_running {
            let result = if game_running {
                controller
                    .enable_game_mode(&config)
                    .map(|(snapshot, text)| {
                        active_limit = Some(snapshot);
                        text
                    })
            } else if had_state {
                if let Some(snapshot) = active_limit.take() {
                    controller.restore(snapshot)
                } else {
                    Ok("检测到游戏退出，当前下载器无需恢复。".to_string())
                }
            } else {
                Ok("未检测到游戏运行。".to_string())
            };
            match result {
                Ok(text) => message = text,
                Err(error) => {
                    message = format!("监控出错：{error}");
                    log_line(&config, &message);
                }
            }
            had_state = true;
            last_game_running = game_running;
        }

        if detected != last_detected {
            last_detected = detected.clone();
        }
        save_status(&MonitorStatus {
            running: true,
            game_running,
            detected,
            message,
            updated_at: now_unix(),
        });

        for _ in 0..config.check_interval_seconds.max(1) * 10 {
            if stop_monitor_requested() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    if config.restore_on_exit {
        if let Some(snapshot) = active_limit.take() {
            let _ = controller.restore(snapshot);
        }
    }
    clear_stop_monitor();
    save_status(&MonitorStatus {
        running: false,
        game_running: false,
        detected: String::new(),
        message: "后台监控已停止。".into(),
        updated_at: now_unix(),
    });
    log_line(&config, "Monitor process stopped.");
    if mutex != null_mut() {
        unsafe { CloseHandle(mutex) };
    }
}
