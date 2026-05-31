import csv
import json
import signal
import subprocess
import sys
import threading
import time
import tkinter as tk
from datetime import datetime
from pathlib import Path
from tkinter import filedialog, messagebox, ttk
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import HTTPCookieProcessor, Request, build_opener


DEFAULT_CONFIG = {
    "qbee_url": "http://127.0.0.1:8080",
    "username": "admin",
    "password": "adminadmin",
    "game_folders": [
        "C:\\Program Files (x86)\\Steam\\steamapps",
        "D:\\SteamLibrary\\steamapps"
    ],
    "game_processes": [],
    "exclude_processes": [
        "steam.exe",
        "epicgameslauncher.exe",
        "goggalaxy.exe",
        "wegame.exe",
        "battle.net.exe"
    ],
    "check_interval_seconds": 3,
    "restore_on_exit": True,
    "log_file": "qbee_game_speed_limiter.log"
}


CONFIG_PATH = Path(__file__).with_name("qbee_game_speed_limiter.json")
STOPPING = False


def load_config():
    if not CONFIG_PATH.exists():
        save_config(DEFAULT_CONFIG)
        return DEFAULT_CONFIG.copy()

    with CONFIG_PATH.open("r", encoding="utf-8") as file:
        config = json.load(file)

    merged = DEFAULT_CONFIG.copy()
    merged.update(config)
    return merged


