# FrameMark 第三方分发记录

## 1.1.1：替换旧二进制并自建

2026-10-28 改用官方 FFmpeg 7.1.1 固定提交与全部已确认依赖源码交叉编译，显式 GPL/version3、静态链接、禁用自动发现与网络。精确输入见 `scripts/ffmpeg-sources.json`，真实构建脚本为 `scripts/build-ffmpeg.sh`。保留 x264 CPU、NVENC/QSV/AMF、内置 HEVC 解码、AAC、FreeType/HarfBuzz 文字渲染与 zlib/PNG。无源码补丁，不使用旧 Gyan 二进制或其未知依赖。

新 ffmpeg SHA-256：`f3bc8375340268beb41f92d887175928dde5720e4cf4de43eaf4d46c73b200f0`。
新 ffprobe SHA-256：`cd7ac5393a7323f1325323003d5f05b8a4acea74df6f3a2d39304f141e2c18a6`。

完整对应源码包包含 8 份原始归档、全部构建脚本/配置/工具链版本、许可证、匹配二进制及 SHA256SUMS；完整解包后逐文件校验通过。
源码包 `FrameMark-1.1.1-ffmpeg-corresponding-source.tar.gz`：273,510,184 字节，SHA-256 `11a1fd3519d2fa2ce3413f4611f817cbf5cdd6857843557cf43fc0932f6533a9`。
计划随 [1.1.1 Release](https://github.com/wuzhimingking-arch/FrameMark/releases/tag/v1.1.1) 公开，不以该链接预先存在作为已发布证明；正式发布状态以 Release 和官网验收记录为准。

新 sidecar 已在 Windows 通过 12 项原生 Rust 测试，包含 CPU/NVENC 双模式单帧、多水印、音频、VFR、旋转、取消与原片保护；额外中文路径 HEVC 输入转 H.264：480×270、48 帧、2.000000 秒。动态导入仅含 Windows 系统 DLL。QSV/AMF 已编译，当前没有对应硬件，不冒充真实设备编码验收。全部实际外部组件的许可与原始源码均保留；旧包中 x265、vid.stab、Rubber Band 等不再分发。字体 OFL 与应用依赖（含 MPL）继续保留原始声明和精确源码获取方式。

这项替换解决了旧构建对应源码和构建材料无法对应的问题，不构成对所有司法辖区专利权或所有未来使用方式的法律保证。

## 以下是 1.1.0 历史审查（仅适用于不再公开分发的旧安装包）

审查日期：2026-10-28。资料修订：`1.1.0-compliance.1`。
**结论：BLOCKED，尚不满足本项目公开安装包的放行条件。** 本文是技术证据和待办清单，不是法律意见。
本轮仅补充合规资料与审计工具，应用仍为 1.1.0，不重编或替换现有安装包、不改变软件功能。

## 已确认的对象和来源

- 安装包：`artifacts/windows/帧印_1.1.0_x64-setup.exe`，58,605,001 字节。
  SHA-256：`21ca2c87d52c661500a3f38c0cb41f6a0180611d290642ad0a55ee2ab1fe5bb8`。
- 对安装包做只读解包，不运行安装器；其中 FFmpeg/ffprobe 与
  [Gyan 7.1.1 Essentials 原包](https://github.com/GyanD/codexffmpeg/releases/tag/7.1.1)
  逐文件 SHA-256 一致，并与开发侧固定的 sidecar 一致。
- 执行解包后的 `ffmpeg -version`、`-buildconf`、`-L`：版本为
  `7.1.1-essentials_build-www.gyan.dev`，GCC 14.2.0 / MSYS2；许可证为
  **GPL-3.0-or-later**。启用 `--enable-gpl --enable-version3 --enable-static`，
  包含 libx264/libx265/libxvid/librubberband/libvidstab、GPL 内部滤镜和 libpostproc；没有 `--enable-nonfree`。
  “static”指 FFmpeg 内部的库链接方式，不表示它静态链接进 FrameMark。
- 完整原始功能/依赖版本清单保留在 [上游 README](compliance/ffmpeg-7.1.1/UPSTREAM-README.txt)，
  FFmpeg 原始许可证分类见 [上游 LICENSE.md](compliance/ffmpeg-7.1.1/LICENSE.md)，同目录保留 GPL/LGPL 原文。
  [实测 configure 参数](compliance/ffmpeg-7.1.1/OBSERVED-BUILDCONF.txt) 由审计命令与随包 EXE 交叉校验。
  所有下载 URL、源码版本和 SHA-256 见 [机器可读清单](compliance/ffmpeg-7.1.1/manifest.json)。
  校验值是本次对下载文件计算的指纹，不是上游签名或完整构建来源证明。

## 应用与公开源码的关系

`src-tauri/src/media.rs` 通过 `std::process::Command` 启动独立 EXE，使用文件和标准管道传递媒体、滤镜、进度。
Cargo 清单/锁文件没有 ffmpeg-sys 或 libav 链接依赖。因此目前按独立程序聚合分发处理，
不因同一个安装器中包含 FFmpeg 就自动把 FrameMark 自有源码改为 GPL；MIT 本身也不与 GPLv3 冲突。
如未来改成库链接或更紧密的专用协议，需要重新判断，不能照搬本结论。

GitHub `v1.1.0` 公开源码标签与原始本地 HEAD 为
`e104405e8aeba5106ddc0717ff572cc8da37a48e`，Release 是源码预发布、没有二进制附件。
安装包的主 EXE 与留存的 Release EXE 只有 Tauri 的
`__TAURI_BUNDLE_TYPE_VAR_NSS` / `...UNK` 三字节标记差异；限定规范化该唯一标记后全部字节一致。
这是源码/构建留存的一致性证据，**不是独立环境可复现构建的证明**；标签本身也不是二进制来源证明。

## 已取得的源码与尚缺材料

已保存五份未修改上游源码归档：FFmpeg、x264、x265、vid.stab、Rubber Band。
FFmpeg 核心提交由 Gyan 发布说明及 n7.1.1 标签共同指向；四个库按原包版本/提交标记定位，
x264 的 `r3204 373697b` 还从随包 EXE 字符串交叉核对。x264/x265 使用清单标明的 GitHub 镜像。
归档包含各自上游许可证和构建文件，但**未确认 Gyan 是否应用过额外补丁或构建修改**。

原包列出 34 个依赖的版本标记，却未给出下列已启用依赖的确切版本：
fontconfig、iconv、GnuTLS、libxml2、GMP、bzip2、XZ/liblzma、zlib、OpenJPEG。
其余标记也不能证明所有传递依赖、补丁、版权声明和实际构建输入都已齐备。
Gyan 对应 Release 只附二进制归档；本次检查其公开仓库没有找到 7.1.1 完整依赖源码包和所需构建脚本。
这描述的是本次可核实的公开材料，不断言上游不可能另行提供材料。

当前 `-buildconf`、原包 README、MSYS2/GCC 版本，只是构建信息，**不能冒充控制实际构建的脚本**。
也不能用最新依赖源码、猜测的配方或其他人的构建记录填补缺口。
本轮未发送上游邮件或创建 Issue；已准备 [索取材料草稿](compliance/ffmpeg-7.1.1/UPSTREAM_SOURCE_REQUEST.md)。
上游维护者在 [源码索取讨论](https://github.com/GyanD/codexffmpeg/issues/249#issuecomment-6264377917)
要求从组织官方邮箱提供应用和组织链接，该回复不是本次材料已经交付的证明。

## 许可证义务与放行条件

依据 [GPLv3 第 1、5、6 条](https://www.gnu.org/licenses/gpl-3.0.en.html) 和
[FFmpeg 官方法律说明](https://ffmpeg.org/legal.html)：

1. 保留 GPL、版权、适用的第三方声明，不得把 FFmpeg 重新标为 MIT，也不得限制接收者的 GPL 权利。
2. 所分发 FFmpeg 的完整对应源码应覆盖所需组件、实际修改，以及控制生成和安装所需的脚本。
   第 1 条对 System Libraries、通用工具等有例外；不是要求收集整个 Windows 或所有编译器源码，
   也不是要求字节级可复现。各依赖是否属于例外需按事实判断，不能一概省略。
3. 计划采用第 6(d) 条网络分发：在二进制下载位置明确给出完整对应源码的获取方式，无额外收费。
   该条允许满足条件的不同服务器托管，并非强制同站；分发者仍负责可用性。
   不要用只有源码主页的链接或未经落实的书面承诺替代实际材料。
4. 完成所有实际包含组件的许可证/版权审查，包括 FFmpeg LICENSE.md 要求的 IJG 致谢，
   以及非 GPL 依赖各自的声明。现有 `DEPENDENCIES.txt` 覆盖应用 Rust/npm 依赖，
   **不等于覆盖 FFmpeg 静态依赖**。当前还无法排除未收齐组件的附加义务。
5. 为最终源码包固定清单和校验值，检查完整性/解压/所需构建材料，再审查网站旁的下载说明。
   全部满足后才能上传安装包并解除官网下载禁用；本轮没有放行。

字体仍按 SIL OFL 1.1；安装包内原始字体/OFL、GPLv3 文本、应用依赖声明均已检查存在。
WebView2 是单独的微软组件，不属于本项目 MIT 授权。
应用依赖声明还包含 MPL-2.0 的 cssparser、cssparser-macros、dtoa-short、option-ext、selectors，
随包声明列有各自版本和精确 crates.io 源码下载地址；这类文件级许可不要求把自有应用整体改成 MPL。
其他声明包含 MIT/Apache-2.0/BSD/ISC/Unicode/Zlib 等，均不得被顶层 MIT 声明覆盖。
审查未发现“自有代码必须整体 GPL”这一直接冲突，但剩余第三方依赖材料未齐，不能宣告整体分发已无冲突。
如对聚合边界、例外或完整对应源码范围有争议，应由许可证专业人员确认，不能用一份法律免责声明替代材料。

## 可复核的后台验证

本地资料位于 `artifacts/compliance/FrameMark-1.1.0/`，大归档不提交 Git。
原始完整归档名为 `upstream-7.1.1.7z`；不要误用初次中断下载留下的同版本不完整文件。
被动解包后的安装器目录为 `installer-image/`，上游原包目录为 `upstream/ffmpeg-7.1.1-essentials_build/`。
审计不安装辅助 MSI，不修改系统目录、PATH 或当前应用，也不启动 GUI。

```powershell
npm run audit:ffmpeg
```

该命令检查清单哈希、安装器与上游 EXE 一致、Tauri 标记限定差异、字体和随包许可证一致，
并实际运行安装器中的 ffmpeg/ffprobe，把版本、完整构建参数、许可证输出写入本地 `verification.json`。
包含标记规范化的正反例自检。退出码 **2** 表示完整性通过但分发仍被阻塞；哈希/文件异常则失败。
这不是自动法律认证，也不应作为返回成功就自动发布的 CI 开关。

本轮验证结果：

- `npm run build` 通过；`cargo xwin test --lib` 完成 MSVC 测试编译，但 cargo-xwin 随后试图用
  不存在的 `wine` 启动 Windows 测试程序。直接在 Windows 后台运行其刚生成的测试 EXE 后，
  **12 项通过，0 失败**，包括真实 CPU/可用硬件路径、多水印、PTS、音频、方向、取消、中文路径和边界。
  这些测试使用与安装包哈希一致的 sidecar，不冒充 GUI 安装验收。
- 另外直接运行从安装包解出的 ffmpeg/ffprobe，在中文/空格路径创建并处理 2 秒样本：
  H.264、640×360、30 FPS、60 帧、AAC，输出时长 2.000000 秒。
  第 15 帧显示中文/英文/© 文字，第 14/16 帧没有标记；遍历全部解码帧的亮度阈值检测只检出第 15 帧。
  音频 stream copy 前后压缩流 SHA-256 均为
  `ca7daabde1266a1dc7985597761f0650a19e617bf942ca84a72854ed075a902d`。
- 五个源码归档均可完整列出文件且包含上游许可文件；完整性审计按设计返回 2（BLOCKED）。
- 为不打扰正在使用电脑的用户，没有重新安装/卸载、弹出 GUI 或操作官网管理后台；
  不将过去的 GUI 验收或本轮被动解包当作本轮重新完成的干净系统验收。

## LGPL 替代方案（没有实施）

可另行评估禁用 GPL/nonfree、保留 drawtext/字体及 NVENC/QSV/AMF 的精简 LGPL 构建，
但当前 CPU 必备的 libx264 属于 GPL，不能原样保留在纯 LGPL 方案中。
替代 CPU H.264 编码器需要重新验证画质、平台可用性、许可证和专利分发条件，并重做完整回归。
因此这不是无损替换；未获授权前不更换二进制、不移除功能。LGPL 也并不免除其源码等分发义务。

## 官网状态

保留真实仓库 [wuzhimingking-arch/FrameMark](https://github.com/wuzhimingking-arch/FrameMark)。
本次只读验证 `https://peakedge.tech/api/products/framemark` 返回 404，符合产品仍未公开的状态。
未上传安装包、未开放下载、未改动线上产品草稿，未宣称已有公开下载文件可供哈希验证。
