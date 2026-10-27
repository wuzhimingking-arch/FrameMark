# Windows FFmpeg 自建流程

在 Ubuntu 24.04 / WSL 的 Linux 文件系统构建，避免在 `/mnt/c` 编译的性能和权限问题。

```sh
sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 nasm make cmake ninja-build pkg-config curl python3
bash scripts/build-ffmpeg.sh /opt/framemark-ffmpeg
```

`scripts/ffmpeg-sources.json` 固定每个原始归档的版本、下载地址和 SHA-256；脚本下载缺失输入并拒绝哈希不一致的输入。目录不可包含其他版本的已解压源码或编译缓存。首次构建请使用空目录；重复构建仅用于相同输入。源码无补丁，编译选项全部在脚本内。

保留 CPU libx264、内置 HEVC 解码、AAC、drawtext、FreeType、HarfBuzz、zlib/PNG、NVENC、QSV/libvpl 和 AMF。禁用第三方自动发现和网络；不链接旧 Gyan 包的 x265、vid.stab、Rubber Band、TLS、OpenJPEG 等外部库。硬件驱动仍由操作系统提供，不随包分发；可用性以真实编码探测为准。

本构建显式启用 GPL 与 version3，FFmpeg 可执行程序按 GPL-3.0-or-later 分发。FrameMark 仍通过独立子进程调用，不与 libav 链接。保留各依赖的原始许可证、版权说明以及完整源码。FreeType 采用其 FTL 许可；HarfBuzz 为 MIT 风格许可，oneVPL 与 AMF 为 MIT，NVIDIA 编码头文件保留其 MIT 许可，x264 为 GPL-2.0-or-later。编译器运行库保留 GCC Runtime Library Exception 等适用声明。

发布资料包必须包含全部 `sources/*.tar.gz`、本脚本、版本清单、`SHA256SUMS`、`logs/` 和许可证。公开发布时与二进制一起上传，不能只提供上游主页。工具链版本记录于 `logs/compiler.txt`；构建配置记录于 `logs/ffmpeg-config.log` 和 `logs/ffmpeg-config.h`。不声称不同工具链能生成字节相同的可执行文件。

在实际 Windows 下运行新 ffmpeg/ffprobe、完整 Rust 媒体回归及安装验收后，才可更新产品下载。旧 1.1.0 安装包留存用于比对，不公开补发。
