# V1.0 验证记录

验证日期：2026-08-12

## 自动化与构建

- `npm run build`：通过，TypeScript 严格检查与 Vite 生产构建成功。
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`：7 项通过。
- 随机算法：61 秒/3 秒间隔生成 21 个标记，含最后不足区间；相同 Seed 可复现，不同 Seed 不同。
- 坐标算法：1920×1080 下批量 1000 个位置均未越过 5% 安全区域，连续坐标不完全相同。

## 严格单帧与音频

- 样本：5 秒、10 FPS、H.264 + AAC，中文文件名 `北京旅行 测试01.mp4`，路径包含空格。
- 结果：3 秒间隔生成 2 个目标帧；解码后用 `signalstats` 扫描，仅两个目标帧亮度超过阈值。
- 前一帧和后一帧均未检测到水印；音频流存在，分辨率和时长校验通过。
- 取消测试：FFmpeg 被终止后，不完整输出与任务临时目录均删除。

## 视频矩阵

| 样本 | 验证结果 |
| --- | --- |
| 1920×1080 / 30 FPS | 输出分辨率、时长正常 |
| 3840×2160 / 60 FPS | 输出分辨率、时长正常 |
| 1080×1920 竖屏 | 输出方向和宽高正常 |
| 30 分钟长视频 | 流式处理完成，时长正常 |
| VFR | 输入与输出逐帧时间戳数量相同，误差小于 2ms |
| 90° Rotation Metadata | 自动旋转后为 360×640，输出 rotation 为 0 且方向正确 |

## Windows x64 发行

- Tauri release 程序启动成功，窗口标题正确且响应正常。
- NSIS current-user 安装退出码 0；已安装程序、ffmpeg、ffprobe、字体、GPLv3/OFL 许可证存在。
- 已安装程序可启动；NSIS 卸载退出码 0，安装目录及卸载注册表项清除。
- 安装包：`帧印_1.0.0_x64-setup.exe`，约 62.9 MiB。

### 1.0.1 Windows 依赖修复验证

- Windows 目标由 `x86_64-pc-windows-gnullvm` 改为官方 MSVC ABI；PE 导入审计确认 `frame-mark.exe` 不再导入 `WebView2Loader.dll` 或 `libunwind.dll`。
- 同时审计 `frame-mark.exe`、`ffmpeg.exe`、`ffprobe.exe`：除 Windows 系统 DLL/API Set 外没有未随安装包提供的动态库。
- NSIS 配置为 `embedBootstrapper`；构建时下载并内嵌微软 WebView2 Evergreen Bootstrapper，安装脚本先检测已安装 Runtime，缺失时静默安装。
- 将正式安装包复制到随机临时目录，静默安装到独立目录；仅保留 `System32` 与 Windows 根目录的 PATH 后，从正式安装目录启动，窗口标题 `帧印` 正常出现。
- 已验证开始菜单快捷方式、FFmpeg/ffprobe、字体、许可证和卸载程序均存在；静默卸载后安装目录清理完成。
- 本机已有 WebView2 Runtime `151.0.4129.78`，因此未破坏性卸载共享 Runtime；缺失 Runtime 分支通过生成的 NSIS 安装逻辑和内嵌 Bootstrapper 配置验证。
- 安装包：`帧印_1.0.1_x64-setup.exe`，67,447,460 字节；SHA-256：`0720D7E626DDFAA61DB3CCC481CA781E5B2BE868C571A149979C598F63432B99`。

## 未覆盖范围

- 未在独立 Windows 10 与 macOS 机器执行安装和处理。
- 4K 与 30 分钟使用合成内容验证正确性，不作为复杂实拍内容的性能基准。
- HDR/Dolby Vision 当前仅检测并警告，不承诺 HDR 色彩无损保持。
