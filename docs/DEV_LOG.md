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
- 构建说明核对：开发服务器显式绑定 1420 且启用 strictPort，与 Tauri devUrl 对齐，避免源码开发命令等待错误端口。
- 限制：桌面实拍截图未完成；第三方 GPLv3 二进制的完整对应源码/依赖构建材料尚未备齐，公共 Release 只提供源码预发布，不上传安装包，不宣称全部组件 MIT。

## 2026-10-28 · 1.1.0 合规资料修订 1

- 需求：核对既有 Windows 安装包的 FFmpeg GPLv3 分发材料，仅在完整义务确认后恢复官网上架。
- 版本：仅资料与审计工具变更，资料版本为 `1.1.0-compliance.1`；不重编应用、不修改产品 1.1.0 版本或原安装包。
- 溯源：下载 Gyan 7.1.1 Essentials 原包并被动解包安装器，ffmpeg/ffprobe SHA-256 与原包及项目 sidecar 一致。
  实测许可证 GPL-3.0-or-later，包含 GPL 库，未启用 nonfree；FrameMark 使用独立进程，不链接 libav。
- 源码：取得并验证 FFmpeg 精确提交以及 x264/x265/vid.stab/Rubber Band 上游快照归档，保留原始许可、
  原包 README、构建参数和 URL/SHA-256 清单。未伪造上游补丁或将这些快照称作完整对应源码。
- 文档：新增合规审查、未发送的上游索取材料草稿；更新双语 README、第三方声明（含 IJG 致谢）、项目上下文。
  应用自有源码维持 MIT，FFmpeg、字体、MPL 等依赖各自保留许可。
  原始第三方 README/许可证保留字节，使用精确路径 Git attributes 避免自动换行改写原文。
- 审计：新增 `npm run audit:ffmpeg`，检查归档/EXE 哈希、原包一致性、唯一 Tauri NSIS 标记差异、字体/许可和构建参数；
  包含标记处理正反例自检。完整性通过仍返回 2，明确禁止误当成公开分发放行。
- 测试：前端构建通过；cargo-xwin 编译后误用缺失的 Wine 启动测试，改为原生运行生成的 Windows 测试 EXE，12 项全部通过。
  另用安装包解出的 FFmpeg 处理中文/空格路径样本：2 秒、640×360、30 FPS、60 帧、H.264/AAC；
  遍历解码帧仅第 15 帧含水印，前后帧无水印，音频压缩流 SHA-256 前后一致。五份源码归档均可列出文件及许可证。
  Rust 格式检查、暂存区差异检查、包含新增资料的公开文件/历史敏感模式扫描均通过；扫描不替代人工审查。
- 剩余：原包未标明 9 个直接依赖版本，完整传递依赖/补丁/版权声明及必要构建脚本未获确认；已准备官方邮箱请求草稿，未代用户发送。
  不用最新版依赖或推测配方替代实际材料；LGPL 替代会影响现有 libx264 CPU 路径，仅说明方案、不擅自更换。
- 官网：只读核实公开产品 API 仍为 404，不上传二进制、不开放下载、不更改草稿；没有公开下载文件可供完整性验证。
  未占用前台，不重复安装/卸载或 GUI 验收。当前仍不能宣告 1.1.0 具备公开分发条件。
## 2026-10-28 · 1.1.1

- 以固定 FFmpeg 7.1.1 及 x264、FreeType、HarfBuzz、zlib、nv-codec-headers、AMF、oneVPL 源码自行交叉编译，替换未知依赖材料的旧 Gyan 包。完整输入、脚本、配置、许可证与哈希保留并随 Release 发布。
- 维持 CPU、NVENC/QSV/AMF、HEVC 解码、AAC、多水印与 PNG 预览；保留旧 1.1.0 安装包，不补发旧二进制。
- 修复真实 Windows IPC 后备路径返回数字数组导致预览 Blob 无效的问题，CSP 明确允许本机 IPC；新增 ArrayBuffer/数字数组两条实际图片解码回归。
- 构建、前端测试、格式检查和 12 项原生 Rust 测试通过；CPU/NVIDIA 双模式、VFR、旋转、音频、取消、中文路径与 HEVC 转 H.264 通过。QSV/AMF 已编译但没有对应设备实测。
- 最终安装包 32,193,101 字节，SHA-256 344a499d888ec02ae4868a679003d7999bec32df95c657ab98b3e38b0390a144。安装、独立目录、干净 PATH 启动、图标、系统 DLL、无伴生控制台与卸载检查通过；真实桌面预览和 NVIDIA 导出通过，保存真实截图。
- 安装包未签名，不承诺 Windows/SmartScreen 无提示。

