#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif

#include <windows.h>
#include <commctrl.h>
#include <tlhelp32.h>
#include <winhttp.h>
#include <shlobj.h>

#include <algorithm>
#include <atomic>
#include <chrono>
#include <fstream>
#include <map>
#include <regex>
#include <set>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "winhttp.lib")

namespace
{
    const wchar_t* AppTitle = L"qbee 游戏限速助手";
    const wchar_t* RunKeyPath = L"Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const wchar_t* RunValueName = L"QbeeGameSpeedLimiter";

    enum ControlId
    {
        IdUrl = 1001,
        IdUser,
        IdPassword,
        IdInterval,
        IdStartWithWindows,
        IdAutoStartMonitor,
        IdFolderList,
        IdTest,
        IdScan,
        IdAdd,
        IdRemove,
        IdOpenConfig,
        IdSave,
        IdStart,
        IdStop
    };

    struct AppConfig
    {
        std::wstring qbeeUrl = L"http://127.0.0.1:8080";
        std::wstring username = L"admin";
        std::wstring password;
        std::vector<std::wstring> gameFolders = {
            L"C:\\Program Files (x86)\\Steam\\steamapps",
            L"D:\\SteamLibrary\\steamapps"
        };
        std::vector<std::wstring> gameProcesses;
        std::vector<std::wstring> excludeProcesses = {
            L"steam.exe", L"steamservice.exe", L"steamwebhelper.exe",
            L"wallpaper32.exe", L"wallpaper64.exe", L"wallpaper_engine.exe",
            L"epicgameslauncher.exe", L"goggalaxy.exe", L"wegame.exe", L"battle.net.exe"
        };
        std::vector<std::wstring> excludePathKeywords = {
            L"\\steamapps\\common\\steamworks shared\\",
            L"\\steamapps\\common\\proton ",
            L"\\steamapps\\common\\steam linux runtime",
            L"\\_commonredist\\", L"\\redist\\", L"\\redistributable\\",
            L"\\installer\\", L"\\uninstall\\", L"\\launcher\\", L"\\wallpaper_engine\\"
        };
        std::vector<std::wstring> excludeSteamAppKeywords = {
            L"wallpaper", L"dedicated server", L"server tool", L"server dedicated",
            L"sdk", L"tool", L"tools", L"benchmark", L"editor", L"modding",
            L"workshop", L"proton", L"redistributable", L"runtime"
        };
        int checkIntervalSeconds = 5;
        bool restoreOnExit = true;
        bool startWithWindows = false;
        bool autoStartMonitor = false;
        std::wstring logFile = L"qbee_game_speed_limiter.log";
    };

    std::wstring ToWide(const std::string& value)
    {
        if (value.empty()) return L"";
        int size = MultiByteToWideChar(CP_UTF8, 0, value.data(), (int)value.size(), nullptr, 0);
        std::wstring result(size, 0);
        MultiByteToWideChar(CP_UTF8, 0, value.data(), (int)value.size(), &result[0], size);
        return result;
    }

