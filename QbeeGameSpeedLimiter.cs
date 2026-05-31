using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Management;
using System.Net;
using Microsoft.Win32;
using System.Text.RegularExpressions;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace QbeeGameSpeedLimiter
{
    public class AppConfig
    {
        public string qbee_url { get; set; }
        public string username { get; set; }
        public string password { get; set; }
        public List<string> game_folders { get; set; }
        public List<string> game_processes { get; set; }
        public List<string> exclude_processes { get; set; }
        public List<string> exclude_path_keywords { get; set; }
        public List<string> exclude_steam_app_keywords { get; set; }
        public int check_interval_seconds { get; set; }
        public bool restore_on_exit { get; set; }
        public bool start_with_windows { get; set; }
        public bool auto_start_monitor { get; set; }
        public string log_file { get; set; }

        public static AppConfig Default()
        {
            return new AppConfig
            {
                qbee_url = "http://127.0.0.1:8080",
                username = "admin",
                password = "adminadmin",
                game_folders = new List<string>
                {
                    @"C:\Program Files (x86)\Steam\steamapps",
                    @"D:\SteamLibrary\steamapps"
                },
                game_processes = new List<string>(),
                exclude_processes = new List<string>
                {
                    "steam.exe",
                    "steamservice.exe",
                    "steamwebhelper.exe",
                    "wallpaper32.exe",
                    "wallpaper64.exe",
                    "wallpaper_engine.exe",
                    "epicgameslauncher.exe",
                    "goggalaxy.exe",
                    "wegame.exe",
                    "battle.net.exe"
                },
                exclude_path_keywords = new List<string>
                {
                    @"\steamapps\common\steamworks shared\",
                    @"\steamapps\common\proton ",
                    @"\steamapps\common\steam linux runtime",
                    @"\_commonredist\",
                    @"\redist\",
                    @"\redistributable\",
                    @"\installer\",
                    @"\uninstall\",
                    @"\launcher\",
                    @"\wallpaper_engine\"
                },
                exclude_steam_app_keywords = new List<string>
                {
                    "wallpaper",
                    "dedicated server",
                    "server tool",
                    "server dedicated",
                    "sdk",
                    "tool",
                    "tools",
                    "benchmark",
                    "editor",
                    "modding",
                    "workshop",
                    "proton",
                    "redistributable",
                    "runtime"
                },
                check_interval_seconds = 5,
                restore_on_exit = true,
                start_with_windows = false,
                auto_start_monitor = false,
                log_file = "qbee_game_speed_limiter.log"
            };
        }
    }

    public static class ConfigStore
    {
        public static readonly string AppDirectory = AppDomain.CurrentDomain.BaseDirectory;
        public static readonly string ConfigPath = Path.Combine(AppDirectory, "qbee_game_speed_limiter.json");

        public static AppConfig Load()
        {
            if (!File.Exists(ConfigPath))
            {
                var created = AppConfig.Default();
                Save(created);
                return created;
            }

            var serializer = new JavaScriptSerializer();
            var config = serializer.Deserialize<AppConfig>(File.ReadAllText(ConfigPath, Encoding.UTF8));
            var defaults = AppConfig.Default();

            if (config == null) return defaults;
            if (string.IsNullOrWhiteSpace(config.qbee_url)) config.qbee_url = defaults.qbee_url;
            if (config.username == null) config.username = defaults.username;
            if (config.password == null) config.password = defaults.password;
            if (config.game_folders == null) config.game_folders = defaults.game_folders;
            if (config.game_processes == null) config.game_processes = defaults.game_processes;
            if (config.exclude_processes == null) config.exclude_processes = defaults.exclude_processes;
            if (config.exclude_path_keywords == null) config.exclude_path_keywords = defaults.exclude_path_keywords;
            if (config.exclude_steam_app_keywords == null) config.exclude_steam_app_keywords = defaults.exclude_steam_app_keywords;
            if (config.check_interval_seconds < 1) config.check_interval_seconds = defaults.check_interval_seconds;
            if (string.IsNullOrWhiteSpace(config.log_file)) config.log_file = defaults.log_file;

            return config;
        }

        public static void Save(AppConfig config)
        {
            var serializer = new JavaScriptSerializer();
            var json = serializer.Serialize(config);
            File.WriteAllText(ConfigPath, PrettyJson(json), Encoding.UTF8);
        }

        private static string PrettyJson(string json)
        {
            var output = new StringBuilder();
            var indent = 0;
            var inString = false;

            foreach (var ch in json)
            {
                if (ch == '"' && (output.Length == 0 || output[output.Length - 1] != '\\'))
                {
                    inString = !inString;
                }

                if (inString)
                {
                    output.Append(ch);
                    continue;
                }

                if (ch == '{' || ch == '[')
                {
                    output.Append(ch).AppendLine();
                    indent++;
                    output.Append(new string(' ', indent * 2));
                }
                else if (ch == '}' || ch == ']')
                {
                    output.AppendLine();
                    indent--;
                    output.Append(new string(' ', indent * 2)).Append(ch);
                }
                else if (ch == ',')
                {
                    output.Append(ch).AppendLine();
                    output.Append(new string(' ', indent * 2));
                }
                else if (ch == ':')
                {
                    output.Append(": ");
                }
                else
                {
                    output.Append(ch);
                }
            }

            return output.ToString();
        }
    }

    public class QbeeClient
    {
        private string baseUrl;
        private readonly string username;
        private readonly string password;
        private readonly CookieContainer cookies = new CookieContainer();
        private bool loggedIn;

        public QbeeClient(AppConfig config)
        {
            baseUrl = (config.qbee_url ?? "").TrimEnd('/');
            username = config.username ?? "";
            password = config.password ?? "";
        }

        public bool SpeedLimitsEnabled()
        {
            EnsureLogin();
            var result = Request("GET", "/api/v2/transfer/speedLimitsMode", null).Trim();
            return result == "1";
        }

        public bool SetSpeedLimits(bool enabled)
        {
            var current = SpeedLimitsEnabled();
            if (current == enabled) return false;
            Request("POST", "/api/v2/transfer/toggleSpeedLimitsMode", "");
            return true;
        }

        private void EnsureLogin()
        {
            if (loggedIn) return;
            ValidateServer();

            if (CanUseWithoutLogin())
            {
                loggedIn = true;
                return;
            }

            var body = "username=" + Uri.EscapeDataString(username) +
                       "&password=" + Uri.EscapeDataString(password);
            var result = Request("POST", "/api/v2/auth/login", body).Trim();
            if (result != "Ok.")
            {
                throw new InvalidOperationException("qbee 登录失败，请检查 Web UI 用户名和密码。");
            }
            loggedIn = true;
        }

        private void ValidateServer()
        {
            try
            {
                var content = Request("GET", "/", null);
                if (content.IndexOf("CEF remote debugging", StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    if (TrySwitchToIpv6Loopback())
                    {
                        return;
                    }

                    throw new InvalidOperationException(
                        "当前地址打开的是 CEF remote debugging，不是 qBittorrent Web UI。Steam 的 CEF 可能占用了 127.0.0.1:8080。请尝试填写 http://[::1]:8080，或在 qbee 设置里换一个 Web UI 端口。");
                }
            }
            catch (WebException error)
            {
                var response = error.Response as HttpWebResponse;
                if (response == null)
                {
                    throw;
                }
            }
        }

        private bool TrySwitchToIpv6Loopback()
        {
            Uri uri;
            if (!Uri.TryCreate(baseUrl, UriKind.Absolute, out uri))
            {
                return false;
            }

            var host = uri.Host.ToLowerInvariant();
            if (host != "localhost" && host != "127.0.0.1")
            {
                return false;
            }

            var builder = new UriBuilder(uri)
            {
                Host = "::1"
            };
            var candidate = builder.Uri.ToString().TrimEnd('/');
            var original = baseUrl;

            try
            {
                baseUrl = candidate;
                Request("GET", "/api/v2/app/version", null);
                return true;
            }
            catch
            {
                baseUrl = original;
                return false;
            }
        }

        private bool CanUseWithoutLogin()
        {
            try
            {
                Request("GET", "/api/v2/app/version", null);
                return true;
            }
            catch (WebException error)
            {
                var response = error.Response as HttpWebResponse;
                if (response != null && response.StatusCode == HttpStatusCode.NotFound)
                {
                    throw new InvalidOperationException(
                        "当前地址没有 qBittorrent Web API。请确认填的是 qbee Web UI 地址，例如 http://[::1]:8080 或 http://127.0.0.1:8081。");
                }
                return false;
            }
        }

        private string Request(string method, string path, string body)
        {
            if (string.IsNullOrWhiteSpace(baseUrl))
            {
                throw new InvalidOperationException("qbee Web UI 地址不能为空。");
            }

            var request = (HttpWebRequest)WebRequest.Create(baseUrl + path);
            request.Method = method;
            request.CookieContainer = cookies;
            request.Referer = baseUrl;
            request.UserAgent = "qbee-game-speed-limiter/3.0";
            request.Timeout = 5000;
            request.ReadWriteTimeout = 5000;

            if (body != null)
            {
                var bytes = Encoding.UTF8.GetBytes(body);
                request.ContentType = "application/x-www-form-urlencoded";
                request.ContentLength = bytes.Length;
                using (var stream = request.GetRequestStream())
                {
                    stream.Write(bytes, 0, bytes.Length);
                }
            }

            using (var response = (HttpWebResponse)request.GetResponse())
            using (var stream = response.GetResponseStream())
            using (var reader = new StreamReader(stream, Encoding.UTF8))
            {
                return reader.ReadToEnd();
            }
        }
    }

    public class GameMonitor
    {
        private readonly Action<string> onStatus;
        private readonly Action<string> onDetectedGame;
        private readonly Action onStopped;
        private readonly ManualResetEvent stopSignal = new ManualResetEvent(false);
        private Thread worker;
        private bool stopping;

        public GameMonitor(Action<string> onStatus, Action<string> onDetectedGame, Action onStopped)
        {
            this.onStatus = onStatus;
            this.onDetectedGame = onDetectedGame;
            this.onStopped = onStopped;
        }

        public bool IsRunning
        {
            get { return worker != null && worker.IsAlive; }
        }

        public void Start(AppConfig config)
        {
            if (IsRunning) return;
            stopping = false;
            stopSignal.Reset();
            worker = new Thread(() => Run(config));
            worker.IsBackground = true;
            worker.Start();
        }

        public void Stop()
        {
            stopping = true;
            stopSignal.Set();
            if (worker != null && worker.IsAlive)
            {
                worker.Join(2000);
            }
        }

        private void Run(AppConfig config)
        {
            var client = new QbeeClient(config);
            var detector = new GameProcessDetector(config);
            bool? lastGameState = null;
            var appEnabledSpeedLimits = false;
            var lastDetectedGame = "";
            var lastPublishedDetectedGame = "";
            var lastStillRunningLog = DateTime.MinValue;

            Log(config, "Monitor started.");
            Log(config, "Game folders: " + string.Join(", ", config.game_folders ?? new List<string>()));
            onStatus("监控中");

            try
            {
                while (!stopping)
                {
                    var detectedGame = detector.Detect();
                    var gameRunning = detectedGame != null;
                    if ((detectedGame ?? "") != lastPublishedDetectedGame)
                    {
                        lastPublishedDetectedGame = detectedGame ?? "";
                        onDetectedGame(lastPublishedDetectedGame);
                    }

                    if (lastGameState == null || gameRunning != lastGameState.Value)
                    {
                        if (gameRunning)
                        {
                            var alreadyEnabled = client.SpeedLimitsEnabled();
                            if (!alreadyEnabled)
                            {
                                client.SetSpeedLimits(true);
                                appEnabledSpeedLimits = true;
                            }
                            else
                            {
                                appEnabledSpeedLimits = false;
                            }

                            var message = alreadyEnabled ? "检测到游戏运行，备用速度限制原本已打开，本次不会在退出后强制关闭。" : "检测到游戏运行，已打开备用速度限制。";
                            Log(config, message + " Detected: " + detectedGame);
                            onStatus(message);
                        }
                        else if (lastGameState != null)
                        {
                            var changed = false;
                            if (appEnabledSpeedLimits)
                            {
                                changed = client.SetSpeedLimits(false);
                            }

                            var message = appEnabledSpeedLimits
                                ? (changed ? "检测到游戏退出，已关闭备用速度限制。" : "检测到游戏退出，备用速度限制已是关闭状态。")
                                : "检测到游戏退出，保留原本已打开的备用速度限制。";
                            Log(config, message);
                            onStatus(message);
                            appEnabledSpeedLimits = false;
                        }

                        lastGameState = gameRunning;
                        lastDetectedGame = detectedGame ?? "";
                        lastStillRunningLog = DateTime.Now;
                    }
                    else if (gameRunning && detectedGame != lastDetectedGame)
                    {
                        lastDetectedGame = detectedGame ?? "";
                        Log(config, "Still detecting game process. Detected: " + lastDetectedGame);
                        lastStillRunningLog = DateTime.Now;
                    }
                    else if (gameRunning && (DateTime.Now - lastStillRunningLog).TotalMinutes >= 5)
                    {
                        Log(config, "Still detecting game process. Detected: " + detectedGame);
                        lastStillRunningLog = DateTime.Now;
                    }

                    stopSignal.WaitOne(Math.Max(1, config.check_interval_seconds) * 1000);
                }
            }
            catch (Exception error)
            {
                Log(config, "Error: " + error.Message);
                onStatus("监控出错：" + error.Message);
            }
            finally
            {
                onDetectedGame("");
                if (config.restore_on_exit && appEnabledSpeedLimits)
                {
                    try
                    {
                        client.SetSpeedLimits(false);
                        Log(config, "Program exited. Disabled alternative speed limits that this app enabled.");
                    }
                    catch (Exception error)
                    {
                        Log(config, "Failed to disable alternative speed limits on exit: " + error.Message);
                    }
                }
                onStopped();
            }
        }

        private class GameProcessDetector
        {
            private readonly List<string> folders;
            private readonly HashSet<string> processNames;
            private readonly HashSet<string> excluded;
            private readonly List<string> excludedPathKeywords;
            private readonly Dictionary<int, string> pathCache = new Dictionary<int, string>();
            private readonly Dictionary<int, string> nameCache = new Dictionary<int, string>();

            public GameProcessDetector(AppConfig config)
            {
                folders = BuildDetectionFolders(config);
                processNames = new HashSet<string>(
                    (config.game_processes ?? new List<string>())
                    .Where(item => !string.IsNullOrWhiteSpace(item))
                    .Select(item => item.ToLowerInvariant()));
                excluded = new HashSet<string>(
                    (config.exclude_processes ?? new List<string>())
                    .Where(item => !string.IsNullOrWhiteSpace(item))
                    .Select(item => item.ToLowerInvariant()));
                excludedPathKeywords = (config.exclude_path_keywords ?? new List<string>())
                    .Where(item => !string.IsNullOrWhiteSpace(item))
                    .Select(item => item.ToLowerInvariant())
                    .ToList();
            }

            public string Detect()
            {
                Process[] processes;
                try
                {
                    processes = Process.GetProcesses();
                }
                catch
                {
                    return null;
                }

                var liveIds = new HashSet<int>();

                foreach (var process in processes)
                {
                    using (process)
                    {
                        string name;
                        try
                        {
                            liveIds.Add(process.Id);
                            name = (process.ProcessName ?? "").ToLowerInvariant();
                        }
                        catch
                        {
                            continue;
                        }

                        if (string.IsNullOrWhiteSpace(name)) continue;
                        var exeName = name.EndsWith(".exe") ? name : name + ".exe";
                        if (excluded.Contains(exeName) || excluded.Contains(name)) continue;
                        if (processNames.Contains(exeName) || processNames.Contains(name)) return exeName;

                        var path = GetExecutablePath(process, exeName);
                        if (string.IsNullOrWhiteSpace(path)) continue;

                        var normalizedPath = NormalizePath(path);
                        if (excludedPathKeywords.Any(keyword => normalizedPath.Contains(keyword))) continue;
                        if (folders.Any(folder => IsUnderFolder(path, folder))) return path;
                    }
                }

                RemoveDeadCacheEntries(liveIds);
                return null;
            }

            private string GetExecutablePath(Process process, string exeName)
            {
                string cached;
                string cachedName;
                if (pathCache.TryGetValue(process.Id, out cached) &&
                    nameCache.TryGetValue(process.Id, out cachedName) &&
                    cachedName == exeName)
                {
                    return cached;
                }

                var path = "";
                try
                {
                    if (process.MainModule != null)
                    {
                        path = process.MainModule.FileName;
                    }
                }
                catch
                {
                    path = GetExecutablePathByWmi(process.Id);
                }

                pathCache[process.Id] = path ?? "";
                nameCache[process.Id] = exeName;
                return path;
            }

            private void RemoveDeadCacheEntries(HashSet<int> liveIds)
            {
                if (pathCache.Count < 256) return;

                foreach (var id in pathCache.Keys.ToList())
                {
                    if (!liveIds.Contains(id))
                    {
                        pathCache.Remove(id);
                        nameCache.Remove(id);
                    }
                }
            }
        }

        private static string GetExecutablePathByWmi(int processId)
        {
            try
            {
                using (var searcher = new ManagementObjectSearcher("SELECT ExecutablePath FROM Win32_Process WHERE ProcessId = " + processId))
                {
                    foreach (ManagementObject process in searcher.Get())
                    {
                        return Convert.ToString(process["ExecutablePath"] ?? "");
                    }
                }
            }
            catch
            {
            }

            return null;
        }

        private static List<string> BuildDetectionFolders(AppConfig config)
        {
            var configured = (config.game_folders ?? new List<string>())
                .Where(item => !string.IsNullOrWhiteSpace(item))
                .Select(NormalizePath)
                .ToList();

            var steamAppKeywords = (config.exclude_steam_app_keywords ?? new List<string>())
                .Where(item => !string.IsNullOrWhiteSpace(item))
                .Select(item => item.ToLowerInvariant())
                .ToList();

            var expanded = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (var folder in configured)
            {
                if (Path.GetFileName(folder).Equals("steamapps", StringComparison.OrdinalIgnoreCase))
                {
                    foreach (var appFolder in SteamInstalledAppFolders(folder, steamAppKeywords))
                    {
                        expanded.Add(appFolder);
                    }
                }
                else
                {
                    expanded.Add(folder);
                }
            }

            return expanded.ToList();
        }

        private static IEnumerable<string> SteamInstalledAppFolders(string steamappsFolder, List<string> excludeKeywords)
        {
            foreach (var manifest in Directory.GetFiles(steamappsFolder, "appmanifest_*.acf"))
            {
                string text;
                try
                {
                    text = File.ReadAllText(manifest);
                }
                catch
                {
                    continue;
                }

                var name = ExtractAcfValue(text, "name");
                var installDir = ExtractAcfValue(text, "installdir");
                if (string.IsNullOrWhiteSpace(installDir)) continue;

                var haystack = (name + " " + installDir).ToLowerInvariant();
                if (excludeKeywords.Any(keyword => haystack.Contains(keyword))) continue;

                var appFolder = Path.Combine(steamappsFolder, "common", installDir);
                if (Directory.Exists(appFolder)) yield return NormalizePath(appFolder);
            }
        }

        private static string ExtractAcfValue(string text, string key)
        {
            var match = Regex.Match(text, "\"" + Regex.Escape(key) + "\"\\s+\"([^\"]*)\"", RegexOptions.IgnoreCase);
            return match.Success ? match.Groups[1].Value.Replace(@"\\", @"\") : "";
        }

        private static bool IsUnderFolder(string filePath, string folderPath)
        {
            var normalizedFile = NormalizePath(filePath);
            return normalizedFile == folderPath || normalizedFile.StartsWith(folderPath + "\\");
        }

        private static string NormalizePath(string value)
        {
            return Path.GetFullPath(Environment.ExpandEnvironmentVariables(value.Trim())).TrimEnd('\\', '/').ToLowerInvariant();
        }

        private static void Log(AppConfig config, string message)
        {
            var line = "[" + DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss") + "] " + message;
            var path = Path.Combine(ConfigStore.AppDirectory, config.log_file ?? "qbee_game_speed_limiter.log");
            File.AppendAllText(path, line + Environment.NewLine, Encoding.UTF8);
        }
    }

    public static class GameLibraryScanner
    {
        public static List<string> Scan()
        {
            var folders = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            AddSteamLibraries(folders);
            AddEpicLibraries(folders);
            AddCommonFolders(folders);
            return folders
                .Where(Directory.Exists)
                .OrderBy(item => item, StringComparer.OrdinalIgnoreCase)
                .ToList();
        }

        private static void AddSteamLibraries(HashSet<string> folders)
        {
            var candidates = new List<string>
            {
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), @"Steam\steamapps\libraryfolders.vdf"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), @"Steam\steamapps\libraryfolders.vdf")
            };

            foreach (var drive in DriveInfo.GetDrives().Where(drive => drive.IsReady))
            {
                candidates.Add(Path.Combine(drive.RootDirectory.FullName, @"SteamLibrary\steamapps\libraryfolders.vdf"));
            }

            foreach (var file in candidates.Where(File.Exists))
            {
                var steamapps = Path.GetDirectoryName(file);
                if (!string.IsNullOrWhiteSpace(steamapps)) folders.Add(steamapps);

                var text = File.ReadAllText(file);
                foreach (Match match in Regex.Matches(text, "\"path\"\\s+\"([^\"]+)\"", RegexOptions.IgnoreCase))
                {
                    var libraryRoot = match.Groups[1].Value.Replace(@"\\", @"\");
                    folders.Add(Path.Combine(libraryRoot, "steamapps"));
                }
            }
        }

        private static void AddEpicLibraries(HashSet<string> folders)
        {
            var manifestDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
                @"Epic\EpicGamesLauncher\Data\Manifests");
            if (!Directory.Exists(manifestDir)) return;

            foreach (var file in Directory.GetFiles(manifestDir, "*.item"))
            {
                var text = File.ReadAllText(file);
                var match = Regex.Match(text, "\"InstallLocation\"\\s*:\\s*\"([^\"]+)\"", RegexOptions.IgnoreCase);
                if (match.Success)
                {
                    folders.Add(match.Groups[1].Value.Replace(@"\\", @"\"));
                }
            }
        }

        private static void AddCommonFolders(HashSet<string> folders)
        {
            foreach (var drive in DriveInfo.GetDrives().Where(drive => drive.IsReady))
            {
                var root = drive.RootDirectory.FullName;
                foreach (var relative in new[]
                {
                    @"SteamLibrary\steamapps",
                    @"Games",
                    @"Epic Games",
                    @"GOG Games",
                    @"WeGameApps",
                    @"XboxGames",
                    @"Battle.net"
                })
                {
                    folders.Add(Path.Combine(root, relative));
                }
            }

            folders.Add(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), @"Steam\steamapps"));
            folders.Add(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), @"Steam\steamapps"));
        }
    }

    public static class StartupManager
    {
        private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
        private const string ValueName = "QbeeGameSpeedLimiter";

        public static bool IsEnabled()
        {
            using (var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, false))
            {
                return key != null && key.GetValue(ValueName) != null;
            }
        }

        public static void SetEnabled(bool enabled)
        {
            using (var key = Registry.CurrentUser.CreateSubKey(RunKeyPath))
            {
                if (key == null)
                {
                    throw new InvalidOperationException("无法打开当前用户启动项注册表。");
                }

                if (enabled)
                {
                    key.SetValue(ValueName, "\"" + Application.ExecutablePath + "\"");
                }
                else
                {
                    key.DeleteValue(ValueName, false);
                }
            }
        }
    }

    public class MainForm : Form
    {
        private readonly TextBox urlBox = new TextBox();
        private readonly TextBox usernameBox = new TextBox();
        private readonly TextBox passwordBox = new TextBox();
        private readonly NumericUpDown intervalBox = new NumericUpDown();
        private readonly CheckBox startWithWindowsBox = new CheckBox();
        private readonly CheckBox autoStartMonitorBox = new CheckBox();
        private readonly ListBox folderList = new ListBox();
        private readonly Label statusLabel = new Label();
        private readonly Label detectedLabel = new Label();
        private readonly Button startButton = new Button();
        private readonly Button stopButton = new Button();
        private readonly GameMonitor monitor;
        private AppConfig config;
        private bool stoppingMonitor;
        private bool closingAfterStop;

        public MainForm()
        {
            config = ConfigStore.Load();
            monitor = new GameMonitor(SetStatusSafe, SetDetectedGameSafe, SetStoppedSafe);
            Text = "qbee Game Speed Limiter";
            MinimumSize = new Size(860, 620);
            Size = new Size(920, 680);
            StartPosition = FormStartPosition.CenterScreen;
            Font = new Font("Microsoft YaHei UI", 9F);
            BackColor = Color.FromArgb(244, 247, 251);

            BuildUi();
            LoadConfigToUi();
            if (config.auto_start_monitor)
            {
                BeginInvoke((Action)StartMonitor);
            }
        }

        private void BuildUi()
        {
            var root = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(22),
                ColumnCount = 1,
                RowCount = 5
            };
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 76));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 212));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 56));
            Controls.Add(root);

            var header = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                ColumnCount = 2,
                RowCount = 1,
                Margin = new Padding(0, 0, 0, 12)
            };
            header.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            header.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 180));
            root.Controls.Add(header, 0, 0);

            var titleBlock = new TableLayoutPanel { Dock = DockStyle.Fill, RowCount = 2 };
            titleBlock.RowStyles.Add(new RowStyle(SizeType.Absolute, 36));
            titleBlock.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            header.Controls.Add(titleBlock, 0, 0);
            titleBlock.Controls.Add(new Label
            {
                Text = "qbee 游戏限速助手",
                Dock = DockStyle.Fill,
                Font = new Font("Microsoft YaHei UI", 18F, FontStyle.Bold),
                ForeColor = Color.FromArgb(32, 45, 64)
            }, 0, 0);
            titleBlock.Controls.Add(new Label
            {
                Text = "检测到游戏运行时开启备用速度限制，退出游戏后自动恢复。",
                Dock = DockStyle.Fill,
                ForeColor = Color.FromArgb(95, 110, 130)
            }, 0, 1);

            var statusPill = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Color.FromArgb(232, 240, 255),
                Padding = new Padding(14, 10, 14, 10),
                Margin = new Padding(0, 6, 0, 10)
            };
            header.Controls.Add(statusPill, 1, 0);
            statusLabel.Text = "就绪";
            statusLabel.Dock = DockStyle.Fill;
            statusLabel.TextAlign = ContentAlignment.MiddleCenter;
            statusLabel.ForeColor = Color.FromArgb(33, 86, 178);
            statusLabel.Font = new Font("Microsoft YaHei UI", 9F, FontStyle.Bold);
            statusPill.Controls.Add(statusLabel);

            var account = CreateCard("连接与启动");
            root.Controls.Add(account, 0, 1);

            var accountLayout = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(18, 16, 18, 16),
                ColumnCount = 4,
                RowCount = 4
            };
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 82));
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 82));
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            account.Controls.Add(accountLayout, 0, 1);

            StyleTextBox(urlBox);
            StyleTextBox(usernameBox);
            StyleTextBox(passwordBox);
            AddRow(accountLayout, 0, "地址", urlBox, 1, 3);
            AddRow(accountLayout, 1, "用户名", usernameBox, 1, 1);
            passwordBox.UseSystemPasswordChar = true;
            AddRow(accountLayout, 1, "密码", passwordBox, 3, 1);

            var options = new FlowLayoutPanel
            {
                Dock = DockStyle.Fill,
                FlowDirection = FlowDirection.LeftToRight,
                Margin = new Padding(0, 10, 0, 0)
            };
            intervalBox.Minimum = 1;
            intervalBox.Maximum = 60;
            intervalBox.Width = 72;
            intervalBox.BackColor = Color.White;
            var testButton = CreateButton("测试连接", false);
            testButton.Click += delegate { TestConnection(); };
            options.Controls.Add(MutedLabel("检查间隔"));
            options.Controls.Add(intervalBox);
            options.Controls.Add(MutedLabel("秒"));
            options.Controls.Add(testButton);
            accountLayout.Controls.Add(options, 1, 2);
            accountLayout.SetColumnSpan(options, 3);

            var startupOptions = new FlowLayoutPanel
            {
                Dock = DockStyle.Fill,
                FlowDirection = FlowDirection.LeftToRight,
                Margin = new Padding(0, 8, 0, 0)
            };
            startWithWindowsBox.Text = "开机自启动";
            startWithWindowsBox.AutoSize = true;
            startWithWindowsBox.ForeColor = Color.FromArgb(49, 62, 80);
            autoStartMonitorBox.Text = "启动后自动开始监控";
            autoStartMonitorBox.AutoSize = true;
            autoStartMonitorBox.ForeColor = Color.FromArgb(49, 62, 80);
            startupOptions.Controls.Add(startWithWindowsBox);
            startupOptions.Controls.Add(autoStartMonitorBox);
            accountLayout.Controls.Add(startupOptions, 1, 3);
            accountLayout.SetColumnSpan(startupOptions, 3);

            var folders = CreateCard("游戏库文件夹");
            root.Controls.Add(folders, 0, 2);
            var folderLayout = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(18, 14, 18, 18),
                ColumnCount = 2,
                RowCount = 1
            };
            folderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            folderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 124));
            folders.Controls.Add(folderLayout, 0, 1);

            folderList.Dock = DockStyle.Fill;
            folderList.BorderStyle = BorderStyle.None;
            folderList.BackColor = Color.FromArgb(248, 250, 253);
            folderList.ForeColor = Color.FromArgb(38, 52, 72);
            folderList.Font = new Font("Consolas", 9F);
            folderLayout.Controls.Add(folderList, 0, 0);

            var folderButtons = new FlowLayoutPanel
            {
                Dock = DockStyle.Fill,
                FlowDirection = FlowDirection.TopDown,
                WrapContents = false,
                Padding = new Padding(12, 0, 0, 0)
            };
            folderLayout.Controls.Add(folderButtons, 1, 0);

            var scanButton = CreateButton("自动扫描", true);
            var addButton = CreateButton("添加", false);
            var removeButton = CreateButton("删除", false);
            var openButton = CreateButton("打开配置", false);
            scanButton.Click += delegate { ScanFolders(); };
            addButton.Click += delegate { AddFolder(); };
            removeButton.Click += delegate { RemoveFolder(); };
            openButton.Click += delegate { Process.Start("explorer.exe", ConfigStore.AppDirectory); };
            folderButtons.Controls.Add(scanButton);
            folderButtons.Controls.Add(addButton);
            folderButtons.Controls.Add(removeButton);
            folderButtons.Controls.Add(openButton);

            detectedLabel.Text = "当前检测到：无";
            detectedLabel.Dock = DockStyle.Fill;
            detectedLabel.TextAlign = ContentAlignment.MiddleLeft;
            detectedLabel.AutoEllipsis = true;
            detectedLabel.ForeColor = Color.FromArgb(83, 98, 118);
            detectedLabel.Padding = new Padding(4, 0, 0, 0);
            root.Controls.Add(detectedLabel, 0, 3);

            var actions = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                ColumnCount = 3,
                RowCount = 1,
                Margin = new Padding(0, 8, 0, 0)
            };
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 116));
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 116));
            root.Controls.Add(actions, 0, 4);

            var saveButton = CreateButton("保存配置", false);
            saveButton.Dock = DockStyle.Left;
            saveButton.Width = 116;
            saveButton.Click += delegate { SaveFromUi(); };
            startButton.Text = "开始监控";
            StyleButton(startButton, true);
            startButton.Click += delegate { StartMonitor(); };
            stopButton.Text = "停止监控";
            StyleButton(stopButton, false);
            stopButton.Enabled = false;
            stopButton.Click += delegate { StopMonitor(); };
            actions.Controls.Add(saveButton, 0, 0);
            actions.Controls.Add(startButton, 1, 0);
            actions.Controls.Add(stopButton, 2, 0);
        }

        private static TableLayoutPanel CreateCard(string title)
        {
            var card = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                RowCount = 2,
                ColumnCount = 1,
                BackColor = Color.White,
                Margin = new Padding(0, 0, 0, 14)
            };
            card.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            card.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            card.Controls.Add(new Label
            {
                Text = title,
                Dock = DockStyle.Fill,
                Padding = new Padding(18, 12, 18, 0),
                Font = new Font("Microsoft YaHei UI", 10F, FontStyle.Bold),
                ForeColor = Color.FromArgb(38, 52, 72)
            }, 0, 0);
            return card;
        }

        private static Label MutedLabel(string text)
        {
            return new Label
            {
                Text = text,
                AutoSize = true,
                ForeColor = Color.FromArgb(83, 98, 118),
                Padding = new Padding(0, 7, 8, 0)
            };
        }

        private static Button CreateButton(string text, bool primary)
        {
            var button = new Button { Text = text, Width = 104, Height = 34, Margin = new Padding(0, 0, 0, 10) };
            StyleButton(button, primary);
            return button;
        }

        private static void StyleButton(Button button, bool primary)
        {
            button.FlatStyle = FlatStyle.Flat;
            button.FlatAppearance.BorderSize = 0;
            button.BackColor = primary ? Color.FromArgb(41, 111, 235) : Color.FromArgb(232, 237, 245);
            button.ForeColor = primary ? Color.White : Color.FromArgb(38, 52, 72);
            button.Font = new Font("Microsoft YaHei UI", 9F, FontStyle.Bold);
            button.Height = 34;
            button.Margin = new Padding(6, 0, 0, 0);
            button.Cursor = Cursors.Hand;
        }

        private static void StyleTextBox(TextBox box)
        {
            box.BorderStyle = BorderStyle.FixedSingle;
            box.BackColor = Color.FromArgb(248, 250, 253);
            box.ForeColor = Color.FromArgb(38, 52, 72);
            box.Margin = new Padding(0, 4, 12, 4);
        }

        private static void AddRow(TableLayoutPanel panel, int row, string label, TextBox box, int column, int span)
        {
            panel.Controls.Add(new Label
            {
                Text = label,
                Dock = DockStyle.Fill,
                TextAlign = ContentAlignment.MiddleLeft,
                ForeColor = Color.FromArgb(83, 98, 118)
            }, column - 1, row);
            box.Dock = DockStyle.Fill;
            panel.Controls.Add(box, column, row);
            if (span > 1)
            {
                panel.SetColumnSpan(box, span);
            }
        }

        private void LoadConfigToUi()
        {
            urlBox.Text = config.qbee_url;
            usernameBox.Text = config.username;
            passwordBox.Text = config.password;
            intervalBox.Value = Math.Max(1, Math.Min(60, config.check_interval_seconds));
            startWithWindowsBox.Checked = StartupManager.IsEnabled();
            autoStartMonitorBox.Checked = config.auto_start_monitor;
            folderList.Items.Clear();
            foreach (var folder in config.game_folders ?? new List<string>())
            {
                folderList.Items.Add(folder);
            }
        }

        private bool SaveFromUi()
        {
            var url = urlBox.Text.Trim();
            Uri parsedUrl;
            if (!Uri.TryCreate(url, UriKind.Absolute, out parsedUrl) ||
                (parsedUrl.Scheme != Uri.UriSchemeHttp && parsedUrl.Scheme != Uri.UriSchemeHttps))
            {
                MessageBox.Show(this, "请输入有效的 qB Web UI 地址，例如 http://127.0.0.1:8080。", "配置不完整", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                urlBox.Focus();
                return false;
            }

            if (folderList.Items.Count == 0)
            {
                MessageBox.Show(this, "请至少添加一个游戏库文件夹，或点击“自动扫描”。", "配置不完整", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return false;
            }

            config.qbee_url = url;
            config.username = usernameBox.Text.Trim();
            config.password = passwordBox.Text;
            config.check_interval_seconds = Convert.ToInt32(intervalBox.Value);
            config.game_folders = folderList.Items.Cast<string>().ToList();
            config.start_with_windows = startWithWindowsBox.Checked;
            config.auto_start_monitor = autoStartMonitorBox.Checked;
            ConfigStore.Save(config);
            try
            {
                StartupManager.SetEnabled(startWithWindowsBox.Checked);
            }
            catch (Exception error)
            {
                MessageBox.Show(this, error.Message, "开机自启动设置失败", MessageBoxButtons.OK, MessageBoxIcon.Error);
                startWithWindowsBox.Checked = StartupManager.IsEnabled();
                config.start_with_windows = startWithWindowsBox.Checked;
                ConfigStore.Save(config);
                return false;
            }
            statusLabel.Text = "已保存";
            return true;
        }

        private void AddFolder()
        {
            using (var dialog = new FolderBrowserDialog())
            {
                dialog.Description = "选择游戏库文件夹";
                if (dialog.ShowDialog(this) != DialogResult.OK) return;

                var existing = folderList.Items.Cast<string>().Any(item =>
                    string.Equals(item, dialog.SelectedPath, StringComparison.OrdinalIgnoreCase));
                if (!existing)
                {
                    folderList.Items.Add(dialog.SelectedPath);
                    statusLabel.Text = "已添加游戏库";
                }
            }
        }

        private void ScanFolders()
        {
            statusLabel.Text = "正在扫描游戏库...";
            ThreadPool.QueueUserWorkItem(delegate
            {
                try
                {
                    var found = GameLibraryScanner.Scan();
                    if (IsDisposed) return;
                    BeginInvoke((Action)(() =>
                    {
                        var existing = new HashSet<string>(folderList.Items.Cast<string>(), StringComparer.OrdinalIgnoreCase);
                        var added = 0;

                        foreach (var folder in found)
                        {
                            if (existing.Add(folder))
                            {
                                folderList.Items.Add(folder);
                                added++;
                            }
                        }

                        statusLabel.Text = added > 0 ? "自动扫描完成，新增 " + added + " 个游戏库。" : "自动扫描完成，没有发现新的游戏库。";
                    }));
                }
                catch (Exception error)
                {
                    SetStatusSafe("自动扫描失败：" + error.Message);
                }
            });
        }

        private void RemoveFolder()
        {
            var selected = folderList.SelectedIndex;
            if (selected >= 0)
            {
                folderList.Items.RemoveAt(selected);
                statusLabel.Text = "已删除游戏库";
            }
        }

        private void TestConnection()
        {
            if (!SaveFromUi()) return;
            statusLabel.Text = "正在测试连接...";
            ThreadPool.QueueUserWorkItem(delegate
            {
                try
                {
                    var client = new QbeeClient(config);
                    var enabled = client.SpeedLimitsEnabled();
                    SetStatusSafe(enabled ? "连接成功，备用速度限制当前已打开。" : "连接成功，备用速度限制当前已关闭。");
                }
                catch (Exception error)
                {
                    SetStatusSafe("连接失败：" + error.Message);
                    BeginInvoke((Action)(() => MessageBox.Show(this, error.Message, "连接失败", MessageBoxButtons.OK, MessageBoxIcon.Error)));
                }
            });
        }

        private void StartMonitor()
        {
            if (!SaveFromUi()) return;
            monitor.Start(config);
            startButton.Enabled = false;
            stopButton.Enabled = true;
            statusLabel.Text = "监控中";
        }

        private void StopMonitor()
        {
            stopButton.Enabled = false;
            startButton.Enabled = false;
            stoppingMonitor = true;
            statusLabel.Text = "正在停止监控...";
            ThreadPool.QueueUserWorkItem(delegate
            {
                monitor.Stop();
                SetStatusSafe("已停止");
            });
        }

        private void SetStatusSafe(string text)
        {
            if (IsDisposed) return;
            if (InvokeRequired)
            {
                BeginInvoke((Action)(() => statusLabel.Text = text));
            }
            else
            {
                statusLabel.Text = text;
            }
        }

        private void SetDetectedGameSafe(string path)
        {
            var text = string.IsNullOrWhiteSpace(path) ? "当前检测到：无" : "当前检测到：" + path;
            if (IsDisposed) return;
            if (InvokeRequired)
            {
                BeginInvoke((Action)(() => detectedLabel.Text = text));
            }
            else
            {
                detectedLabel.Text = text;
            }
        }

        private void SetStoppedSafe()
        {
            if (IsDisposed) return;
            if (InvokeRequired)
            {
                BeginInvoke((Action)SetStoppedSafe);
                return;
            }

            startButton.Enabled = true;
            stopButton.Enabled = false;
            stoppingMonitor = false;
        }

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            if (monitor.IsRunning && !closingAfterStop)
            {
                if (stoppingMonitor)
                {
                    e.Cancel = true;
                    statusLabel.Text = "正在停止监控，请稍等...";
                    return;
                }

                var result = MessageBox.Show(this, "监控仍在运行。要停止监控并退出吗？", "退出", MessageBoxButtons.YesNo, MessageBoxIcon.Question);
                if (result != DialogResult.Yes)
                {
                    e.Cancel = true;
                    return;
                }
                e.Cancel = true;
                stoppingMonitor = true;
                startButton.Enabled = false;
                stopButton.Enabled = false;
                statusLabel.Text = "正在停止监控，完成后会自动退出...";
                ThreadPool.QueueUserWorkItem(delegate
                {
                    monitor.Stop();
                    if (!IsDisposed)
                    {
                        BeginInvoke((Action)(() =>
                        {
                            closingAfterStop = true;
                            stoppingMonitor = false;
                            Close();
                        }));
                    }
                });
                return;
            }
            base.OnFormClosing(e);
        }
    }

    public static class Program
    {
        [STAThread]
        public static void Main()
        {
            bool createdNew;
            using (var mutex = new Mutex(true, "QbeeGameSpeedLimiter.SingleInstance", out createdNew))
            {
                if (!createdNew)
                {
                    MessageBox.Show("qbee 游戏限速助手已经在运行。", "qbee Game Speed Limiter", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }

                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Application.Run(new MainForm());
            }
        }
    }
}
