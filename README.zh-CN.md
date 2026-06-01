# qBittorrent 游戏限速助手

[English](README.md) | 简体中文

玩游戏时自动降低 qBittorrent / qBittorrent Enhanced Edition 下载和上传速度，退出游戏后自动恢复。适合一边挂 BT / PT 下载，一边玩联机游戏、语音聊天或直播的 Windows 用户。

[立即下载 Windows 版](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/latest) · [查看第一次使用教程](#第一次使用) · [常见问题](#常见问题)

![配置界面截图](docs/config-ui.png)

## 它解决什么问题？

后台 qBittorrent 正在下载电影、游戏或 PT 资源时，上传和下载可能占满带宽，导致 CS2、Valorant、Minecraft、幻兽帕鲁等联机游戏延迟升高、丢包，语音也可能卡顿。

本工具会在检测到游戏运行时自动打开 qB 的“备用速度限制”，游戏退出后再自动恢复。你不需要每次进游戏前手动切换限速。

## 主要特点

- 支持 Steam / Epic / Xbox / Battle.net / EA / Ubisoft / WeGame 等常见游戏目录。
- 支持 qBittorrent 与 qBittorrent Enhanced Edition。
- 支持 Windows 开机自启动。
- 后台监控独立运行，低占用。
- 配置界面用完可关，不影响后台监控。
- 不会覆盖你原本手动开启的备用限速状态。
- 可自动扫描游戏库，也可以手动添加目录。

## 适合谁使用？

- 一边挂 BT / PT 下载，一边玩网游的玩家。
- qBittorrent Enhanced Edition 用户。
- 家庭宽带、NAS、影音下载用户。
- 经常遇到“后台下载导致游戏延迟飙升”的用户。
- 想让 qB 自动进入低速模式，但不想每次手动切换的人。

## 下载哪个文件

打开 [Releases 页面](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases)，下载：

```text
qbee-game-speed-limiter-windows.zip
```

解压后应看到 5 个文件：

```text
qbee_limiter_config.exe
qbee_limiter_monitor.exe
qbee_game_speed_limiter.json
README.zh-CN.md
LICENSE
```

平时主要打开 `qbee_limiter_config.exe`。`qbee_limiter_monitor.exe` 是后台监控程序，不需要你手动配置它。

## 第一次使用

### 1. 先打开 qB 的 Web UI

在 qBittorrent / qBittorrent Enhanced Edition 里进入：

```text
工具 -> 选项 -> WebUI
```

请确认：

- 勾选了“Web 用户界面 / Web User Interface”。
- 端口通常是 `8080`。
- 如果你勾选了“Bypass authentication for clients on localhost”，本工具通常可以不填密码。
- 如果没有勾选 localhost 免验证，请记住这里设置的用户名和密码。

### 2. 打开配置界面

双击：

```text
qbee_limiter_config.exe
```

它会在浏览器里打开一个本地配置页面。

### 3. 填 qB Web UI 信息

常见填写方式：

```text
地址：http://127.0.0.1:8080
用户名：你的 qB Web UI 用户名
密码：你的 qB Web UI 密码
```

如果 qB 设置了 localhost 免验证，可以先把用户名和密码留空，然后点“测试连接”。

注意：这里填的是 qBittorrent 的 Web UI 信息，不是 PeerBanHelper 的 Token，也不是 GitHub 密码。

### 4. 添加游戏库目录

点击“自动扫描”。

程序会尝试寻找 Steam、Epic、GOG、WeGame、XboxGames、Battle.net、EA、Ubisoft 和常见游戏目录。

如果没有扫到你的游戏目录，可以手动添加。例如：

```text
C:\Program Files (x86)\Steam\steamapps
D:\SteamLibrary\steamapps
```

Steam 用户建议添加到 `steamapps` 这一层，不要只添加某一个游戏 exe。

### 5. 测试并保存

推荐顺序：

1. 点击“测试连接”。
2. 看到连接成功后，点击“自动扫描”。
3. 勾选“保存后自动启动监控”。
4. 点击“保存并应用”。

保存成功后，后台监控程序会自动启动。之后你可以关闭浏览器里的配置页面。

## 平时怎么用

- 正常打开 qB。
- 正常打开游戏。
- 检测到游戏运行后，工具会打开 qB 的备用速度限制。
- 游戏退出后，工具会自动关闭它打开的限制。

如果游戏启动前你已经手动打开了备用速度限制，工具会尊重你的手动设置，游戏退出后不会帮你关掉。

## 开机自启动

在配置页面里打开“开机自启动”并保存。

开机后启动的是 `qbee_limiter_monitor.exe` 后台程序，不会弹出配置界面。

## 常见问题

### qB 下载为什么会影响游戏延迟？

BT 下载和上传会占用带宽，尤其是上传跑满时，可能导致游戏数据包排队，出现高延迟、丢包、语音卡顿。本工具的作用是自动切换 qBittorrent 备用限速模式，让游戏期间 qB 使用较低速度。

### 点“测试连接”失败

先在浏览器里打开你填写的地址，例如：

```text
http://127.0.0.1:8080
```

如果浏览器也打不开，说明 qB Web UI 没开，或者端口填错了。

### 打开的是 CEF remote debugging

这通常是 Steam 占用了 `127.0.0.1:8080`。可以试：

```text
http://[::1]:8080
```

如果还不行，建议在 qB 的 WebUI 设置里把端口改成 `8081`，然后本工具里填写：

```text
http://127.0.0.1:8081
```

### 密码一直错误

请确认你填的是 qB Web UI 的用户名和密码。qB 设置页面里的密码框如果显示 `Change current password`，不代表当前密码就是这串文字。

如果你只在本机使用，可以在 qB WebUI 设置里勾选 localhost 免验证，然后本工具里把用户名和密码留空再测试。

### 自动扫描没有找到所有游戏

自动扫描只能找常见平台和常见目录。没有扫到时，手动添加游戏库目录即可。

Steam 游戏请添加 `steamapps` 文件夹；其他平台一般添加装游戏的总文件夹。

### 关闭游戏后没有恢复

默认每 5 秒检查一次，所以可能会等几秒。若仍不恢复，请确认：

- 后台监控还在运行。
- 游戏进程确实已经退出。
- 游戏目录没有被误加到排除规则里。

## 隐私提醒

`qbee_game_speed_limiter.json` 会在本机保存 qB Web UI 地址、用户名和密码。不要把自己的真实配置上传到 GitHub 或发给别人。

## 后续计划

后续更适合截图展示、也更能提升长期使用体验的功能包括：系统托盘图标、当前状态展示、Windows 通知、最近触发记录、每个游戏单独限速值。

## 开发者

构建方式、设计稿、更新日志等开发资料放在仓库中，不再放进发行包。

```powershell
.\build.ps1
```

## 许可证

MIT License
