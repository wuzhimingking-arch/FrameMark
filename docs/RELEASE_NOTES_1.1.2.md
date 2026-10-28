# 帧印 FrameMark 1.1.2 — 安装与完成弹窗优化（预发布）

- 处理完成弹窗增加右上角 ×；Esc 和原“再次处理”均可关闭，不删除视频或历史、不启动任务。
- 默认 README 改为简体中文，保留 README.en.md。
- NSIS 固定为简体中文；机器级安装使用管理员权限，新装默认系统 64 位 Program Files 下的 framemark。
- 已有机器级安装沿用原目录；用户可以手动选择安装路径，安装目录不附版本号。
- 检测到旧版当前用户安装时明确拦截，先卸载旧程序且不要选择删除用户数据，再安装新版。
  目前不自动执行跨权限卸载或搬移数据库；不是已经通过验收的自动迁移。
- 数据库、设置和日志仍使用原有用户应用数据路径，未改动数据层或水印算法。
- 沿用 1.1.1 自建 FFmpeg，完整对应源码和构建材料包含于本 Release 的
  `FrameMark-1.1.1-ffmpeg-corresponding-source.tar.gz`，保留原组件版本命名，内容未修改。
  亦可从 [v1.1.1 Release](https://github.com/wuzhimingking-arch/FrameMark/releases/tag/v1.1.1) 获取同一份材料。

## 验收边界

本机后台会话无管理员权限，没有绕过 UAC 或占用用户前台。
已在隔离 Windows 上验证**同一 SHA-256 安装包**：默认 Program Files/framemark、机器级重装、
直接 EXE/根快捷方式/开始菜单/桌面四个入口在干净 PATH 下启动、删除桌面快捷方式不删除程序、
卸载清理与用户数据保留；旧 1.1.1 用户级安装明确拒绝重复安装，卸载旧版后可装新版。
证据：[隔离安装测试](https://github.com/wuzhimingking-arch/FrameMark/actions/runs/40745690592)。
中文向导尚未逐页人工验收，旧用户级安装不自动跨权限迁移，因此仍标记预发布。
官网 CMS 认证返回 401，官网暂时保留 1.1.1 稳定下载；没有声称两个渠道已同步。

已通过：前端构建、无头完成弹窗 ×/Esc/原按钮回归、12 项原生 Rust 测试，以及正式构建产物的
GUI 子系统/系统 DLL/简体中文/管理员权限/WebView2 只读审计。UI 测试使用 IPC fixture，不冒充桌面验收。

安装包：`FrameMark_1.1.2_x64-setup.exe`，32,190,201 字节（本地原始“帧印”文件名的同字节发布副本）。
SHA-256：`9c268c9e68f92cd7d1e3e03775112bdae47995bee238ac298dfc183be56d6762`。
FFmpeg 源码包 SHA-256：`3ba4f02dd8674a6b1b3b13e74293010d9cac31aaea78e02849660826fb00d8bb`。
