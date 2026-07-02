# qBittorrent 游戏限速助手

适合一边挂 BT / PT / 磁力下载、一边玩联机游戏的 Windows 用户。

## v0.3.0 重点

- 新增多下载器配置：qBittorrent / qBEE、Transmission、aria2 / Motrix、BitComet / 比特彗星。
- qB 和 Transmission 会自动切换备用限速。
- aria2 / Motrix 会在游戏中临时切换全局上下行限速，退出后恢复原值。
- BitComet 已加入列表，但由于缺少稳定公开远程限速 API，本版会明确提示暂不自动控制。
- 配置界面增加轻量动画、下载器选择、aria2 限速值和“创建桌面入口”。

## 下载

普通用户请下载：

```text
qbee-game-speed-limiter-windows.zip
```

解压后双击 `qbee_limiter_config.exe`，按中文说明选择下载客户端并自动扫描游戏库。
