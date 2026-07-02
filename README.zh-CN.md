# 下载器游戏限速助手

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
| BitComet / 比特彗星 | 已支持（新版 WebUI） | 临时修改全局上下行限速，退出游戏后恢复 |

## 主要特点

- 支持 Steam / Epic / Xbox / Battle.net / EA / Ubisoft / WeGame 等常见游戏目录。
- 支持 qB、Transmission、aria2 / Motrix、µTorrent / BitTorrent Classic、Deluge、BitComet / 比特彗星等主流下载客户端。
- 支持 Windows 开机自启动。
- 后台监控独立运行，低占用。
- 配置界面用完可关，不影响后台监控。
- 不会覆盖你原本手动开启的 qB / Transmission 备用限速状态。
- 对 aria2、µTorrent / BitTorrent、Deluge、BitComet 会记录原来的全局限速，游戏退出后恢复。
- 可一键创建桌面入口，减少下次查找程序的麻烦。
- Windows exe 和快捷入口使用项目应用图标。
- 配置页会按所选下载客户端只显示相关输入项，减少误填。
- 内置“运行自检”，能提示文件缺失、地址错误、游戏库路径无效、后台状态异常等常见问题。
- 附带便携安装/卸载脚本，不需要管理员权限。

## 下载哪个文件

打开 [Releases 页面](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases)，下载：

```text
download-client-game-speed-limiter-windows.zip
```

解压后应看到这些用户文件：

```text
download_limiter_config.exe
download_limiter_monitor.exe
download_client_game_speed_limiter.json
install.cmd
install.ps1
uninstall.cmd
uninstall.ps1
README.zh-CN.md
LICENSE
```

推荐先双击 `install.cmd`，它会自动创建桌面和开始菜单入口，并打开配置界面；也可以直接双击 `download_limiter_config.exe` 便携使用。`download_limiter_monitor.exe` 是后台监控程序，不需要你手动配置它。

旧版 `qbee_game_speed_limiter.json` 会在首次启动时自动读取；安装脚本也会把旧配置复制成新文件名，升级时不需要重新填写账号密码。

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

BitComet / 比特彗星：启用远程访问 / WebUI。建议使用 BitComet 2.16 或更新版本。地址填写你在 BitComet 中设置的实际端口，例如：

```text
http://127.0.0.1:80
```

### 2. 安装或打开配置界面

推荐方式：双击 `install.cmd`。它会创建桌面入口和开始菜单入口，并自动打开配置界面。

如果你不想安装，也可以直接双击：

```text
download_limiter_config.exe
```

它会在浏览器里打开一个本地配置页面。

### 3. 选择下载客户端并测试连接

在“下载客户端”里选择你正在使用的客户端。

- qB：填写 qB Web UI 地址、用户名、密码。
- Transmission：填写 Transmission RPC 地址、用户名、密码。
- aria2 / Motrix：填写 JSON-RPC 地址；如果设置了 RPC Secret，也要填入。
- µTorrent / BitTorrent Classic：填写 Web UI 地址、用户名、密码。
- Deluge：填写 Deluge Web JSON 地址；通常只需要填写密码。
- BitComet：填写 BitComet WebUI 地址、用户名、密码，并设置游戏中的下载/上传限速。

配置页只会显示当前客户端需要的输入项。qB 和 Transmission 的具体限速值请在下载器自己的“备用限速”里设置；aria2、µTorrent / BitTorrent、Deluge、BitComet 使用配置页里的“游戏中下载/上传限速”。

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

如果游戏启动前你已经手动打开了 qB / Transmission 备用限速，工具会尊重你的手动设置，游戏退出后不会帮你关掉。对 aria2、µTorrent / BitTorrent、Deluge、BitComet 这类全局限速型客户端，工具会在进入游戏前记录原值，游戏退出后恢复。

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

对 aria2、µTorrent / BitTorrent、Deluge、BitComet，建议从下载 `512 KiB/s`、上传 `128 KiB/s` 开始。网速较高可以适当调大；如果联机游戏仍然延迟高，就继续降低上传限速。

### BitComet 怎么设置？

BitComet / 比特彗星需要先在客户端里启用远程访问 / WebUI，然后在本工具里选择 `BitComet / 比特彗星`，填写 WebUI 地址、用户名、密码。建议使用 BitComet 2.16 或更新版本。

如果“测试连接”失败，先在浏览器打开你填写的地址，确认能看到 BitComet WebUI；再检查端口、用户名、密码是否和 BitComet 设置一致。

### 自动扫描没有找到所有游戏

自动扫描只能找常见平台和常见目录。没有扫到时，手动添加游戏库目录即可。

### 关闭游戏后没有恢复

默认每 5 秒检查一次，所以可能会等几秒。若仍不恢复，请确认：

- 后台监控还在运行。
- 游戏进程确实已经退出。
- 游戏目录没有被误加到排除规则里。

## 隐私提醒

`download_client_game_speed_limiter.json` 会在本机保存下载器远程控制地址、用户名、密码或 aria2 secret。不要把自己的真实配置上传到 GitHub 或发给别人。

## 后续计划

后续更适合截图展示、也更能提升长期使用体验的功能包括：系统托盘图标、Windows 通知、最近触发记录、每个游戏单独限速值，以及对迅雷等没有稳定公开 API 的客户端探索 Windows 级进程限速方案。

## 卸载

运行：

```powershell
.\uninstall.cmd
```

它会停止后台监控，删除桌面/开始菜单入口，并移除开机启动项。配置文件默认保留，你可以手动删除整个解压目录来彻底清理。高级用户也可以直接运行 `uninstall.ps1`。

## 开发者

```powershell
.\build.ps1
```

## 许可证

MIT License
