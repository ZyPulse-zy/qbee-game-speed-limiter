using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Management;
using System.Net;
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
        public int check_interval_seconds { get; set; }
        public bool restore_on_exit { get; set; }
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
                    "epicgameslauncher.exe",
                    "goggalaxy.exe",
                    "wegame.exe",
                    "battle.net.exe"
                },
                check_interval_seconds = 3,
                restore_on_exit = true,
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
        private readonly string baseUrl;
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
            request.Timeout = 10000;

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
        private Thread worker;
        private bool stopping;

        public GameMonitor(Action<string> onStatus)
        {
            this.onStatus = onStatus;
        }

        public bool IsRunning
        {
            get { return worker != null && worker.IsAlive; }
        }

        public void Start(AppConfig config)
        {
            if (IsRunning) return;
            stopping = false;
            worker = new Thread(() => Run(config));
            worker.IsBackground = true;
            worker.Start();
        }

        public void Stop()
        {
            stopping = true;
            if (worker != null && worker.IsAlive)
            {
                worker.Join(5000);
            }
        }

        private void Run(AppConfig config)
        {
            var client = new QbeeClient(config);
            bool? lastGameState = null;
            var lastDetectedGame = "";
            var lastStillRunningLog = DateTime.MinValue;

            Log(config, "Monitor started.");
            Log(config, "Game folders: " + string.Join(", ", config.game_folders ?? new List<string>()));
            onStatus("监控中");

            try
            {
                while (!stopping)
                {
                    var detectedGame = DetectRunningGame(config);
                    var gameRunning = detectedGame != null;

                    if (lastGameState == null || gameRunning != lastGameState.Value)
                    {
                        if (gameRunning)
                        {
                            var changed = client.SetSpeedLimits(true);
                            var message = changed ? "检测到游戏运行，已打开备用速度限制。" : "检测到游戏运行，备用速度限制已是打开状态。";
                            Log(config, message + " Detected: " + detectedGame);
                            onStatus(message);
                        }
                        else if (lastGameState != null)
                        {
                            var changed = client.SetSpeedLimits(false);
                            var message = changed ? "检测到游戏退出，已关闭备用速度限制。" : "检测到游戏退出，备用速度限制已是关闭状态。";
                            Log(config, message);
                            onStatus(message);
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

                    Thread.Sleep(Math.Max(1, config.check_interval_seconds) * 1000);
                }
            }
            catch (Exception error)
            {
                Log(config, "Error: " + error.Message);
                onStatus("监控出错：" + error.Message);
            }
            finally
            {
                if (config.restore_on_exit)
                {
                    try
                    {
                        client.SetSpeedLimits(false);
                        Log(config, "Program exited. Tried to disable alternative speed limits.");
                    }
                    catch (Exception error)
                    {
                        Log(config, "Failed to disable alternative speed limits on exit: " + error.Message);
                    }
                }
            }
        }

        private static string DetectRunningGame(AppConfig config)
        {
            var folders = (config.game_folders ?? new List<string>())
                .Where(item => !string.IsNullOrWhiteSpace(item))
                .Select(NormalizePath)
                .ToList();
            var processNames = new HashSet<string>(
                (config.game_processes ?? new List<string>()).Select(item => item.ToLowerInvariant()));
            var excluded = new HashSet<string>(
                (config.exclude_processes ?? new List<string>()).Select(item => item.ToLowerInvariant()));

            using (var searcher = new ManagementObjectSearcher("SELECT Name, ExecutablePath FROM Win32_Process"))
            {
                foreach (ManagementObject process in searcher.Get())
                {
                    var name = Convert.ToString(process["Name"] ?? "").ToLowerInvariant();
                    var path = Convert.ToString(process["ExecutablePath"] ?? "");
                    if (string.IsNullOrWhiteSpace(name) || excluded.Contains(name)) continue;
                    if (processNames.Contains(name)) return name;
                    if (!string.IsNullOrWhiteSpace(path) && folders.Any(folder => IsUnderFolder(path, folder))) return path;
                }
            }

            return null;
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

    public class MainForm : Form
    {
        private readonly TextBox urlBox = new TextBox();
        private readonly TextBox usernameBox = new TextBox();
        private readonly TextBox passwordBox = new TextBox();
        private readonly NumericUpDown intervalBox = new NumericUpDown();
        private readonly ListBox folderList = new ListBox();
        private readonly Label statusLabel = new Label();
        private readonly Button startButton = new Button();
        private readonly Button stopButton = new Button();
        private readonly GameMonitor monitor;
        private AppConfig config;

        public MainForm()
        {
            config = ConfigStore.Load();
            monitor = new GameMonitor(SetStatusSafe);
            Text = "qbee Game Speed Limiter";
            MinimumSize = new Size(760, 560);
            Size = new Size(820, 600);
            StartPosition = FormStartPosition.CenterScreen;
            Font = new Font("Microsoft YaHei UI", 9F);

            BuildUi();
            LoadConfigToUi();
        }

        private void BuildUi()
        {
            var root = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(18),
                ColumnCount = 1,
                RowCount = 3
            };
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 168));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            Controls.Add(root);

            var account = new GroupBox { Text = "qbee Web UI", Dock = DockStyle.Fill };
            root.Controls.Add(account, 0, 0);

            var accountLayout = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(12),
                ColumnCount = 2,
                RowCount = 4
            };
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 90));
            accountLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            account.Controls.Add(accountLayout);

            AddRow(accountLayout, 0, "地址", urlBox);
            AddRow(accountLayout, 1, "用户名", usernameBox);
            passwordBox.UseSystemPasswordChar = true;
            AddRow(accountLayout, 2, "密码", passwordBox);

            var options = new FlowLayoutPanel { Dock = DockStyle.Fill, FlowDirection = FlowDirection.LeftToRight };
            intervalBox.Minimum = 1;
            intervalBox.Maximum = 60;
            intervalBox.Width = 64;
            var testButton = new Button { Text = "测试连接", AutoSize = true };
            testButton.Click += delegate { TestConnection(); };
            options.Controls.Add(new Label { Text = "检查间隔", AutoSize = true, Padding = new Padding(0, 6, 0, 0) });
            options.Controls.Add(intervalBox);
            options.Controls.Add(new Label { Text = "秒", AutoSize = true, Padding = new Padding(0, 6, 18, 0) });
            options.Controls.Add(testButton);
            accountLayout.Controls.Add(options, 1, 3);

            var folders = new GroupBox { Text = "游戏库文件夹", Dock = DockStyle.Fill };
            root.Controls.Add(folders, 0, 1);
            var folderLayout = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(12),
                ColumnCount = 2,
                RowCount = 1
            };
            folderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            folderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 112));
            folders.Controls.Add(folderLayout);

            folderList.Dock = DockStyle.Fill;
            folderLayout.Controls.Add(folderList, 0, 0);

            var folderButtons = new FlowLayoutPanel
            {
                Dock = DockStyle.Fill,
                FlowDirection = FlowDirection.TopDown,
                WrapContents = false
            };
            folderLayout.Controls.Add(folderButtons, 1, 0);

            var scanButton = new Button { Text = "自动扫描", Width = 92 };
            var addButton = new Button { Text = "添加", Width = 92 };
            var removeButton = new Button { Text = "删除", Width = 92 };
            var openButton = new Button { Text = "打开配置", Width = 92 };
            scanButton.Click += delegate { ScanFolders(); };
            addButton.Click += delegate { AddFolder(); };
            removeButton.Click += delegate { RemoveFolder(); };
            openButton.Click += delegate { Process.Start("explorer.exe", ConfigStore.AppDirectory); };
            folderButtons.Controls.Add(scanButton);
            folderButtons.Controls.Add(addButton);
            folderButtons.Controls.Add(removeButton);
            folderButtons.Controls.Add(openButton);

            var actions = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                ColumnCount = 4,
                RowCount = 1
            };
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 96));
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 96));
            actions.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 96));
            root.Controls.Add(actions, 0, 2);

            statusLabel.Text = "就绪";
            statusLabel.Dock = DockStyle.Fill;
            statusLabel.TextAlign = ContentAlignment.MiddleLeft;
            actions.Controls.Add(statusLabel, 0, 0);

            var saveButton = new Button { Text = "保存配置", Dock = DockStyle.Fill };
            saveButton.Click += delegate { SaveFromUi(); };
            startButton.Text = "开始监控";
            startButton.Dock = DockStyle.Fill;
            startButton.Click += delegate { StartMonitor(); };
            stopButton.Text = "停止监控";
            stopButton.Dock = DockStyle.Fill;
            stopButton.Enabled = false;
            stopButton.Click += delegate { StopMonitor(); };
            actions.Controls.Add(saveButton, 1, 0);
            actions.Controls.Add(startButton, 2, 0);
            actions.Controls.Add(stopButton, 3, 0);
        }

        private static void AddRow(TableLayoutPanel panel, int row, string label, TextBox box)
        {
            panel.Controls.Add(new Label
            {
                Text = label,
                Dock = DockStyle.Fill,
                TextAlign = ContentAlignment.MiddleLeft
            }, 0, row);
            box.Dock = DockStyle.Fill;
            panel.Controls.Add(box, 1, row);
        }

        private void LoadConfigToUi()
        {
            urlBox.Text = config.qbee_url;
            usernameBox.Text = config.username;
            passwordBox.Text = config.password;
            intervalBox.Value = Math.Max(1, Math.Min(60, config.check_interval_seconds));
            folderList.Items.Clear();
            foreach (var folder in config.game_folders ?? new List<string>())
            {
                folderList.Items.Add(folder);
            }
        }

        private void SaveFromUi()
        {
            config.qbee_url = urlBox.Text.Trim();
            config.username = usernameBox.Text.Trim();
            config.password = passwordBox.Text;
            config.check_interval_seconds = Convert.ToInt32(intervalBox.Value);
            config.game_folders = folderList.Items.Cast<string>().ToList();
            ConfigStore.Save(config);
            statusLabel.Text = "已保存";
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
            var found = GameLibraryScanner.Scan();
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
            SaveFromUi();
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
            SaveFromUi();
            monitor.Start(config);
            startButton.Enabled = false;
            stopButton.Enabled = true;
            statusLabel.Text = "监控中";
        }

        private void StopMonitor()
        {
            monitor.Stop();
            startButton.Enabled = true;
            stopButton.Enabled = false;
            statusLabel.Text = "已停止";
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

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            if (monitor.IsRunning)
            {
                var result = MessageBox.Show(this, "监控仍在运行。要停止监控并退出吗？", "退出", MessageBoxButtons.YesNo, MessageBoxIcon.Question);
                if (result != DialogResult.Yes)
                {
                    e.Cancel = true;
                    return;
                }
                monitor.Stop();
            }
            base.OnFormClosing(e);
        }
    }

    public static class Program
    {
        [STAThread]
        public static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new MainForm());
        }
    }
}
