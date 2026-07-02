# qbee 游戏限速助手

[English](README.md) | 简体中文

![程序图标](docs/app-icon.png)

玩游戏时自动降低下载器下载和上传速度，退出游戏后自动恢复。适合一边挂 BT / PT / 磁力下载，一边玩联机游戏、语音聊天或直播的 Windows 用户。

[立即下载 Windows 版](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/latest) · [第一次使用教程](#第一次使用) · [支持的下载客户端](#支持的下载客户端) · [常见问题](#常见问题)

![配置界面截图](docs/config-ui.png)

## 它解决什么问题？

后台下载器正在下载电影、游戏或 PT 资源时，上传和下载可能占满带宽，导致 CS2、Valorant、Minecraft、幻兽帕鲁等联机游戏延迟升高、丢包，语音也可能卡顿。

本工具会在检测到游戏运行时自动进入“游戏限速模式”，游戏退出后再恢复原来的下载器状态。你不需要每次进游戏前手动切换限速。

## 支持的下载客户端

| 客户端 | 当前支持状态 | 游戏中如何限速 |
| --- | --- | --- |
| qBittorrent / qBittorrent Enhanced Edition | 已支持 | 自动切换备用速度限制 |
| Transmission | 已支持 | 自动切换 `alt-speed-enabled` 备用限速 |
| aria2 / Motrix | 已支持 | 临时修改全局上下行限速，退出游戏后恢复 |
| µTorrent / BitTorrent Classic | 已支持 | 临时修改全局上下行限速，退出游戏后恢复 |
| Deluge | 已支持 | 临时修改全局上下行限速，退出游戏后恢复 |
| BitComet / 比特彗星 | 已加入主流客户端列表，暂不自动控制 | 当前缺少稳定公开的远程限速 API，本版会明确提示，不会假装成功 |

如果你主要用 BitComet，当前版本可以先作为游戏检测和状态工具使用；自动限速需要后续走 Windows 级进程限速方案，或等待 BitComet 提供稳定远程接口。

## 主要特点

- 支持 Steam / Epic / Xbox / Battle.net / EA / Ubisoft / WeGame 等常见游戏目录。
- 支持 qB、Transmission、aria2 / Motrix、µTorrent / BitTorrent Classic、Deluge，并对 BitComet 给出明确状态说明。
- 支持 Windows 开机自启动。
- 后台监控独立运行，低占用。
- 配置界面用完可关，不影响后台监控。
- 不会覆盖你原本手动开启的 qB / Transmission 备用限速状态。
- 对 aria2、µTorrent / BitTorrent、Deluge 会记录原来的全局限速，游戏退出后恢复。
- 可一键创建桌面入口，减少下次查找程序的麻烦。
- 内置“运行自检”，能提示文件缺失、地址错误、游戏库路径无效、后台状态异常等常见问题。
- 附带便携安装/卸载脚本，不需要管理员权限。

## 下载哪个文件

打开 [Releases 页面](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases)，下载：

```text
qbee-game-speed-limiter-windows.zip
```

解压后应看到这些用户文件：

```text
qbee_limiter_config.exe
qbee_limiter_monitor.exe
qbee_game_speed_limiter.json
install.ps1
uninstall.ps1
README.zh-CN.md
LICENSE
```

推荐先右键运行 `install.ps1`（或在 PowerShell 中运行），它会创建桌面和开始菜单入口；也可以直接双击 `qbee_limiter_config.exe` 使用。`qbee_limiter_monitor.exe` 是后台监控程序，不需要你手动配置它。

## 第一次使用

### 1. 打开下载器的远程控制接口

qBittorrent / qBEE：进入 `工具 -> 选项 -> WebUI`，启用 Web UI。常见地址是：

```text
http://127.0.0.1:8080
```

Transmission：启用 Remote / RPC，常见地址是：

```text
http://127.0.0.1:9091/transmission/rpc
```

aria2 / Motrix：启用 JSON-RPC，常见地址是：

```text
http://127.0.0.1:6800/jsonrpc
```

µTorrent / BitTorrent Classic：启用 Web UI，地址通常类似：

```text
http://127.0.0.1:8080/gui
```

Deluge：启用 Deluge Web，JSON 地址通常是：

```text
http://127.0.0.1:8112/json
```

BitComet / 比特彗星：当前没有稳定公开的远程限速 API，本工具会提示当前限制，暂不自动限速。

### 2. 安装或打开配置界面

推荐方式：右键 `install.ps1`，选择“使用 PowerShell 运行”。它会创建桌面入口和开始菜单入口。

如果你不想安装，也可以直接双击：

```text
qbee_limiter_config.exe
```

它会在浏览器里打开一个本地配置页面。

### 3. 选择下载客户端并测试连接

在“下载客户端”里选择你正在使用的客户端。

- qB：填写 qB Web UI 地址、用户名、密码。
- Transmission：填写 Transmission RPC 地址、用户名、密码。
- aria2 / Motrix：填写 JSON-RPC 地址；如果设置了 RPC Secret，也要填入。
- µTorrent / BitTorrent Classic：填写 Web UI 地址、用户名、密码。
- Deluge：填写 Deluge Web JSON 地址；通常只需要填写密码。
- BitComet：本版会明确提示暂不支持自动限速。

如果 qB 设置了 localhost 免验证，可以先把用户名和密码留空，然后点“测试连接”。

注意：这里填的是下载器远程控制信息，不是 PeerBanHelper 的 Token，也不是 GitHub 密码。

### 4. 添加游戏库目录

点击“自动扫描”。

程序会尝试寻找 Steam、Epic、GOG、WeGame、XboxGames、Battle.net、EA、Ubisoft 和常见游戏目录。

如果没有扫到你的游戏目录，可以手动添加。例如：

```text
C:\Program Files (x86)\Steam\steamapps
D:\SteamLibrary\steamapps
```

Steam 用户建议添加到 `steamapps` 这一层，不要只添加某一个游戏 exe。

### 5. 保存、启动、创建入口

推荐顺序：

1. 点击“测试连接”。
2. 看到连接成功后，点击“自动扫描”。
3. 勾选“保存后自动启动监控”。
4. 点击“保存并应用”。
5. 点击“运行自检”，确认文件、地址、游戏库和后台状态没有明显问题。
6. 点击“创建桌面入口”，以后从桌面打开配置器。

保存成功后，后台监控程序会自动启动。之后你可以关闭浏览器里的配置页面。

## 平时怎么用

- 正常打开下载器。
- 正常打开游戏。
- 检测到游戏运行后，工具会自动进入游戏限速模式。
- 游戏退出后，工具会自动恢复它改动过的下载器状态。

如果游戏启动前你已经手动打开了 qB / Transmission 备用限速，工具会尊重你的手动设置，游戏退出后不会帮你关掉。对 aria2、µTorrent / BitTorrent、Deluge 这类全局限速型客户端，工具会在进入游戏前记录原值，游戏退出后恢复。

## 运行自检会检查什么？

配置页面里的“运行自检”会检查：

- 两个 exe 是否在同一目录。
- 配置文件是否存在。
- 当前下载客户端地址是否以 `http://` 或 `https://` 开头。
- 游戏库目录是否为空，路径是否真实存在。
- 后台监控是否正在运行，状态是否长时间未更新。
- 是否启用了开机自启动。
- 当前客户端是否需要额外准备远程接口，BitComet 是否只能提示、不能自动控制。

如果你不确定哪里填错了，先点“运行自检”。

## 常见问题

### qB 下载为什么会影响游戏延迟？

BT 下载和上传会占用带宽，尤其是上传跑满时，可能导致游戏数据包排队，出现高延迟、丢包、语音卡顿。本工具的作用是自动切换下载器到较低速度。

### qB 点“测试连接”失败

先在浏览器里打开你填写的地址，例如：

```text
http://127.0.0.1:8080
```

如果浏览器也打不开，说明 qB Web UI 没开，或者端口填错了。

### qB 地址打开的是 CEF remote debugging

这通常是 Steam 占用了 `127.0.0.1:8080`。可以试：

```text
http://[::1]:8080
```

如果还不行，建议在 qB 的 WebUI 设置里把端口改成 `8081`，然后本工具里填写：

```text
http://127.0.0.1:8081
```

### 游戏中限速填多少？

对 aria2、µTorrent / BitTorrent、Deluge，建议从下载 `512 KiB/s`、上传 `128 KiB/s` 开始。网速较高可以适当调大；如果联机游戏仍然延迟高，就继续降低上传限速。

### BitComet 为什么不能自动限速？

BitComet / 比特彗星目前没有像 qB、Transmission、aria2、µTorrent、Deluge 那样稳定公开、适合自动化的远程限速 API。为了避免误导用户，本版会明确提示“暂不自动控制”，不会显示假成功。后续可以考虑 Windows 级进程限速方案来覆盖 BitComet。

### 自动扫描没有找到所有游戏

自动扫描只能找常见平台和常见目录。没有扫到时，手动添加游戏库目录即可。

### 关闭游戏后没有恢复

默认每 5 秒检查一次，所以可能会等几秒。若仍不恢复，请确认：

- 后台监控还在运行。
- 游戏进程确实已经退出。
- 游戏目录没有被误加到排除规则里。

## 隐私提醒

`qbee_game_speed_limiter.json` 会在本机保存下载器远程控制地址、用户名、密码或 aria2 secret。不要把自己的真实配置上传到 GitHub 或发给别人。

## 后续计划

后续更适合截图展示、也更能提升长期使用体验的功能包括：系统托盘图标、Windows 通知、最近触发记录、每个游戏单独限速值、Windows 级进程限速以覆盖 BitComet、迅雷等没有稳定公开 API 的客户端。

## 卸载

运行：

```powershell
.\uninstall.ps1
```

它会停止后台监控，删除桌面/开始菜单入口，并移除开机启动项。配置文件默认保留，你可以手动删除整个解压目录来彻底清理。

## 开发者

```powershell
.\build.ps1
```

## 许可证

MIT License
