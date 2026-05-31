import csv
import json
import signal
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
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


def load_or_create_config():
    if not CONFIG_PATH.exists():
        CONFIG_PATH.write_text(
            json.dumps(DEFAULT_CONFIG, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        print(f"Created config file: {CONFIG_PATH}")
        print("Edit qbee URL, account, password, and game_folders, then run again.")
        sys.exit(0)

    with CONFIG_PATH.open("r", encoding="utf-8") as file:
        config = json.load(file)

    merged = DEFAULT_CONFIG.copy()
    merged.update(config)
    return merged


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
            "User-Agent": "qbee-game-speed-limiter/2.0",
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


def main():
    signal.signal(signal.SIGINT, handle_stop)
    signal.signal(signal.SIGTERM, handle_stop)

    config = load_or_create_config()
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
                    log(config, ("Game running. Enabled alternative speed limits." if changed else "Game running. Alternative speed limits already enabled.") + suffix)
                elif last_game_state is not None:
                    changed = client.set_speed_limits(False)
                    log(config, "Game closed. Disabled alternative speed limits." if changed else "Game closed. Alternative speed limits already disabled.")
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


if __name__ == "__main__":
    raise SystemExit(main())