    std::string ToUtf8(const std::wstring& value)
    {
        if (value.empty()) return "";
        int size = WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), nullptr, 0, nullptr, nullptr);
        std::string result(size, 0);
        WideCharToMultiByte(CP_UTF8, 0, value.data(), (int)value.size(), &result[0], size, nullptr, nullptr);
        return result;
    }

    std::wstring Lower(std::wstring value)
    {
        std::transform(value.begin(), value.end(), value.begin(), [](wchar_t ch) { return (wchar_t)towlower(ch); });
        return value;
    }

    std::wstring Trim(const std::wstring& value)
    {
        size_t first = value.find_first_not_of(L" \t\r\n");
        if (first == std::wstring::npos) return L"";
        size_t last = value.find_last_not_of(L" \t\r\n");
        return value.substr(first, last - first + 1);
    }

    std::wstring AppDirectory()
    {
        wchar_t path[MAX_PATH]{};
        GetModuleFileNameW(nullptr, path, MAX_PATH);
        std::wstring value(path);
        size_t slash = value.find_last_of(L"\\/");
        return slash == std::wstring::npos ? L"." : value.substr(0, slash);
    }

    std::wstring ConfigPath()
    {
        return AppDirectory() + L"\\qbee_game_speed_limiter.json";
    }

    std::wstring NormalizePath(const std::wstring& path)
    {
        wchar_t expanded[MAX_PATH * 4]{};
        ExpandEnvironmentStringsW(path.c_str(), expanded, (DWORD)(MAX_PATH * 4));

        wchar_t full[MAX_PATH * 4]{};
        if (GetFullPathNameW(expanded, (DWORD)(MAX_PATH * 4), full, nullptr) == 0)
        {
            return Lower(Trim(path));
        }

        std::wstring result(full);
        while (!result.empty() && (result.back() == L'\\' || result.back() == L'/')) result.pop_back();
        return Lower(result);
    }

    bool StartsWithFolder(const std::wstring& file, const std::wstring& folder)
    {
        std::wstring normalizedFile = NormalizePath(file);
        std::wstring normalizedFolder = NormalizePath(folder);
        return normalizedFile == normalizedFolder ||
               (normalizedFile.size() > normalizedFolder.size() &&
                normalizedFile.compare(0, normalizedFolder.size(), normalizedFolder) == 0 &&
                normalizedFile[normalizedFolder.size()] == L'\\');
    }

    std::wstring JsonEscape(const std::wstring& value)
    {
        std::wstring output;
        for (wchar_t ch : value)
        {
            if (ch == L'\\') output += L"\\\\";
            else if (ch == L'"') output += L"\\\"";
            else if (ch == L'\n') output += L"\\n";
            else if (ch != L'\r') output += ch;
        }
        return output;
    }

    std::wstring JsonUnescape(std::wstring value)
    {
        std::wstring output;
        for (size_t i = 0; i < value.size(); ++i)
        {
            if (value[i] == L'\\' && i + 1 < value.size())
            {
                wchar_t next = value[++i];
                if (next == L'n') output += L'\n';
                else output += next;
            }
            else
            {
                output += value[i];
            }
        }
        return output;
    }

    std::wstring ExtractString(const std::wstring& json, const std::wstring& key, const std::wstring& fallback)
    {
        std::wregex pattern(L"\"" + key + L"\"\\s*:\\s*\"((?:\\\\.|[^\"])*)\"");
        std::wsmatch match;
        return std::regex_search(json, match, pattern) ? JsonUnescape(match[1].str()) : fallback;
    }

    int ExtractInt(const std::wstring& json, const std::wstring& key, int fallback)
    {
        std::wregex pattern(L"\"" + key + L"\"\\s*:\\s*(\\d+)");
        std::wsmatch match;
        return std::regex_search(json, match, pattern) ? std::max(1, std::stoi(match[1].str())) : fallback;
    }

    bool ExtractBool(const std::wstring& json, const std::wstring& key, bool fallback)
    {
        std::wregex pattern(L"\"" + key + L"\"\\s*:\\s*(true|false)");
        std::wsmatch match;
        if (!std::regex_search(json, match, pattern)) return fallback;
        return match[1].str() == L"true";
    }

    std::vector<std::wstring> ExtractArray(const std::wstring& json, const std::wstring& key, const std::vector<std::wstring>& fallback)
    {
        std::wregex pattern(L"\"" + key + L"\"\\s*:\\s*\\[([\\s\\S]*?)\\]");
        std::wsmatch match;
        if (!std::regex_search(json, match, pattern)) return fallback;

        std::vector<std::wstring> values;
        std::wregex itemPattern(L"\"((?:\\\\.|[^\"])*)\"");
        std::wstring body = match[1].str();
        for (auto it = std::wsregex_iterator(body.begin(), body.end(), itemPattern); it != std::wsregex_iterator(); ++it)
        {
            values.push_back(JsonUnescape((*it)[1].str()));
        }
        return values;
    }

    std::string ReadFileUtf8(const std::wstring& path)
    {
        std::ifstream file(path.c_str(), std::ios::binary);
        if (!file) return "";
        std::ostringstream buffer;
        buffer << file.rdbuf();
        return buffer.str();
    }

    void WriteFileUtf8(const std::wstring& path, const std::string& data)
    {
        std::ofstream file(path.c_str(), std::ios::binary | std::ios::trunc);
        file.write(data.data(), (std::streamsize)data.size());
    }

    void AppendJsonArray(std::wstringstream& out, const wchar_t* key, const std::vector<std::wstring>& values, bool comma = true)
    {
        out << L"  \"" << key << L"\": [\n";
        for (size_t i = 0; i < values.size(); ++i)
        {
            out << L"    \"" << JsonEscape(values[i]) << L"\"" << (i + 1 == values.size() ? L"\n" : L",\n");
        }
        out << L"  ]" << (comma ? L"," : L"") << L"\n";
    }

    void SaveConfig(const AppConfig& config)
    {
        std::wstringstream out;
        out << L"{\n";
        out << L"  \"qbee_url\": \"" << JsonEscape(config.qbeeUrl) << L"\",\n";
        out << L"  \"username\": \"" << JsonEscape(config.username) << L"\",\n";
        out << L"  \"password\": \"" << JsonEscape(config.password) << L"\",\n";
        AppendJsonArray(out, L"game_folders", config.gameFolders);
        AppendJsonArray(out, L"game_processes", config.gameProcesses);
        AppendJsonArray(out, L"exclude_processes", config.excludeProcesses);
        AppendJsonArray(out, L"exclude_path_keywords", config.excludePathKeywords);
        AppendJsonArray(out, L"exclude_steam_app_keywords", config.excludeSteamAppKeywords);
        out << L"  \"check_interval_seconds\": " << config.checkIntervalSeconds << L",\n";
        out << L"  \"restore_on_exit\": " << (config.restoreOnExit ? L"true" : L"false") << L",\n";
        out << L"  \"start_with_windows\": " << (config.startWithWindows ? L"true" : L"false") << L",\n";
        out << L"  \"auto_start_monitor\": " << (config.autoStartMonitor ? L"true" : L"false") << L",\n";
        out << L"  \"log_file\": \"" << JsonEscape(config.logFile) << L"\"\n";
        out << L"}\n";
        WriteFileUtf8(ConfigPath(), ToUtf8(out.str()));
    }

    AppConfig LoadConfig()
    {
        AppConfig config;
        std::string bytes = ReadFileUtf8(ConfigPath());
        if (bytes.empty())
        {
            SaveConfig(config);
            return config;
        }

        std::wstring json = ToWide(bytes);
        config.qbeeUrl = ExtractString(json, L"qbee_url", config.qbeeUrl);
        config.username = ExtractString(json, L"username", config.username);
        config.password = ExtractString(json, L"password", config.password);
        config.gameFolders = ExtractArray(json, L"game_folders", config.gameFolders);
        config.gameProcesses = ExtractArray(json, L"game_processes", config.gameProcesses);
        config.excludeProcesses = ExtractArray(json, L"exclude_processes", config.excludeProcesses);
        config.excludePathKeywords = ExtractArray(json, L"exclude_path_keywords", config.excludePathKeywords);
        config.excludeSteamAppKeywords = ExtractArray(json, L"exclude_steam_app_keywords", config.excludeSteamAppKeywords);
        config.checkIntervalSeconds = ExtractInt(json, L"check_interval_seconds", config.checkIntervalSeconds);
        config.restoreOnExit = ExtractBool(json, L"restore_on_exit", config.restoreOnExit);
        config.startWithWindows = ExtractBool(json, L"start_with_windows", config.startWithWindows);
        config.autoStartMonitor = ExtractBool(json, L"auto_start_monitor", config.autoStartMonitor);
        config.logFile = ExtractString(json, L"log_file", config.logFile);
        return config;
    }

    void Log(const AppConfig& config, const std::wstring& message)
    {
        SYSTEMTIME time{};
        GetLocalTime(&time);
        wchar_t prefix[64]{};
        wsprintfW(prefix, L"[%04d-%02d-%02d %02d:%02d:%02d] ",
                  time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond);
        std::wstring path = AppDirectory() + L"\\" + (config.logFile.empty() ? L"qbee_game_speed_limiter.log" : config.logFile);
        std::ofstream file(path.c_str(), std::ios::binary | std::ios::app);
        std::string line = ToUtf8(prefix + message + L"\r\n");
        file.write(line.data(), (std::streamsize)line.size());
    }

    std::wstring UrlEncode(const std::wstring& value)
    {
        std::string utf8 = ToUtf8(value);
        std::ostringstream out;
        const char* hex = "0123456789ABCDEF";
        for (unsigned char ch : utf8)
        {
            if ((ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9') || ch == '-' || ch == '_' || ch == '.' || ch == '~')
            {
                out << ch;
            }
            else
            {
                out << '%' << hex[ch >> 4] << hex[ch & 15];
            }
        }
        return ToWide(out.str());
    }

    class QbeeClient
    {
    public:
        explicit QbeeClient(const AppConfig& config)
            : baseUrl(Trim(config.qbeeUrl)), username(config.username), password(config.password)
        {
            while (!baseUrl.empty() && baseUrl.back() == L'/') baseUrl.pop_back();
        }

        bool SpeedLimitsEnabled()
        {
            EnsureLogin();
            return Trim(Request(L"GET", L"/api/v2/transfer/speedLimitsMode", L"")) == L"1";
        }

        bool SetSpeedLimits(bool enabled)
        {
            bool current = SpeedLimitsEnabled();
            if (current == enabled) return false;
            Request(L"POST", L"/api/v2/transfer/toggleSpeedLimitsMode", L"");
            return true;
        }

    private:
        std::wstring baseUrl;
        std::wstring username;
        std::wstring password;
        std::wstring cookie;
        bool loggedIn = false;

        void EnsureLogin()
        {
            if (loggedIn) return;
            ValidateServer();
            if (CanUseWithoutLogin())
            {
                loggedIn = true;
                return;
            }

            std::wstring body = L"username=" + UrlEncode(username) + L"&password=" + UrlEncode(password);
            std::wstring result = Trim(Request(L"POST", L"/api/v2/auth/login", body));
            if (result != L"Ok.")
            {
                throw std::runtime_error("qbee login failed");
            }
            loggedIn = true;
        }

        void ValidateServer()
        {
            try
            {
                std::wstring root = Request(L"GET", L"/", L"");
                if (root.find(L"CEF remote debugging") != std::wstring::npos)
                {
                    if (!TrySwitchToIpv6Loopback())
                    {
                        throw std::runtime_error("current URL is CEF remote debugging, not qBittorrent Web UI");
                    }
                }
            }
            catch (...)
            {
                throw;
            }
        }

        bool TrySwitchToIpv6Loopback()
        {
            URL_COMPONENTS parts{};
            wchar_t host[256]{};
            parts.dwStructSize = sizeof(parts);
            parts.lpszHostName = host;
            parts.dwHostNameLength = 256;
            if (!WinHttpCrackUrl(baseUrl.c_str(), 0, 0, &parts)) return false;

            std::wstring hostName(host, parts.dwHostNameLength);
            if (Lower(hostName) != L"localhost" && hostName != L"127.0.0.1") return false;

            std::wstring original = baseUrl;
            std::wstringstream candidate;
            candidate << (parts.nScheme == INTERNET_SCHEME_HTTPS ? L"https://[::1]" : L"http://[::1]");
            if (parts.nPort) candidate << L":" << parts.nPort;
            baseUrl = candidate.str();

            try
            {
                Request(L"GET", L"/api/v2/app/version", L"");
                return true;
            }
            catch (...)
            {
                baseUrl = original;
                return false;
            }
        }

        bool CanUseWithoutLogin()
        {
            try
            {
                Request(L"GET", L"/api/v2/app/version", L"");
                return true;
            }
            catch (...)
            {
                return false;
            }
        }

        std::wstring Request(const std::wstring& method, const std::wstring& path, const std::wstring& body)
        {
            URL_COMPONENTS parts{};
            wchar_t host[256]{};
            parts.dwStructSize = sizeof(parts);
            parts.lpszHostName = host;
            parts.dwHostNameLength = 256;
            if (!WinHttpCrackUrl(baseUrl.c_str(), 0, 0, &parts))
            {
                throw std::runtime_error("invalid qB Web UI URL");
            }

            HINTERNET session = WinHttpOpen(L"qbee-game-speed-limiter/4.0", WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, WINHTTP_NO_PROXY_NAME, WINHTTP_NO_PROXY_BYPASS, 0);
            if (!session) throw std::runtime_error("cannot open HTTP session");
            WinHttpSetTimeouts(session, 5000, 5000, 5000, 5000);

            HINTERNET connect = WinHttpConnect(session, std::wstring(host, parts.dwHostNameLength).c_str(), parts.nPort, 0);
            if (!connect)
            {
                WinHttpCloseHandle(session);
                throw std::runtime_error("cannot connect to qB Web UI");
            }

            DWORD flags = parts.nScheme == INTERNET_SCHEME_HTTPS ? WINHTTP_FLAG_SECURE : 0;
            HINTERNET request = WinHttpOpenRequest(connect, method.c_str(), path.c_str(), nullptr, WINHTTP_NO_REFERER, WINHTTP_DEFAULT_ACCEPT_TYPES, flags);
            if (!request)
            {
                WinHttpCloseHandle(connect);
                WinHttpCloseHandle(session);
                throw std::runtime_error("cannot create HTTP request");
            }

            std::wstring headers = L"Referer: " + baseUrl + L"\r\n";
            if (!cookie.empty()) headers += L"Cookie: " + cookie + L"\r\n";

            std::string bodyUtf8 = ToUtf8(body);
            if (!body.empty()) headers += L"Content-Type: application/x-www-form-urlencoded\r\n";

            BOOL ok = WinHttpSendRequest(
                request,
                headers.c_str(),
                (DWORD)-1L,
                body.empty() ? WINHTTP_NO_REQUEST_DATA : (LPVOID)bodyUtf8.data(),
                (DWORD)bodyUtf8.size(),
                (DWORD)bodyUtf8.size(),
                0);
            if (ok) ok = WinHttpReceiveResponse(request, nullptr);

            if (!ok)
            {
                WinHttpCloseHandle(request);
                WinHttpCloseHandle(connect);
                WinHttpCloseHandle(session);
                throw std::runtime_error("qB Web UI request failed");
            }

            DWORD status = 0;
            DWORD statusSize = sizeof(status);
            WinHttpQueryHeaders(request, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, nullptr, &status, &statusSize, nullptr);
            if (status >= 400)
            {
                WinHttpCloseHandle(request);
                WinHttpCloseHandle(connect);
                WinHttpCloseHandle(session);
                throw std::runtime_error("qB Web UI returned an error");
            }

            DWORD cookieSize = 0;
            WinHttpQueryHeaders(request, WINHTTP_QUERY_SET_COOKIE, WINHTTP_HEADER_NAME_BY_INDEX, nullptr, &cookieSize, WINHTTP_NO_HEADER_INDEX);
            if (GetLastError() == ERROR_INSUFFICIENT_BUFFER && cookieSize > 0)
            {
                std::wstring raw(cookieSize / sizeof(wchar_t), 0);
                if (WinHttpQueryHeaders(request, WINHTTP_QUERY_SET_COOKIE, WINHTTP_HEADER_NAME_BY_INDEX, &raw[0], &cookieSize, WINHTTP_NO_HEADER_INDEX))
                {
                    size_t semicolon = raw.find(L';');
                    cookie = semicolon == std::wstring::npos ? raw : raw.substr(0, semicolon);
                }
            }

            std::string response;
            for (;;)
            {
                DWORD available = 0;
                if (!WinHttpQueryDataAvailable(request, &available) || available == 0) break;
                std::string chunk(available, 0);
                DWORD read = 0;
                if (!WinHttpReadData(request, &chunk[0], available, &read) || read == 0) break;
                chunk.resize(read);
                response += chunk;
            }

            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return ToWide(response);
        }
    };

    std::wstring ExtractAcfValue(const std::wstring& text, const std::wstring& key)
    {
        std::wregex pattern(L"\"" + key + L"\"\\s+\"([^\"]*)\"", std::regex_constants::icase);
        std::wsmatch match;
        return std::regex_search(text, match, pattern) ? match[1].str() : L"";
    }

    bool DirectoryExists(const std::wstring& path)
    {
        DWORD attrs = GetFileAttributesW(path.c_str());
        return attrs != INVALID_FILE_ATTRIBUTES && (attrs & FILE_ATTRIBUTE_DIRECTORY);
    }

    bool FileExists(const std::wstring& path)
    {
        DWORD attrs = GetFileAttributesW(path.c_str());
        return attrs != INVALID_FILE_ATTRIBUTES && !(attrs & FILE_ATTRIBUTE_DIRECTORY);
    }

    std::vector<std::wstring> SteamInstalledAppFolders(const std::wstring& steamapps, const std::vector<std::wstring>& excludeKeywords)
    {
        std::vector<std::wstring> result;
        WIN32_FIND_DATAW data{};
        HANDLE find = FindFirstFileW((steamapps + L"\\appmanifest_*.acf").c_str(), &data);
        if (find == INVALID_HANDLE_VALUE) return result;

        do
        {
            std::wstring text = ToWide(ReadFileUtf8(steamapps + L"\\" + data.cFileName));
            std::wstring name = ExtractAcfValue(text, L"name");
            std::wstring installDir = ExtractAcfValue(text, L"installdir");
            if (installDir.empty()) continue;

            std::wstring haystack = Lower(name + L" " + installDir);
            bool excluded = false;
            for (const auto& keyword : excludeKeywords)
            {
                if (haystack.find(Lower(keyword)) != std::wstring::npos)
                {
                    excluded = true;
                    break;
                }
            }
            if (excluded) continue;

            std::wstring folder = steamapps + L"\\common\\" + installDir;
            if (DirectoryExists(folder)) result.push_back(NormalizePath(folder));
        } while (FindNextFileW(find, &data));

        FindClose(find);
        return result;
    }

    std::vector<std::wstring> BuildDetectionFolders(const AppConfig& config)
    {
        std::set<std::wstring> folders;
        for (const auto& folder : config.gameFolders)
        {
            std::wstring normalized = NormalizePath(folder);
            size_t slash = normalized.find_last_of(L'\\');
            std::wstring leaf = slash == std::wstring::npos ? normalized : normalized.substr(slash + 1);
            if (leaf == L"steamapps")
            {
                for (const auto& appFolder : SteamInstalledAppFolders(folder, config.excludeSteamAppKeywords))
                {
                    folders.insert(appFolder);
                }
            }
            else
            {
                folders.insert(normalized);
            }
        }
        return std::vector<std::wstring>(folders.begin(), folders.end());
    }

    std::vector<std::wstring> ScanGameLibraries()
    {
        std::set<std::wstring> folders;
        std::vector<std::wstring> candidates;

        wchar_t programFiles[MAX_PATH]{};
        SHGetFolderPathW(nullptr, CSIDL_PROGRAM_FILES, nullptr, SHGFP_TYPE_CURRENT, programFiles);
        wchar_t programFilesX86[MAX_PATH]{};
        SHGetFolderPathW(nullptr, CSIDL_PROGRAM_FILESX86, nullptr, SHGFP_TYPE_CURRENT, programFilesX86);
        wchar_t commonData[MAX_PATH]{};
        SHGetFolderPathW(nullptr, CSIDL_COMMON_APPDATA, nullptr, SHGFP_TYPE_CURRENT, commonData);

        candidates.push_back(std::wstring(programFilesX86) + L"\\Steam\\steamapps\\libraryfolders.vdf");
        candidates.push_back(std::wstring(programFiles) + L"\\Steam\\steamapps\\libraryfolders.vdf");

        DWORD drives = GetLogicalDrives();
        for (int i = 0; i < 26; ++i)
        {
            if (!(drives & (1 << i))) continue;
            wchar_t root[] = { (wchar_t)(L'A' + i), L':', L'\\', 0 };
            if (GetDriveTypeW(root) == DRIVE_NO_ROOT_DIR) continue;
            candidates.push_back(std::wstring(root) + L"SteamLibrary\\steamapps\\libraryfolders.vdf");
            for (const auto& rel : { L"SteamLibrary\\steamapps", L"Games", L"Epic Games", L"GOG Games", L"WeGameApps", L"XboxGames" })
            {
                std::wstring folder = std::wstring(root) + rel;
                if (DirectoryExists(folder)) folders.insert(folder);
            }
        }

        for (const auto& file : candidates)
        {
            if (!FileExists(file)) continue;
            size_t slash = file.find_last_of(L'\\');
            if (slash != std::wstring::npos) folders.insert(file.substr(0, slash));
            std::wstring text = ToWide(ReadFileUtf8(file));
            std::wregex pattern(L"\"path\"\\s+\"([^\"]+)\"", std::regex_constants::icase);
            for (auto it = std::wsregex_iterator(text.begin(), text.end(), pattern); it != std::wsregex_iterator(); ++it)
            {
                std::wstring root = (*it)[1].str();
                std::replace(root.begin(), root.end(), L'/', L'\\');
                folders.insert(root + L"\\steamapps");
            }
        }

        std::wstring epicManifests = std::wstring(commonData) + L"\\Epic\\EpicGamesLauncher\\Data\\Manifests\\*.item";
        WIN32_FIND_DATAW data{};
        HANDLE find = FindFirstFileW(epicManifests.c_str(), &data);
        if (find != INVALID_HANDLE_VALUE)
        {
            std::wstring dir = std::wstring(commonData) + L"\\Epic\\EpicGamesLauncher\\Data\\Manifests\\";
            do
            {
                std::wstring text = ToWide(ReadFileUtf8(dir + data.cFileName));
                std::wregex pattern(L"\"InstallLocation\"\\s*:\\s*\"([^\"]+)\"", std::regex_constants::icase);
                std::wsmatch match;
                if (std::regex_search(text, match, pattern))
                {
                    std::wstring folder = match[1].str();
                    std::replace(folder.begin(), folder.end(), L'/', L'\\');
                    if (DirectoryExists(folder)) folders.insert(folder);
                }
            } while (FindNextFileW(find, &data));
            FindClose(find);
        }

        std::vector<std::wstring> result(folders.begin(), folders.end());
        std::sort(result.begin(), result.end());
        return result;
    }

    class GameDetector
    {
    public:
        explicit GameDetector(const AppConfig& config)
            : folders(BuildDetectionFolders(config))
        {
            for (auto item : config.gameProcesses) processNames.insert(Lower(item));
            for (auto item : config.excludeProcesses) excludedNames.insert(Lower(item));
            for (auto item : config.excludePathKeywords) excludedPathKeywords.push_back(Lower(item));
        }

        std::wstring Detect()
        {
            HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if (snapshot == INVALID_HANDLE_VALUE) return L"";

            PROCESSENTRY32W entry{};
            entry.dwSize = sizeof(entry);
            std::set<DWORD> live;
            std::wstring detected;

            if (Process32FirstW(snapshot, &entry))
            {
                do
                {
                    live.insert(entry.th32ProcessID);
                    std::wstring exe = Lower(entry.szExeFile);
                    std::wstring stem = exe;
                    if (stem.size() > 4 && stem.substr(stem.size() - 4) == L".exe") stem.resize(stem.size() - 4);
                    if (excludedNames.count(exe) || excludedNames.count(stem)) continue;
                    if (processNames.count(exe) || processNames.count(stem))
                    {
                        detected = exe;
                        break;
                    }

                    std::wstring path = ProcessPath(entry.th32ProcessID, exe);
                    if (path.empty()) continue;
                    std::wstring normalized = NormalizePath(path);

                    bool excluded = false;
                    for (const auto& keyword : excludedPathKeywords)
                    {
                        if (normalized.find(keyword) != std::wstring::npos)
                        {
                            excluded = true;
                            break;
                        }
                    }
                    if (excluded) continue;

                    for (const auto& folder : folders)
                    {
                        if (StartsWithFolder(path, folder))
                        {
                            detected = path;
                            break;
                        }
                    }
                    if (!detected.empty()) break;
                } while (Process32NextW(snapshot, &entry));
            }

            CloseHandle(snapshot);
            if (cache.size() > 256)
            {
                for (auto it = cache.begin(); it != cache.end();)
                {
                    if (!live.count(it->first)) it = cache.erase(it);
                    else ++it;
                }
            }
            return detected;
        }

    private:
        std::vector<std::wstring> folders;
        std::set<std::wstring> processNames;
        std::set<std::wstring> excludedNames;
        std::vector<std::wstring> excludedPathKeywords;
        std::map<DWORD, std::pair<std::wstring, std::wstring>> cache;

        std::wstring ProcessPath(DWORD pid, const std::wstring& exe)
        {
            auto found = cache.find(pid);
            if (found != cache.end() && found->second.first == exe) return found->second.second;

            std::wstring path;
            HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
            if (process)
            {
                wchar_t buffer[MAX_PATH * 4]{};
                DWORD size = MAX_PATH * 4;
                if (QueryFullProcessImageNameW(process, 0, buffer, &size)) path.assign(buffer, size);
                CloseHandle(process);
            }
            cache[pid] = std::make_pair(exe, path);
            return path;
        }
    };

    class Monitor
    {
    public:
        std::atomic<bool> running{ false };
        std::atomic<bool> stopping{ false };

        template <typename StatusCallback, typename DetectCallback, typename StopCallback>
        void Start(AppConfig config, StatusCallback status, DetectCallback detected, StopCallback stopped)
        {
            if (running) return;
            running = true;
            stopping = false;
            worker = std::thread([=]() mutable
            {
                bool appEnabledSpeedLimits = false;
                bool hadState = false;
                bool lastGameRunning = false;
                std::wstring lastDetected;

                try
                {
                    QbeeClient client(config);
                    GameDetector detector(config);
                    Log(config, L"Monitor started.");
                    status(L"监控中");

                    while (!stopping)
                    {
                        std::wstring current = detector.Detect();
                        bool gameRunning = !current.empty();
                        if (current != lastDetected)
                        {
                            lastDetected = current;
                            detected(current);
                        }

                        if (!hadState || gameRunning != lastGameRunning)
                        {
                            if (gameRunning)
                            {
                                bool alreadyEnabled = client.SpeedLimitsEnabled();
                                if (!alreadyEnabled)
                                {
                                    client.SetSpeedLimits(true);
                                    appEnabledSpeedLimits = true;
                                }
                                else
                                {
                                    appEnabledSpeedLimits = false;
                                }
                                status(alreadyEnabled ? L"检测到游戏运行，备用速度限制原本已打开。" : L"检测到游戏运行，已打开备用速度限制。");
                                Log(config, L"Game detected: " + current);
                            }
                            else if (hadState)
                            {
                                if (appEnabledSpeedLimits)
                                {
                                    client.SetSpeedLimits(false);
                                    status(L"检测到游戏退出，已关闭备用速度限制。");
                                }
                                else
                                {
                                    status(L"检测到游戏退出，保留原本已打开的备用速度限制。");
                                }
                                appEnabledSpeedLimits = false;
                            }
                            hadState = true;
                            lastGameRunning = gameRunning;
                        }

                        for (int i = 0; i < std::max(1, config.checkIntervalSeconds) * 10 && !stopping; ++i)
                        {
                            std::this_thread::sleep_for(std::chrono::milliseconds(100));
                        }
                    }

                    detected(L"");
                    if (config.restoreOnExit && appEnabledSpeedLimits)
                    {
                        client.SetSpeedLimits(false);
                    }
                }
                catch (const std::exception& error)
                {
                    status(L"监控出错：" + ToWide(error.what()));
                    Log(config, L"Monitor error: " + ToWide(error.what()));
                }

                running = false;
                stopping = false;
                stopped();
            });
            worker.detach();
        }

        void Stop()
        {
            stopping = true;
        }

    private:
        std::thread worker;
    };

    AppConfig g_config;
    Monitor g_monitor;
    HWND g_main = nullptr;
    HWND g_url = nullptr;
    HWND g_user = nullptr;
    HWND g_password = nullptr;
    HWND g_interval = nullptr;
    HWND g_startup = nullptr;
    HWND g_autoStart = nullptr;
    HWND g_folders = nullptr;
    HWND g_status = nullptr;
    HWND g_detected = nullptr;
    HWND g_start = nullptr;
    HWND g_stop = nullptr;
    HFONT g_font = nullptr;
    HFONT g_titleFont = nullptr;
    HBRUSH g_bgBrush = nullptr;
    HBRUSH g_cardBrush = nullptr;
    HBRUSH g_inputBrush = nullptr;
    std::atomic<bool> g_closingAfterStop{ false };

    std::wstring GetText(HWND hwnd)
    {
        int length = GetWindowTextLengthW(hwnd);
        std::wstring value(length, 0);
        GetWindowTextW(hwnd, &value[0], length + 1);
        return value;
    }

    void SetStatus(const std::wstring& text)
    {
        if (g_main) PostMessageW(g_main, WM_APP + 1, 0, (LPARAM)new std::wstring(text));
    }

    void SetDetected(const std::wstring& text)
    {
        if (g_main) PostMessageW(g_main, WM_APP + 2, 0, (LPARAM)new std::wstring(text));
    }

    void NotifyStopped()
    {
        if (g_main) PostMessageW(g_main, WM_APP + 3, 0, 0);
    }

    void SetStartupEnabled(bool enabled)
    {
        HKEY key{};
        if (RegCreateKeyExW(HKEY_CURRENT_USER, RunKeyPath, 0, nullptr, 0, KEY_SET_VALUE, nullptr, &key, nullptr) != ERROR_SUCCESS) return;
        if (enabled)
        {
            wchar_t path[MAX_PATH]{};
            GetModuleFileNameW(nullptr, path, MAX_PATH);
            std::wstring command = L"\"" + std::wstring(path) + L"\"";
            RegSetValueExW(key, RunValueName, 0, REG_SZ, (const BYTE*)command.c_str(), (DWORD)((command.size() + 1) * sizeof(wchar_t)));
        }
        else
        {
            RegDeleteValueW(key, RunValueName);
        }
        RegCloseKey(key);
    }

    bool StartupEnabled()
    {
        HKEY key{};
        bool enabled = false;
        if (RegOpenKeyExW(HKEY_CURRENT_USER, RunKeyPath, 0, KEY_QUERY_VALUE, &key) == ERROR_SUCCESS)
        {
            enabled = RegQueryValueExW(key, RunValueName, nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS;
            RegCloseKey(key);
        }
        return enabled;
    }

    void AddFolderToList(const std::wstring& folder)
    {
        SendMessageW(g_folders, LB_ADDSTRING, 0, (LPARAM)folder.c_str());
    }

    void LoadConfigToUi()
    {
        SetWindowTextW(g_url, g_config.qbeeUrl.c_str());
        SetWindowTextW(g_user, g_config.username.c_str());
        SetWindowTextW(g_password, g_config.password.c_str());
        SetWindowTextW(g_interval, std::to_wstring(g_config.checkIntervalSeconds).c_str());
        SendMessageW(g_startup, BM_SETCHECK, StartupEnabled() ? BST_CHECKED : BST_UNCHECKED, 0);
        SendMessageW(g_autoStart, BM_SETCHECK, g_config.autoStartMonitor ? BST_CHECKED : BST_UNCHECKED, 0);
        SendMessageW(g_folders, LB_RESETCONTENT, 0, 0);
        for (const auto& folder : g_config.gameFolders) AddFolderToList(folder);
    }

    bool SaveFromUi(bool showMessage)
    {
        std::wstring url = Trim(GetText(g_url));
        URL_COMPONENTS parts{};
        parts.dwStructSize = sizeof(parts);
        if (!WinHttpCrackUrl(url.c_str(), 0, 0, &parts) ||
            (parts.nScheme != INTERNET_SCHEME_HTTP && parts.nScheme != INTERNET_SCHEME_HTTPS))
        {
            MessageBoxW(g_main, L"请输入有效的 qB Web UI 地址，例如 http://127.0.0.1:8080。", AppTitle, MB_ICONWARNING);
            return false;
        }

        int count = (int)SendMessageW(g_folders, LB_GETCOUNT, 0, 0);
        if (count <= 0)
        {
            MessageBoxW(g_main, L"请至少添加一个游戏库文件夹，或点击“自动扫描”。", AppTitle, MB_ICONWARNING);
            return false;
        }

        g_config.qbeeUrl = url;
        g_config.username = GetText(g_user);
        g_config.password = GetText(g_password);
        g_config.checkIntervalSeconds = std::max(1, _wtoi(GetText(g_interval).c_str()));
        g_config.startWithWindows = SendMessageW(g_startup, BM_GETCHECK, 0, 0) == BST_CHECKED;
        g_config.autoStartMonitor = SendMessageW(g_autoStart, BM_GETCHECK, 0, 0) == BST_CHECKED;
        g_config.gameFolders.clear();
        for (int i = 0; i < count; ++i)
        {
            int length = (int)SendMessageW(g_folders, LB_GETTEXTLEN, i, 0);
            std::wstring item(length, 0);
            SendMessageW(g_folders, LB_GETTEXT, i, (LPARAM)&item[0]);
            g_config.gameFolders.push_back(item);
        }
        SaveConfig(g_config);
        SetStartupEnabled(g_config.startWithWindows);
        if (showMessage) SetWindowTextW(g_status, L"已保存");
        return true;
    }

    HWND CreateChild(const wchar_t* cls, const wchar_t* text, DWORD style, int x, int y, int w, int h, int id)
    {
        HWND hwnd = CreateWindowExW(0, cls, text, WS_CHILD | WS_VISIBLE | style, x, y, w, h, g_main, (HMENU)(INT_PTR)id, GetModuleHandleW(nullptr), nullptr);
        SendMessageW(hwnd, WM_SETFONT, (WPARAM)g_font, TRUE);
        return hwnd;
    }

    HWND CreateButton(const wchar_t* text, int x, int y, int w, int h, int id)
    {
        return CreateChild(L"BUTTON", text, BS_PUSHBUTTON | BS_FLAT, x, y, w, h, id);
    }

    HWND CreateLabel(const wchar_t* text, int x, int y, int w, int h, HFONT font = nullptr)
    {
        HWND hwnd = CreateChild(L"STATIC", text, SS_LEFT, x, y, w, h, 0);
        SendMessageW(hwnd, WM_SETFONT, (WPARAM)(font ? font : g_font), TRUE);
        return hwnd;
    }

    void StartMonitoring()
    {
        if (!SaveFromUi(false)) return;
        EnableWindow(g_start, FALSE);
        EnableWindow(g_stop, TRUE);
        SetWindowTextW(g_status, L"监控中");
        g_monitor.Start(g_config, SetStatus, SetDetected, NotifyStopped);
    }

    void StopMonitoring()
    {
        EnableWindow(g_stop, FALSE);
        EnableWindow(g_start, FALSE);
        SetWindowTextW(g_status, L"正在停止监控...");
        g_monitor.Stop();
    }

    void TestConnection()
    {
        if (!SaveFromUi(false)) return;
        SetWindowTextW(g_status, L"正在测试连接...");
        std::thread([]()
        {
            try
            {
                QbeeClient client(g_config);
                bool enabled = client.SpeedLimitsEnabled();
                SetStatus(enabled ? L"连接成功，备用速度限制当前已打开。" : L"连接成功，备用速度限制当前已关闭。");
            }
            catch (const std::exception& error)
            {
                SetStatus(L"连接失败：" + ToWide(error.what()));
            }
        }).detach();
    }

    void ScanFoldersAsync()
    {
        SetWindowTextW(g_status, L"正在扫描游戏库...");
        std::thread([]()
        {
            auto found = ScanGameLibraries();
            PostMessageW(g_main, WM_APP + 4, 0, (LPARAM)new std::vector<std::wstring>(found));
        }).detach();
    }

    void AddFolder()
    {
        BROWSEINFOW info{};
        info.hwndOwner = g_main;
        info.lpszTitle = L"选择游戏库文件夹";
        info.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;
        PIDLIST_ABSOLUTE pid = SHBrowseForFolderW(&info);
        if (!pid) return;
        wchar_t path[MAX_PATH]{};
        if (SHGetPathFromIDListW(pid, path)) AddFolderToList(path);
        CoTaskMemFree(pid);
    }

    LRESULT CALLBACK WindowProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
    {
        switch (message)
        {
        case WM_CREATE:
        {
            g_main = hwnd;
            g_bgBrush = CreateSolidBrush(RGB(246, 248, 251));
            g_cardBrush = CreateSolidBrush(RGB(255, 255, 255));
            g_inputBrush = CreateSolidBrush(RGB(249, 251, 253));
            g_font = CreateFontW(18, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");
            g_titleFont = CreateFontW(30, 0, 0, 0, FW_SEMIBOLD, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");

            CreateLabel(AppTitle, 28, 22, 360, 40, g_titleFont);
            CreateLabel(L"低占用原生版：检测游戏运行并自动切换 qB 备用速度限制", 30, 62, 520, 24);
            g_status = CreateLabel(L"就绪", 660, 32, 190, 30);

            CreateLabel(L"连接", 34, 108, 120, 24);
            CreateLabel(L"地址", 48, 146, 58, 24);
            g_url = CreateChild(L"EDIT", L"", WS_BORDER | ES_AUTOHSCROLL, 110, 140, 720, 30, IdUrl);
            CreateLabel(L"用户名", 48, 188, 58, 24);
            g_user = CreateChild(L"EDIT", L"", WS_BORDER | ES_AUTOHSCROLL, 110, 182, 260, 30, IdUser);
            CreateLabel(L"密码", 402, 188, 58, 24);
            g_password = CreateChild(L"EDIT", L"", WS_BORDER | ES_AUTOHSCROLL | ES_PASSWORD, 462, 182, 260, 30, IdPassword);
            CreateLabel(L"间隔", 48, 231, 58, 24);
            g_interval = CreateChild(L"EDIT", L"", WS_BORDER | ES_NUMBER, 110, 225, 70, 30, IdInterval);
            CreateLabel(L"秒", 188, 231, 40, 24);
            CreateButton(L"测试连接", 240, 223, 104, 34, IdTest);
            g_startup = CreateChild(L"BUTTON", L"开机自启动", BS_AUTOCHECKBOX, 390, 228, 130, 26, IdStartWithWindows);
            g_autoStart = CreateChild(L"BUTTON", L"启动后自动开始监控", BS_AUTOCHECKBOX, 530, 228, 210, 26, IdAutoStartMonitor);

            CreateLabel(L"游戏库", 34, 300, 120, 24);
            g_folders = CreateChild(L"LISTBOX", L"", WS_BORDER | LBS_NOTIFY | WS_VSCROLL | WS_HSCROLL, 48, 336, 640, 210, IdFolderList);
            CreateButton(L"自动扫描", 716, 336, 110, 34, IdScan);
            CreateButton(L"添加", 716, 380, 110, 34, IdAdd);
            CreateButton(L"删除", 716, 424, 110, 34, IdRemove);
            CreateButton(L"打开配置", 716, 468, 110, 34, IdOpenConfig);

            g_detected = CreateLabel(L"当前检测到：无", 48, 560, 780, 26);
            CreateButton(L"保存", 484, 600, 100, 36, IdSave);
            g_start = CreateButton(L"开始监控", 596, 600, 112, 36, IdStart);
            g_stop = CreateButton(L"停止监控", 720, 600, 112, 36, IdStop);
            EnableWindow(g_stop, FALSE);

            LoadConfigToUi();
            if (g_config.autoStartMonitor) PostMessageW(hwnd, WM_COMMAND, IdStart, 0);
            return 0;
        }
        case WM_CTLCOLORSTATIC:
        {
            HDC dc = (HDC)wParam;
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, RGB(40, 52, 70));
            return (LRESULT)g_bgBrush;
        }
        case WM_CTLCOLOREDIT:
        case WM_CTLCOLORLISTBOX:
        {
            HDC dc = (HDC)wParam;
            SetTextColor(dc, RGB(40, 52, 70));
            SetBkColor(dc, RGB(249, 251, 253));
            return (LRESULT)g_inputBrush;
        }
        case WM_ERASEBKGND:
        {
            HDC dc = (HDC)wParam;
            RECT rect{};
            GetClientRect(hwnd, &rect);
            FillRect(dc, &rect, g_bgBrush);
            RECT card1{ 28, 96, 850, 272 };
            RECT card2{ 28, 288, 850, 552 };
            FillRect(dc, &card1, g_cardBrush);
            FillRect(dc, &card2, g_cardBrush);
            return 1;
        }
        case WM_COMMAND:
        {
            switch (LOWORD(wParam))
            {
            case IdTest: TestConnection(); break;
            case IdScan: ScanFoldersAsync(); break;
            case IdAdd: AddFolder(); break;
            case IdRemove:
            {
                int index = (int)SendMessageW(g_folders, LB_GETCURSEL, 0, 0);
                if (index != LB_ERR) SendMessageW(g_folders, LB_DELETESTRING, index, 0);
                break;
            }
            case IdOpenConfig:
                ShellExecuteW(hwnd, L"open", L"explorer.exe", AppDirectory().c_str(), nullptr, SW_SHOWNORMAL);
                break;
            case IdSave:
                SaveFromUi(true);
                break;
            case IdStart:
                StartMonitoring();
                break;
            case IdStop:
                StopMonitoring();
                break;
            }
            return 0;
        }
        case WM_APP + 1:
        {
            std::wstring* text = (std::wstring*)lParam;
            SetWindowTextW(g_status, text->c_str());
            delete text;
            return 0;
        }
        case WM_APP + 2:
        {
            std::wstring* text = (std::wstring*)lParam;
            std::wstring label = text->empty() ? L"当前检测到：无" : L"当前检测到：" + *text;
            SetWindowTextW(g_detected, label.c_str());
            delete text;
            return 0;
        }
        case WM_APP + 3:
            EnableWindow(g_start, TRUE);
            EnableWindow(g_stop, FALSE);
            if (g_closingAfterStop)
            {
                DestroyWindow(hwnd);
            }
            return 0;
        case WM_APP + 4:
        {
            auto* found = (std::vector<std::wstring>*)lParam;
            std::set<std::wstring> existing;
            int count = (int)SendMessageW(g_folders, LB_GETCOUNT, 0, 0);
            for (int i = 0; i < count; ++i)
            {
                int length = (int)SendMessageW(g_folders, LB_GETTEXTLEN, i, 0);
                std::wstring item(length, 0);
                SendMessageW(g_folders, LB_GETTEXT, i, (LPARAM)&item[0]);
                existing.insert(Lower(item));
            }
            int added = 0;
            for (const auto& folder : *found)
            {
                if (!existing.count(Lower(folder)))
                {
                    AddFolderToList(folder);
                    ++added;
                }
            }
            delete found;
            SetWindowTextW(g_status, added > 0 ? (L"自动扫描完成，新增 " + std::to_wstring(added) + L" 个游戏库。").c_str() : L"自动扫描完成，没有发现新的游戏库。");
            return 0;
        }
        case WM_CLOSE:
            if (g_monitor.running)
            {
                int choice = MessageBoxW(hwnd, L"监控仍在运行。要停止监控并退出吗？", AppTitle, MB_YESNO | MB_ICONQUESTION);
                if (choice != IDYES) return 0;
                g_closingAfterStop = true;
                StopMonitoring();
                return 0;
            }
            DestroyWindow(hwnd);
            return 0;
        case WM_DESTROY:
            if (g_font) DeleteObject(g_font);
            if (g_titleFont) DeleteObject(g_titleFont);
            if (g_bgBrush) DeleteObject(g_bgBrush);
            if (g_cardBrush) DeleteObject(g_cardBrush);
            if (g_inputBrush) DeleteObject(g_inputBrush);
            PostQuitMessage(0);
            return 0;
        }
        return DefWindowProcW(hwnd, message, wParam, lParam);
    }
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int show)
{
    HANDLE mutex = CreateMutexW(nullptr, TRUE, L"QbeeGameSpeedLimiter.Native.SingleInstance");
    if (GetLastError() == ERROR_ALREADY_EXISTS)
    {
        MessageBoxW(nullptr, L"qbee 游戏限速助手已经在运行。", AppTitle, MB_OK | MB_ICONINFORMATION);
        return 0;
    }

    INITCOMMONCONTROLSEX controls{ sizeof(controls), ICC_STANDARD_CLASSES };
    InitCommonControlsEx(&controls);
    g_config = LoadConfig();

    WNDCLASSW cls{};
    cls.hInstance = instance;
    cls.lpszClassName = L"QbeeGameSpeedLimiterWindow";
    cls.lpfnWndProc = WindowProc;
    cls.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    cls.hbrBackground = nullptr;
    RegisterClassW(&cls);

    HWND hwnd = CreateWindowExW(
        0,
        cls.lpszClassName,
        AppTitle,
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        900,
        700,
        nullptr,
        nullptr,
        instance,
        nullptr);

    ShowWindow(hwnd, show);
    UpdateWindow(hwnd);

    MSG msg{};
    while (GetMessageW(&msg, nullptr, 0, 0))
    {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    if (mutex) CloseHandle(mutex);
    return 0;
}
