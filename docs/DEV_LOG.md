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

## 2026-08-13 · 1.0.1

- 需求：修复正式 Windows 程序启动时报 `WebView2Loader.dll` 缺失，并检查其他发行依赖。
- 根因：项目把 Windows 默认构建目标强制为 `gnullvm`；该 ABI 的 `webview2-com-sys` 动态导入 `WebView2Loader.dll`，同时程序依赖 `libunwind.dll`。构建目录虽有 Loader，但复制裸 `frame-mark.exe` 时遗漏依赖，且裸 EXE 被误当成发行物。
- 修改：移除全局 gnullvm 目标，Windows 发行统一改为 `x86_64-pc-windows-msvc`；新增一键构建脚本和发行验证脚本，`artifacts/windows` 只输出正式 NSIS 安装包，并与 Vite 的 `dist` 前端目录隔离。
- WebView2：NSIS 改为内嵌微软 Evergreen Bootstrapper，安装时检测 Runtime，缺失时由 Bootstrapper 安装；不向系统目录手工复制 DLL。
- 依赖：安装包继续包含 FFmpeg、ffprobe、Noto Sans SC 和许可证；PE 审计同时覆盖三个 EXE，拒绝 WebView2Loader、libunwind、GNU/VC++ 外置运行库和其他非系统 DLL。
- 测试：TypeScript/Vite 构建通过；MSVC Windows 测试程序 7 项通过；1.0.1 安装包在临时副本、独立安装目录和最小 PATH 下启动主界面成功，开始菜单、资源、卸载均通过。

## 2026-08-14 · 1.0.2

- 需求：消除正式启动及 FFmpeg 调用的控制台窗口，并加入自动 GPU H.264 编码加速。
- 根因：Release 二进制仍使用 Windows Console subsystem；FFmpeg/ffprobe 没有统一设置 Windows 无控制台创建标志。原 FFmpeg 9.0.1 虽列出 NVENC，但要求 NVENC API 13.1 / NVIDIA 驱动 610+，与本机 RTX 5070 Ti 的 596.49 / API 13.0 不兼容。
- 修改：Release 入口使用 Windows GUI subsystem；所有 FFmpeg/ffprobe 创建均使用 `CREATE_NO_WINDOW`，保留 stdout、stderr、进度和退出码。设置页新增自动、NVENC、QSV、AMF、CPU；启动时对随包 FFmpeg 做最小 H.264 实测，按 NVENC、QSV、AMF、CPU 选择，硬件初始化失败清理部分输出后自动回退 CPU。
- 分发：固定 FFmpeg 7.1.1 Essentials 静态构建，实测兼容当前 NVIDIA 驱动，且含 NVENC/QSV/AMF；许可证说明同步更新。
- 测试：MSVC Rust 9 项通过，覆盖 CPU 与 NVENC 严格单帧、音频、VFR 时间戳、旋转、取消和不可用硬件回退。CPU/NVENC 实测：1080P30/6 秒为 2.83/0.87 秒，4K60/3 秒为 9.20/4.25 秒；NVENC 采样 Video Encode 峰值分别为 48%/50%。
- 发行：1.0.2 NSIS 安装包在独立临时目录、干净 PATH 下启动成功；PE 审计确认 GUI subsystem、无 WebView2Loader 或其他遗漏动态库。

## 2026-10-27 · 1.1.0

- 需求：自由水印信息、多选与同时出现/依次闪现、安装根目录启动入口、GitHub 公开源码。
- 修改：取消平台必填 UI，支持 240 字符/8 行、编辑、复制、删除、上下排序和本次独立顺序；设置保存选中 ID 顺序。
- 迁移：user_version=1 事务增加 identities.sort_order 与 histories.plan_json，不删除旧表/文字/历史；旧历史按算法 V1 读取，新历史保存 V2 计划及文字快照。
- 计划：保留原区间选帧，新增按模式展开的多水印事件；实际 FFmpeg 字体光栅测量、换行、有限缩小、碰撞检测与容量报错；预览直接复用计划和 FFmpeg 渲染。
- 修复：同文字复制水印的同帧绘制按选择槽隔离，避免表达式合并导致坐标相加；文本 expansion=none 保留百分号等原文字面含义。
- 发布：显式配置应用/安装包图标，NSIS 根目录“帧印.lnk”指向安装的主 EXE；保留 GUI subsystem、GPU 实测与 CPU 回退。FFmpeg 版本增加哈希核验，安装资源收集第三方依赖许可证。
- 测试：Rust 覆盖迁移/CRUD/排序/重启/历史、1000 事件/3000 个矩形、同文独立绘制、两模式 CPU/NVENC、真实单帧、VFR 时间戳、音频流哈希、预览、旋转、中文路径与取消；无头前端验证多选、顺序、模式统计、多行 CRUD、复制、预览切换、空选拦截和 150% 像素密度。
- 安装：独立目录根快捷方式/图标及干净 PATH GUI 启动通过；用户中止桌面操作后改为全后台，最终检查可通过 FRAMEMARK_VERIFY_NO_GUI=1 跳过 GUI 启动。
- 开源：补充英文/中文 README、MIT、贡献说明、Issue 模板、Windows CI 与全 Git 历史敏感内容扫描。
- 限制：桌面实拍截图未完成；第三方 GPLv3 二进制的完整对应源码/依赖构建材料尚未备齐，公共 Release 只提供源码预发布，不上传安装包，不宣称全部组件 MIT。
