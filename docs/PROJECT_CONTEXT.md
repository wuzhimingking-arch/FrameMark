# PROJECT_CONTEXT

## 产品目标

“帧印”是本地桌面端随机单帧视频版权水印工具。它将视频划分为连续时间区间，在每个区间的真实视频帧中随机选择一帧，并把平台身份文字放在安全区域内的随机坐标。

## 当前版本

- 版本：1.0.1（Windows x64 V1）
- 目标：Windows 10/11 x64，架构兼容后续 macOS。

## 核心架构

- Tauri 2 提供桌面窗口和 Rust 命令边界。
- 原生 TypeScript/HTML/CSS 实现三页界面，避免无必要 UI 框架。
- Rust 负责 ffprobe/FFmpeg、确定性随机布局、SQLite、任务生命周期和文件安全。
- FFmpeg、ffprobe 与 Noto Sans SC 字体作为安装资源分发。
- Windows 正式发行使用 MSVC 目标，WebView2 Loader 静态链接；NSIS 内嵌微软 Evergreen Bootstrapper，在缺少 WebView2 Runtime 时检测并安装。

## 已实现功能

- 视频点击选择与桌面拖拽；ffprobe 展示时长、分辨率、帧率、编解码、旋转和 HDR 信息。
- 平台身份增删改、启停与 SQLite 持久化；默认设置保存到应用数据目录。
- 按真实解码帧 PTS 划分区间并选择严格单帧，末尾不足完整间隔仍处理。
- 安全区域二维随机位置、相邻位置分散、确定性 Seed、随机透明度和随机字号。
- FFmpeg 流式 H.264 高质量处理；兼容音频优先 stream copy；输出不覆盖、空间预检、进度、取消清理和输出校验。
- 视频样式预览、安全区域辅助框、历史列表和每帧时间/PTS/帧序号/坐标/样式详情。
- 本地日志、缺失文件状态、播放与打开文件夹操作。

## 已知限制

- V1 输出固定为高质量 H.264；H.265 可作为输入但会转 H.264。
- HDR 会在处理前明确警告，当前不保证 HDR 色彩保持；不支持 Dolby Vision 无损保留。
- 不提供硬件编码、图片 Logo、AI 检测或联网功能。
- WebView2 Runtime 缺失时，内嵌 Bootstrapper 仍需联网获取微软 Evergreen Runtime；应用本身不依赖 Node、Rust、Visual Studio 或项目目录。
