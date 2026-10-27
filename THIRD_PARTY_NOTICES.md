# 第三方组件说明

## 1.1.1 自建 FFmpeg

FrameMark 1.1.1 使用官方 FFmpeg 7.1.1 源码自行构建的 Windows sidecar，通过独立进程调用，不与本程序链接。启用 GPL 与 version3，FFmpeg 可执行程序按 GPL-3.0-or-later 分发。完整对应源码、依赖源码、构建脚本、配置和 SHA-256 清单随同版本提供于 [1.1.1 Release](https://github.com/wuzhimingking-arch/FrameMark/releases/tag/v1.1.1)。固定输入见 `scripts/ffmpeg-sources.json`，构建说明见 `docs/build/FFMPEG_WINDOWS.md`。

实际外部依赖为 x264 r3204/373697b（GPL-2.0-or-later）、FreeType 2.13.3（FTL）、HarfBuzz 10.0.1（MIT 风格）、nv-codec-headers 12.2.72.0（MIT）、AMF 1.4.35 头文件（MIT）、oneVPL dispatcher 2.14.0（MIT）。原始版权和许可随安装包保留。GCC 运行库按其 GPL 与 GCC Runtime Library Exception 分发，MinGW-w64 运行库保留原始声明；不限制用户依据各组件许可证复制、修改及再分发。硬件驱动由系统提供，不随包分发。

Portions of this software are copyright © 1996–2024 The FreeType Project (www.freetype.org). All rights reserved.
This software is based in part on the work of the Independent JPEG Group.

PNG 预览使用 zlib 1.3.1（zlib 许可），保留原始 LICENSE。NVIDIA 编码/解码头文件的原始版权和许可随完整对应源码保留。

新构建不含旧 Gyan 包的 x265、vid.stab、Rubber Band 等外部库。安装包未进行 Authenticode 签名，Windows/SmartScreen 可能显示未知发布者；不声称已签名或不会出现安全提示。

## 1.1.0 历史 FFmpeg（不公开分发旧二进制）

发行构建使用 Gyan.dev 的 FFmpeg 7.1.1 Essentials 静态构建。FFmpeg 是独立 sidecar，不与本程序链接。
实测 `ffmpeg -L` 的适用许可证为 **GPL-3.0-or-later**；完整 GPLv3 文本随安装包位于
`resources/licenses/FFmpeg-GPLv3.txt`。Copyright (c) 2000–2025 the FFmpeg developers；
各源文件和依赖保留各自版权。构建配置及源码入口见下方审查资料。

- 构建发布页：[GyanD/codexffmpeg 7.1.1](https://github.com/GyanD/codexffmpeg/releases/tag/7.1.1)。
- FFmpeg 对应源码提交：[db69d06eeeab4f46da15030a80d539efb4503ca8](https://github.com/FFmpeg/FFmpeg/commit/db69d06eeeab4f46da15030a80d539efb4503ca8)。
- 二进制启用 `--enable-gpl --enable-version3 --enable-static`、libx264/libx265、
  NVENC、libvpl/QSV、AMF，没有 `--enable-nonfree`。
- 当前 ffmpeg.exe SHA-256：`B90225987BDD042CCA09A1EFB5E34E9848F2D1DBF5FBCD388753A44145522997`。
- 当前 ffprobe.exe SHA-256：`05E8FA639450F8191635192871AE37A3EC3E4638FA12F3B7D49C6522BA16A8ED`。
- [合规审查与剩余清单](docs/FFMPEG_COMPLIANCE.md)、[源码归档 URL/校验值](docs/compliance/ffmpeg-7.1.1/manifest.json)、
  [原包构建/依赖说明](docs/compliance/ffmpeg-7.1.1/UPSTREAM-README.txt)、
  [该提交的原始许可说明](docs/compliance/ffmpeg-7.1.1/LICENSE.md)。
- FFmpeg 含来自 Independent JPEG Group 的代码：This software is based in part on the work of the Independent JPEG Group.
  该致谢不表示已确认所有依赖的版权声明齐全，亦不表示上游未应用补丁。
- 自有源码采用 MIT 不改变 FFmpeg 的 GPLv3 义务。链接源码主页或附上 GPL 文本，
  不能代替与所分发二进制匹配的完整对应源码、依赖与构建材料。
- **公开二进制发布阻塞**：当前尚未备齐该第三方静态构建及所含库的完整对应源码和构建材料。
  因此本仓库只公开自有源码与许可证文本，不上传 FFmpeg/字体二进制或含它们的安装包；
  本地构建产物不可直接据此宣称已完成公开再分发合规。
  参见 [FFmpeg legal](https://ffmpeg.org/legal.html) 和
  [GPLv3](https://www.gnu.org/licenses/gpl-3.0.html)，发布者应先完成对应材料和分发审查。

## Noto Sans CJK

水印字体使用 Noto Sans CJK SC，版权归其贡献者所有，按 SIL Open Font License 1.1 分发。字体来源为 [notofonts/noto-cjk](https://github.com/notofonts/noto-cjk)，完整许可证随安装包位于 `resources/licenses/Noto-OFL.txt`。

## 应用与其他依赖

FrameMark 自有 Rust/TypeScript 源码、文档与 `assets/app-icon.svg` 及其生成图标使用 MIT。
Tauri 和官方插件采用 MIT 或 Apache-2.0；serde、rusqlite 等 Rust 库保留各自许可证，
SQLite 为公有领域；Vite/TypeScript 等开发工具保留 MIT/Apache-2.0 等原许可证。
依赖版本由 `Cargo.lock` 与 `package-lock.json` 锁定，不能将第三方组件重新声明为 MIT。

Windows WebView2 Evergreen Bootstrapper/Runtime 属于微软组件，按微软许可使用和安装；
不包含在本项目 MIT 授权中，也不向系统目录复制未经来源确认的 DLL。
