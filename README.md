# 帧印（shuiyingongju）

帧印是一款 Windows 桌面工具：从视频的连续时间区间中按真实视频帧选择单帧，并生成带版权水印的输出图片。

## 功能与边界

- 每个连续时间区间选择且只选择一帧
- 水印完整矩形保持在安全区域内
- 相同 Seed 和输入参数生成相同布局
- 不修改或覆盖原视频，取消或失败时清理未完成输出
- 音频兼容时流复制，视频默认使用高质量 H.264 重编码
- 本地处理视频，不上传文件，不包含账号、云端、AI 或剪辑功能

## 技术栈

Tauri 2、原生 TypeScript/HTML/CSS、Rust、SQLite，以及随应用分发的 FFmpeg/ffprobe。

Windows 10/11 x64 是 V1 首要目标。当前版本：`1.0.2`，产品名称为“帧印”。

## 开发与验证

```powershell
npm install
npm run build
cd src-tauri
cargo test
cd ..
npm run verify:windows
```

Windows 正式发行使用 NSIS 安装包；第三方组件和许可证见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
