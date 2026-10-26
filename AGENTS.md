# 帧印开发规范

## 技术栈与目标

- 桌面框架：Tauri 2；界面：原生 TypeScript、HTML、CSS；核心：Rust。
- 视频：随应用分发的 FFmpeg/ffprobe；本地流式处理，不上传文件。
- 数据：应用数据目录中的 SQLite；不得写入安装目录。
- Windows 10/11 x64 是 V1 首要目标；文件、进程和路径逻辑必须保持 macOS 可移植性。

## 核心约束

- 每个连续时间区间按真实视频帧 PTS 选且只选一帧；最后不足完整间隔的区间也必须处理。
- 多水印计划区分事件数与绘制数：同时出现的全部水印在同一个目标帧，依次闪现按选择顺序循环；旧库和历史文字快照不得清空。
- 预览复用原生计划和 FFmpeg 渲染，文字实际尺寸参与安全区域与碰撞检查；相同内容的复制水印也要独立绘制。
- 每个水印的完整矩形必须在安全区域内；相同 Seed 和输入参数必须生成相同布局。
- 永不修改或覆盖原视频；取消或失败时删除未完成输出。
- 音频兼容时流复制；视频默认高质量 H.264 重编码；HDR 必须在处理前提示。
- 不增加剪辑、联网、AI、云账户或未来阶段功能。

## 工程与验证

- Rust 代码用 `cargo fmt`；核心随机算法必须有自动化测试。
- 前端提交前运行 `npm run build`，后端运行 `cargo test`。
- 视频处理变更需用真实样本验证 ffprobe 信息、音频、方向、时长与严格单帧。
- Windows 正式发行固定使用 `x86_64-pc-windows-msvc` 与 NSIS 安装包；不得把 `target` 中的裸 EXE 当作交付物。
- Windows 安装包必须内嵌 WebView2 Bootstrapper，发布前运行 `npm run verify:windows` 审计动态库并从独立安装目录启动。
- 硬件编码器必须由随包 FFmpeg 的实际最小编码测试决定可用性；不可用或任务初始化失败时清理临时输出并回退 CPU。
- 中文文件通过 PowerShell 读取时显式使用 UTF-8；不要因终端乱码重写源码。

## 版本、文档与 Git

- 版本以 `src-tauri/tauri.conf.json` 为唯一产品版本源；发布同步 Cargo 与 npm 包版本。
- 项目事实变化时更新 `docs/PROJECT_CONTEXT.md`；每个独立任务追加 `docs/DEV_LOG.md`。
- 每阶段只暂存本任务文件并使用具体中文提交信息；不提交密钥、Token、keystore、`local.properties`。
- 未明确要求时不 push、不创建 GitHub 仓库、不生成 APK/AAB。
- 自有源码采用 MIT；含 GPLv3 FFmpeg 的公开安装包必须先备齐完整对应源码及构建材料，不得将第三方组件统称 MIT。
