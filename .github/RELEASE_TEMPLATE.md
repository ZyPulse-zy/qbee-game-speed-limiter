# 下载器游戏限速助手

适合一边挂 BT / PT / 磁力下载、一边玩联机游戏的 Windows 用户。

## v0.3.5 重点

- 项目正式从 `qbee Game Speed Limiter` 改名为“下载器游戏限速助手 / Download Client Game Speed Limiter”。
- 发行包文件名改为 `download-client-game-speed-limiter-windows.zip`。
- 配置器和后台程序改名为 `download_limiter_config.exe` / `download_limiter_monitor.exe`。
- 配置文件改名为 `download_client_game_speed_limiter.json`。
- 首次升级会自动读取旧版 `qbee_game_speed_limiter.json`，安装脚本也会把旧配置迁移到新文件名。
- 开机启动项、桌面入口、开始菜单入口同步改名，并会清理旧版 qbee 入口。
- 继续支持自动限速：qBittorrent / qBEE、Transmission、aria2 / Motrix、µTorrent / BitTorrent Classic、Deluge、BitComet / 比特彗星。

## 下载

普通用户请下载：

```text
download-client-game-speed-limiter-windows.zip
```

解压后建议先双击 `install.cmd` 创建快捷入口并打开配置界面，也可以直接双击 `download_limiter_config.exe` 便携使用。按中文说明选择下载客户端并自动扫描游戏库。
