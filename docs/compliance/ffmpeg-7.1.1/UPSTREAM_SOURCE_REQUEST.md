# 上游材料索取草稿（未发送）

维护者公开要求从组织官方邮箱提出请求并附应用和组织链接。
由项目所有者选择官方邮箱及收件方式；不要在仓库或聊天里粘贴邮箱密码、Token。
本文件仅准备请求内容，没有代用户联系上游。

参考：https://github.com/GyanD/codexffmpeg/issues/249#issuecomment-6264377917

Subject: Corresponding source and build materials for FFmpeg 7.1.1 Essentials (FrameMark)

Hello Gyan,

We are preparing a public Windows distribution of FrameMark, a local desktop
video-watermarking application. It invokes FFmpeg/ffprobe as separate processes.

- Application: https://github.com/wuzhimingking-arch/FrameMark
- Organization/product website: https://peakedge.tech
- Build: 7.1.1-essentials_build-www.gyan.dev (release dated March 12, 2025)
- Archive: ffmpeg-7.1.1-essentials_build.7z
- ffmpeg.exe SHA-256: b90225987bdd042cca09a1efb5e34e9848f2d1dbf5fbcd388753a44145522997
- ffprobe.exe SHA-256: 05e8fa639450f8191635192871ae37a3ec3e4638fa12f3b7d49c6522ba16a8ed
- FFmpeg source revision: db69d06eeeab4f46da15030a80d539efb4503ca8

We have the original archive, its README, the GPL text and the upstream FFmpeg
source at this revision. We have not yet publicly distributed our installer.

Could you provide or identify the complete corresponding source and necessary
build materials for these exact binaries, including:

1. Source revisions/archives, patches or other modifications for the included
   FFmpeg code and libraries, and the included transitive dependencies.
2. Scripts and configuration needed to build these components and FFmpeg;
   relevant build environment/package records where needed to identify inputs.
   We are not requesting sources for tools excluded by the GPL's exceptions.
3. Original copyright/license notices and clarification of any applicable
   exceptions or redistributability restrictions for included SDK/header code.
4. Any existing stable source bundle and instructions we may use for downstream
   distribution under GPLv3 section 6(d).

The original package README does not identify versions for fontconfig, iconv,
GnuTLS, libxml2, GMP, bzip2, XZ/liblzma, zlib or OpenJPEG. We would especially
appreciate identification of those sources and any patches used. Configure flags
alone do not give us the complete dependency build materials.

Thank you.
