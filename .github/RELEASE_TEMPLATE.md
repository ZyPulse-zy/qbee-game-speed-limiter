# qbee 游戏限速助手

适合一边挂 BT / PT / 磁力下载、一边玩联机游戏的 Windows 用户。

## v0.3.3 重点

- 新增可双击的 `install.cmd` / `uninstall.cmd`，新手不用再右键运行 PowerShell 脚本。
- 应用图标已嵌入 Windows exe，桌面入口和任务栏显示会更完整。
- 快捷入口名称统一为 `qbee 游戏限速助手`，卸载脚本仍会清理旧版入口。
- 已支持自动限速：qBittorrent / qBEE、Transmission、aria2 / Motrix、µTorrent / BitTorrent Classic、Deluge。
- qB 和 Transmission 会自动切换备用限速；aria2、µTorrent / BitTorrent、Deluge 会临时切换全局上下行限速并在游戏退出后恢复。
- BitComet / 比特彗星已加入主流客户端列表，但由于缺少稳定公开远程限速 API，本版会明确提示暂不自动控制。
- 配置界面增加轻量动画、下载器选择、统一游戏中限速值、“运行自检”和“创建桌面入口”。
- 发行包包含 `install.ps1` / `uninstall.ps1`，可创建桌面和开始菜单入口，不需要管理员权限。

## 下载

普通用户请下载：

```text
qbee-game-speed-limiter-windows.zip
```

解压后建议先双击 `install.cmd` 创建快捷入口并打开配置界面，也可以直接双击 `qbee_limiter_config.exe` 便携使用。按中文说明选择下载客户端并自动扫描游戏库。