## 2026-10-29 · 1.1.2（源码预发布，管理员验收待完成）

- 完成弹窗增加可访问的右上角 ×，保留原生 Esc 和三个按钮；关闭只操作 dialog，不删除输出/历史或新建任务。
- 默认 README 改为中文，保留 README.en.md；软件不添加国际化或新的水印功能。
- NSIS 固定 SimpChinese、perMachine。使用 Modern UI 自定义 GUI 初始化回调设置默认 Program Files64/framemark，
  静默安装也在 preinstall 校正；机器级旧路径和显式 /D 保留。首次构建发现 .onGUIInit 与 MUI 冲突，改用正式 MUI_CUSTOMFUNCTION_GUIINIT 扩展点。
- 旧 per-user 安装明确拦截，提示卸载旧程序且不要删除数据，不冒充自动跨权限迁移已实现；原 app_data_dir/app_log_dir 和数据库保持不变。
- 更新发行验证器的全局开始菜单路径，增加 -AuditOnly；非管理员实际安装验收会明确拒绝，不弹 UAC。
- 已通过前端 build、无头 ×/Esc/播放及定位 IPC/再次处理回归、1.1.2 新编译 Windows Rust 测试 EXE 的 12 项测试及格式检查。
- 自建 FFmpeg 继续使用 1.1.1 的两份精确哈希和已公开完整对应源码包；未替换视频管线。
- 尚无管理员权限，未执行真实机器级安装/升级/中文页面/启动/卸载验收；不修改官网 1.1.1 稳定下载，不公开未验收安装包。
  源码同步既有 GitHub 仓库并标记源码预发布；不得把编译和只读审计等同于完整安装验收。
- 后续补充隔离 Windows CI 安装/启动/卸载与默认 Program Files 目录检查，成功后留存被验收的同一安装包。
  第一轮实际安装后快捷方式字符串比较失败；本地复现 WScript 把 8.3 短路径展开为长路径的行为，
  修正两侧规范化比较，正例同文件通过、反例不同文件拒绝，未创建用于比较的链接文件。
- 官网现有本地 CMS 登录配置实际返回 HTTP 401，未重置生产密码、绕过认证或修改下载链接；现有 1.1.1 稳定版保留。
- 默认目录隔离验收发现真实问题：Tauri 在 preinstall hook 前已 SetOutPath，静默修改 INSTDIR 却未同步 OUTDIR，
  导致文件与快捷方式指向分离。修复为同步输出目录，旧空目录仅非递归删除；交互模式不在安装阶段重写用户选择的路径。
- 独立候选诊断进一步证实：WScript 对含环境路径信息的实际 NSIS 链接返回空目标，但 Windows Shell 正常解析并可启动，
  验证器改用 Shell.Application.GetLink 校验实际目标和图标，未削弱目标路径检查。
- 最终同一候选文件通过隔离测试 40745690592：默认目录、重装、旧用户级安装拒绝重复、保留数据卸载旧版、
  EXE/根快捷方式/开始菜单/桌面四入口干净 PATH 启动、新版卸载与数据保留均通过。
  候选工作流仅手动触发并锁定 SHA-256；GitHub 读取私有草稿要求写者访问，使用短期仓库令牌，不持久化账户凭据。
- 最终安装包 32,190,201 字节，SHA-256 9c268c9e68f92cd7d1e3e03775112bdae47995bee238ac298dfc183be56d6762。
  对应源码包重新检查包含 8 份源码归档、构建脚本、配置和校验清单，哈希与 GitHub 公开资产一致。
- GitHub 发布为预发布；官网因 CMS 401 未上传新版、未改版本或下载链接。中文向导未逐页人工验收，
  不声称自动跨权限迁移已实现。后续恢复官网有效授权后方可同步和核验匿名下载。
