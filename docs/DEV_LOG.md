# DEV_LOG

## 2026-08-12 · 0.1.0

- 需求：初始化随机单帧视频版权水印桌面项目。
- 修改：建立 Tauri 2 + TypeScript + Rust 工程，配置 SQLite、FFmpeg sidecar 资源位置和 NSIS 打包目标。
- 决策：V1 使用浏览器原生界面能力，不引入 React；核心处理保持跨平台路径和进程 API。
- 测试：`npm run build` 通过；Rust `windows-gnullvm` x64 target 下 `cargo check` 通过。
- 后续：实现随机算法、视频处理、数据持久化、界面和真实视频验证。

## 2026-08-12 · 0.9.0

- 需求：完成 V1 桌面端全部核心功能。
- 修改：实现平台身份与设置持久化、真实帧 PTS 扫描、确定性随机布局、FFmpeg 单帧滤镜、进度、取消、历史详情、输出保护和三页桌面界面。
- 决策：一个 `drawtext` 滤镜用目标帧序号表达式启用，避免逐帧导图；音频兼容时 stream copy，视频使用 libx264 CRF 17。
- 修复：FFmpeg 9 使用 `-/filter_complex` 文件参数；独立临时目录避免同 Seed 或崩溃遗留冲突。
- 测试：前端生产构建通过；Rust 6 项测试通过；真实中文路径视频严格单帧、音频、时长、分辨率和取消清理通过；本地界面完成视觉检查。
- 后续：完成 Windows NSIS 打包、安装包启动和最终 1.0.0 验收。

## 2026-08-12 · 1.0.0

- 需求：完成 Windows x64 V1 发行验收。
- 修改：内置 FFmpeg 9.0.1 Essentials 静态 sidecar 与 Noto Sans SC 字体，生成 current-user NSIS 安装包。
- 测试：release 程序成功启动，主窗口响应正常；内置 ffmpeg/ffprobe、字体和 GPL/OFL 文本位于发行资源中；NSIS 静默安装、已安装程序启动及静默卸载均通过。
- 视频矩阵：合成并处理 1080p30、4K60、1080×1920 竖屏、30 分钟、VFR 和 90° Rotation Metadata 样本；输出几何、时长、VFR 时间戳和方向均通过。
- 已知限制：未在本轮具备的独立 Windows 10 与 macOS 机器上验证；30 分钟和 4K 用合成样本验证处理正确性，不代表复杂内容的最终性能基准。
