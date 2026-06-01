# qbee 游戏限速助手

[English](README.md) | 简体中文

这是一个 Windows 小工具。打开游戏时，它会自动打开 qBittorrent / qBittorrent Enhanced Edition 的“备用速度限制”；游戏关闭后，它会自动恢复。

最新版下载：[v0.2.3](https://github.com/ZyPulse-zy/qbee-game-speed-limiter/releases/tag/v0.2.3)

## 适合谁

如果你一边下载 BT，一边玩网游、联机游戏或对网络延迟敏感的游戏，这个工具可以帮你在玩游戏时自动降低 qB 下载/上传速度，退出游戏后再恢复正常。

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

## 开发者

构建方式、设计稿、更新日志等开发资料放在仓库中，不再放进发行包。

```powershell
.\build.ps1
```

## 许可证

MIT License
