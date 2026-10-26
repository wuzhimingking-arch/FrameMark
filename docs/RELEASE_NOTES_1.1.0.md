# FrameMark 1.1.0 — 源码预发布 / Source pre-release

- 自由水印信息替代平台必填项，支持多行、复制、编辑、删除和排序。
- 多选水印：同时出现（同一随机帧多条）与依次闪现（按顺序循环）。
- 实测文字尺寸、换行、有限缩小与碰撞检查；预览和输出共用原生绘制代码。
- 保留原区间选帧、严格单帧、安全边界、Seed、NVENC/QSV/AMF 检测和 CPU 回退。
- 旧库事务迁移，不清空身份文字或历史；新历史保存完整多水印快照与算法版本。
- Windows 安装根目录增加带品牌图标的“帧印”入口，继续使用 GUI subsystem。
- 补齐 MIT 自有源码许可、中英文说明、第三方声明、贡献指南、Issue 模板与 Windows CI。

本地验证：Rust 12 项通过；CPU/NVENC 两种模式的 VFR 样本均为 10 个事件，
同时模式 30 次绘制、依次模式 10 次绘制，解码 75 帧，帧时间戳误差小于 2ms，
音频压缩流哈希保持一致。无头前端交互回归通过。

## 发布边界

此 Release 只提供 GitHub 自动生成的源码归档，不附 Windows 安装包。
本地安装包已构建并验证，但所含 GPLv3 FFmpeg 的完整对应源码、依赖和构建材料尚未备齐，
因此不公开上传该二进制。参见 THIRD_PARTY_NOTICES.md。

桌面截图采集尚未完成，无头 IPC 测试画面不会冒充桌面截图。
Intel/AMD 与 macOS 尚未在对应硬件/系统验证；输出为 H.264，HDR 色彩保持有限。

This is a source-only pre-release. FrameMark's original code is MIT; FFmpeg and
other dependencies retain their licenses. No installer is attached until the
complete corresponding source/build-material obligations for the bundled GPLv3
FFmpeg build are resolved. Desktop screenshots and non-NVIDIA/macOS hardware
validation are pending. See the English and Chinese README files.
