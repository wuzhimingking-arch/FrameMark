# 第三方组件说明

## FFmpeg

发行构建使用 Gyan.dev 的 FFmpeg 7.1.1 Essentials 静态构建。FFmpeg 是独立 sidecar，不与本程序链接。当前构建启用了 GPLv3 组件，完整许可证随安装包位于 `resources/licenses/FFmpeg-GPLv3.txt`；构建配置及对应源码入口见 gyan.dev 的发行说明。

- 构建发布页：[GyanD/codexffmpeg 7.1.1](https://github.com/GyanD/codexffmpeg/releases/tag/7.1.1)。
- FFmpeg 对应源码提交：[db69d06eeeab4f46da15030a80d539efb4503ca8](https://github.com/FFmpeg/FFmpeg/commit/db69d06eeeab4f46da15030a80d539efb4503ca8)。
- 二进制启用 `--enable-gpl --enable-version3 --enable-static`、libx264/libx265、
  NVENC、libvpl/QSV、AMF，没有 `--enable-nonfree`。
- 当前 ffmpeg.exe SHA-256：`B90225987BDD042CCA09A1EFB5E34E9848F2D1DBF5FBCD388753A44145522997`。
- 当前 ffprobe.exe SHA-256：`05E8FA639450F8191635192871AE37A3EC3E4638FA12F3B7D49C6522BA16A8ED`。
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
