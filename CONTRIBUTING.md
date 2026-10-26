# Contributing / 参与贡献

Please open an Issue with reproducible steps, application version, Windows version,
encoder and sanitized error messages. Do not attach private videos, watermark IDs,
databases, full local paths or credentials. A small synthetic reproduction is ideal.

欢迎提交问题、功能建议和 PR。请描述预期结果与实际结果，提供脱敏日志和合成样本；
不要上传私人视频、水印身份、数据库、个人路径或密钥。

1. Fork the repository and create a focused branch.
2. Keep local processing, input-file protection, CPU fallback, real PTS selection,
   single-frame events and legacy-database compatibility intact.
3. Run `npm ci`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`,
   then `cargo test --manifest-path src-tauri/Cargo.toml --lib` on Windows after preparing assets.
4. Video changes require decoded-frame checks for both modes, actual coordinates,
   audio, VFR and an available hardware encoder. Installer changes require
   `npm run tauri:build` and `npm run verify:windows`.
5. Update project context and append the development log when behavior changes.
6. Submit a PR explaining the problem, change and test evidence.

原始用户数据库和历史记录必须保留；不要通过删除数据库修复迁移。
项目新增自有代码按 MIT 发布；第三方组件继续遵循自己的许可证。