def save_config(config):
    CONFIG_PATH.write_text(
        json.dumps(config, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )


def normalize_base_url(url):
    return url.rstrip("/")


def normalize_windows_path(path_text):
    return str(Path(path_text).expanduser()).rstrip("\\/").lower()


def is_under_folder(file_path, folder_path):
    file_path = normalize_windows_path(file_path)
    folder_path = normalize_windows_path(folder_path)
    return file_path == folder_path or file_path.startswith(folder_path + "\\")


def log(config, message):
    line = f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {message}"
    print(line)
    log_path = Path(__file__).with_name(config["log_file"])
    with log_path.open("a", encoding="utf-8") as file:
        file.write(line + "\n")


class QbeeClient:
    def __init__(self, base_url, username, password):
        self.base_url = normalize_base_url(base_url)
        self.username = username
        self.password = password
        self.opener = build_opener(HTTPCookieProcessor())
        self.logged_in = False

    def request(self, method, path, data=None, retry_login=True):
        body = None
        headers = {
            "Referer": self.base_url,
            "User-Agent": "qbee-game-speed-limiter/3.0",
        }

        if data is not None:
            body = urlencode(data).encode("utf-8")
            headers["Content-Type"] = "application/x-www-form-urlencoded"

        request = Request(
            self.base_url + path,
            data=body,
            headers=headers,
            method=method,
        )

        try:
            with self.opener.open(request, timeout=10) as response:
                return response.read().decode("utf-8", errors="replace")
        except HTTPError as error:
            if error.code in (401, 403) and retry_login:
                self.login()
                return self.request(method, path, data, retry_login=False)
            raise

    def login(self):
        result = self.request(
            "POST",
            "/api/v2/auth/login",
            {"username": self.username, "password": self.password},
            retry_login=False,
        ).strip()
        if result != "Ok.":
            raise RuntimeError("qbee login failed. Check Web UI username and password.")
        self.logged_in = True

    def speed_limits_enabled(self):
        if not self.logged_in:
            self.login()
        result = self.request("GET", "/api/v2/transfer/speedLimitsMode").strip()
        return result == "1"

    def set_speed_limits(self, enabled):
        current = self.speed_limits_enabled()
        if current == enabled:
            return False
        self.request("POST", "/api/v2/transfer/toggleSpeedLimitsMode")
        return True


def query_running_processes():
    command = [
        "powershell",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        "Get-CimInstance Win32_Process | "
        "Select-Object Name,ExecutablePath | "
        "ConvertTo-Csv -NoTypeInformation",
    ]
    result = subprocess.run(
        command,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8-sig",
        errors="replace",
        creationflags=subprocess.CREATE_NO_WINDOW,
    )
    return list(csv.DictReader(result.stdout.splitlines()))


def detect_running_game(config):
    folders = [folder for folder in config.get("game_folders", []) if folder]
    process_names = {name.lower() for name in config.get("game_processes", [])}
    excluded = {name.lower() for name in config.get("exclude_processes", [])}

    for process in query_running_processes():
        name = (process.get("Name") or "").lower()
        path = process.get("ExecutablePath") or ""

        if not name or name in excluded:
            continue
        if name in process_names:
            return name
        if path and any(is_under_folder(path, folder) for folder in folders):
            return path

    return None


def handle_stop(_signum, _frame):
    global STOPPING
    STOPPING = True


def run_monitor():
    signal.signal(signal.SIGINT, handle_stop)
    signal.signal(signal.SIGTERM, handle_stop)

    config = load_config()
    client = QbeeClient(config["qbee_url"], config["username"], config["password"])
    interval = max(1, int(config["check_interval_seconds"]))
    last_game_state = None

    log(config, "Monitor started.")
    log(config, "Game folders: " + ", ".join(config.get("game_folders", [])))

    try:
        while not STOPPING:
            detected_game = detect_running_game(config)
            game_running = detected_game is not None

            if game_running != last_game_state:
                if game_running:
                    changed = client.set_speed_limits(True)
                    suffix = f" Detected: {detected_game}"
                    message = "Game running. Enabled alternative speed limits."
                    if not changed:
                        message = "Game running. Alternative speed limits already enabled."
                    log(config, message + suffix)
                elif last_game_state is not None:
                    changed = client.set_speed_limits(False)
                    message = "Game closed. Disabled alternative speed limits."
                    if not changed:
                        message = "Game closed. Alternative speed limits already disabled."
                    log(config, message)
                last_game_state = game_running

            time.sleep(interval)
    except (HTTPError, URLError, RuntimeError, subprocess.SubprocessError) as error:
        log(config, f"Error: {error}")
        return 1
    finally:
        if config.get("restore_on_exit", True):
            try:
                client.set_speed_limits(False)
                log(config, "Program exited. Tried to disable alternative speed limits.")
            except Exception as error:
                log(config, f"Failed to disable alternative speed limits on exit: {error}")

    return 0


class SettingsApp:
    def __init__(self, root):
        self.root = root
        self.root.title("qbee Game Speed Limiter")
        self.root.geometry("720x560")
        self.root.minsize(680, 520)
        self.config = load_config()
        self.monitor_process = None

        self.qbee_url = tk.StringVar(value=self.config.get("qbee_url", ""))
        self.username = tk.StringVar(value=self.config.get("username", ""))
        self.password = tk.StringVar(value=self.config.get("password", ""))
        self.interval = tk.IntVar(value=int(self.config.get("check_interval_seconds", 3)))
        self.status = tk.StringVar(value="Ready")

        self.build_ui()
        self.refresh_folders()
        self.root.protocol("WM_DELETE_WINDOW", self.on_close)

    def build_ui(self):
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)

        frame = ttk.Frame(self.root, padding=18)
        frame.grid(row=0, column=0, sticky="nsew")
        frame.columnconfigure(0, weight=1)
        frame.rowconfigure(1, weight=1)

        account = ttk.LabelFrame(frame, text="qbee Web UI", padding=12)
        account.grid(row=0, column=0, sticky="ew")
        account.columnconfigure(1, weight=1)

        ttk.Label(account, text="地址").grid(row=0, column=0, sticky="w", padx=(0, 8), pady=4)
        ttk.Entry(account, textvariable=self.qbee_url).grid(row=0, column=1, sticky="ew", pady=4)

        ttk.Label(account, text="用户名").grid(row=1, column=0, sticky="w", padx=(0, 8), pady=4)
        ttk.Entry(account, textvariable=self.username).grid(row=1, column=1, sticky="ew", pady=4)

        ttk.Label(account, text="密码").grid(row=2, column=0, sticky="w", padx=(0, 8), pady=4)
        ttk.Entry(account, textvariable=self.password, show="*").grid(row=2, column=1, sticky="ew", pady=4)

        interval_row = ttk.Frame(account)
        interval_row.grid(row=3, column=1, sticky="w", pady=(8, 0))
        ttk.Label(interval_row, text="检查间隔").pack(side="left")
        ttk.Spinbox(interval_row, from_=1, to=60, textvariable=self.interval, width=6).pack(side="left", padx=8)
        ttk.Label(interval_row, text="秒").pack(side="left")
        ttk.Button(interval_row, text="测试连接", command=self.test_connection).pack(side="left", padx=(18, 0))

        folders = ttk.LabelFrame(frame, text="游戏库文件夹", padding=12)
        folders.grid(row=1, column=0, sticky="nsew", pady=14)
        folders.columnconfigure(0, weight=1)
        folders.rowconfigure(0, weight=1)

        self.folder_list = tk.Listbox(folders, height=8, activestyle="dotbox")
        self.folder_list.grid(row=0, column=0, sticky="nsew")
        scrollbar = ttk.Scrollbar(folders, orient="vertical", command=self.folder_list.yview)
        scrollbar.grid(row=0, column=1, sticky="ns")
        self.folder_list.configure(yscrollcommand=scrollbar.set)

        folder_buttons = ttk.Frame(folders)
        folder_buttons.grid(row=0, column=2, sticky="ns", padx=(12, 0))
        ttk.Button(folder_buttons, text="添加", command=self.add_folder).pack(fill="x")
        ttk.Button(folder_buttons, text="删除", command=self.remove_folder).pack(fill="x", pady=8)
        ttk.Button(folder_buttons, text="打开配置", command=self.open_config_folder).pack(fill="x")

        actions = ttk.Frame(frame)
        actions.grid(row=2, column=0, sticky="ew")
        actions.columnconfigure(0, weight=1)

        ttk.Label(actions, textvariable=self.status).grid(row=0, column=0, sticky="w")
        ttk.Button(actions, text="保存配置", command=self.save_from_ui).grid(row=0, column=1, padx=6)
        self.start_button = ttk.Button(actions, text="开始监控", command=self.start_monitor)
        self.start_button.grid(row=0, column=2, padx=6)
        self.stop_button = ttk.Button(actions, text="停止监控", command=self.stop_monitor, state="disabled")
        self.stop_button.grid(row=0, column=3)

    def refresh_folders(self):
        self.folder_list.delete(0, tk.END)
        for folder in self.config.get("game_folders", []):
            self.folder_list.insert(tk.END, folder)

    def collect_folders(self):
        return [self.folder_list.get(index) for index in range(self.folder_list.size())]

    def save_from_ui(self):
        try:
            interval = max(1, int(self.interval.get()))
        except (tk.TclError, ValueError):
            interval = 3
            self.interval.set(interval)

        self.config["qbee_url"] = self.qbee_url.get().strip()
        self.config["username"] = self.username.get().strip()
        self.config["password"] = self.password.get()
        self.config["game_folders"] = self.collect_folders()
        self.config["check_interval_seconds"] = interval
        save_config(self.config)
        self.status.set("Saved")

    def add_folder(self):
        folder = filedialog.askdirectory(title="选择游戏库文件夹")
        if not folder:
            return
        current = {item.lower() for item in self.collect_folders()}
        if folder.lower() not in current:
            self.folder_list.insert(tk.END, folder)
            self.status.set("Folder added")

    def remove_folder(self):
        selected = list(self.folder_list.curselection())
        for index in reversed(selected):
            self.folder_list.delete(index)
        if selected:
            self.status.set("Folder removed")

    def open_config_folder(self):
        subprocess.Popen(["explorer", str(CONFIG_PATH.parent)])

    def test_connection(self):
        self.save_from_ui()
        self.status.set("Testing qbee connection...")
        threading.Thread(target=self._test_connection_worker, daemon=True).start()

    def _test_connection_worker(self):
        try:
            client = QbeeClient(
                self.config["qbee_url"],
                self.config["username"],
                self.config["password"],
            )
            enabled = client.speed_limits_enabled()
            text = "Connection OK. Alternative speed limits are "
            text += "enabled." if enabled else "disabled."
            self.root.after(0, lambda: self.status.set(text))
        except Exception as error:
            message = str(error)
            self.root.after(0, lambda: messagebox.showerror("连接失败", message))
            self.root.after(0, lambda: self.status.set("Connection failed"))

    def start_monitor(self):
        self.save_from_ui()
        if self.monitor_process and self.monitor_process.poll() is None:
            return

        command = self.monitor_command()
        self.monitor_process = subprocess.Popen(
            command,
            cwd=str(Path(__file__).parent),
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        self.status.set("Monitoring")
        self.start_button.configure(state="disabled")
        self.stop_button.configure(state="normal")
        self.root.after(1000, self.check_monitor)

    def monitor_command(self):
        if getattr(sys, "frozen", False):
            return [sys.executable, "--monitor"]
        return [sys.executable, str(Path(__file__)), "--monitor"]

    def check_monitor(self):
        if not self.monitor_process:
            return
        if self.monitor_process.poll() is None:
            self.root.after(1000, self.check_monitor)
            return
        self.monitor_process = None
        self.status.set("Monitor stopped")
        self.start_button.configure(state="normal")
        self.stop_button.configure(state="disabled")

    def stop_monitor(self):
        if not self.monitor_process or self.monitor_process.poll() is not None:
            return
        self.monitor_process.terminate()
        try:
            self.monitor_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.monitor_process.kill()
        self.monitor_process = None
        self.status.set("Monitor stopped")
        self.start_button.configure(state="normal")
        self.stop_button.configure(state="disabled")

    def on_close(self):
        if self.monitor_process and self.monitor_process.poll() is None:
            if not messagebox.askyesno("退出", "监控仍在运行。要停止监控并退出吗？"):
                return
            self.stop_monitor()
        self.root.destroy()


def run_gui():
    root = tk.Tk()
    SettingsApp(root)
    root.mainloop()
    return 0


def main():
    if "--monitor" in sys.argv:
        return run_monitor()
    return run_gui()


if __name__ == "__main__":
    raise SystemExit(main())
