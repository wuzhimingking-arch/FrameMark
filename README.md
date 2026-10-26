# FrameMark — Random Single-Frame Video Watermarking

[简体中文](README.zh-CN.md) · [Contributing](CONTRIBUTING.md) · [License](LICENSE)

FrameMark is an open-source, local desktop application for creators. Save custom
copyright text, select several watermarks, and place them on a random real frame
inside each time interval, at independent random coordinates inside safe margins.
It aims to increase the effort of finding and removing every identity mark while
keeping interruption during ordinary playback brief.

Current version: **1.1.0**. Windows 10/11 x64 is the release target.
macOS is planned and has not been built or validated.

## Features

- Free text: names, channels, URLs, @handles, © notices and multiline text
  (up to 240 characters / 8 lines per saved watermark).
- Add, edit, duplicate, delete and reorder saved watermarks.
- Multi-select with independent ordering for a video.
- **Simultaneous**: all selected texts appear on the same one-frame event.
- **Sequential**: one text per event, cycling in selected order.
- Measured text dimensions, wrapping, bounded font reduction and collision rejection.
  An unfit combination stops before encoding; no watermark is silently omitted.
- Actual FFmpeg previews share the output layout and drawing code.
- H.264 encoding: NVENC → Intel QSV → AMD AMF → libx264 CPU in automatic mode.
  Availability requires a successful encode, not just an encoder listing.
  NVIDIA is locally tested; Intel/AMD require validation on corresponding hardware.
  Hardware failure retries with CPU. Apple VideoToolbox is reserved for future macOS work.
- Local SQLite history stores text snapshots, event/drawing counts, order, mode,
  seed, algorithm version, timestamps, frame indexes, coordinates and styling.
- Source video is never edited; compatible audio is copied, VFR timestamps are
  retained, cancellation removes incomplete output, and HDR is warned about.
- GUI-only Windows release, bundled FFmpeg/ffprobe and WebView2 installation support.

## How it works

A 60-second video at a 3-second interval has 20 events. Each interval contributes
one random decoded frame; a final partial interval contributes one too.
Three selected texts in simultaneous mode produce **20 events / 60 drawings**.
Sequential mode produces **20 events / 20 drawings**, rotating A → B → C → A.
If there are fewer events than selected texts, later sequential texts may not appear.

Only target frames are changed. Video is re-encoded to H.264, so unmarked frames
also undergo normal compression. This is not an invisible or indestructible mark:
re-encoding, frame-rate conversion, cropping, frame dropping or AI repair can remove it.
It does not guarantee prevention of reposting or legal proof of ownership.

## Screenshots

Desktop screenshots are pending for this source pre-release. No design mockup or IPC-fixture image is presented as a desktop screenshot. The headless frontend regression is runnable with `npm run test:ui`; real native rendering is covered by Rust/FFmpeg tests.

## Use

1. Open Settings and add your watermark text.
2. Import or drag in a finished video.
3. Check one or more watermarks; adjust their order and select an appearance mode.
4. Set interval, style and margins. Re-randomize the preview or inspect the next text.
5. Start processing. Progress and completion show the actual encoder.
6. Open the new video and retain its history for later frame-by-frame inspection.

Windows installs contain `frame-mark.exe` and a friendly `帧印.lnk` in the installation
root, plus a Start Menu entry. Removing a desktop shortcut does not remove these.
The application and installer share the included project icon.

## Download and distribution status

This public repository distributes **source code**. Build Windows locally using
the instructions below. No public installer download is claimed here.

The locally tested installer bundles a GPLv3 FFmpeg build. A public installer
release is withheld until its complete corresponding FFmpeg/dependency source and
build materials are assembled and checked. A source pre-release must not be
mistaken for a stable binary release. See [third-party notes](THIRD_PARTY_NOTICES.md).

## Build on Windows

Use Node.js **22.12+** (Node 24 recommended), npm, and current stable Rust with
the `x86_64-pc-windows-msvc` target. The tested toolchain was Node 24 / Rust 1.97.
Install the [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/)
(Visual Studio C++ build tools, Windows SDK and WebView2 Runtime).
Internet access is needed for developer dependencies/assets and Runtime bootstrap
installation. Application video processing stays local.

```powershell
npm ci
npm run prepare:assets
npm run tauri -- dev --target x86_64-pc-windows-msvc
```

`prepare:assets` downloads pinned FFmpeg 7.1.1 Essentials to `src-tauri/binaries`
using target-suffixed names, plus Noto Sans SC and licenses. End users do not set PATH.
Do not substitute a different FFmpeg without checking filters, drivers and licenses.

```powershell
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --lib
npm run tauri -- build --target x86_64-pc-windows-msvc
```

The normal Tauri build creates the NSIS installer under
`src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`.

For the project's alternative LLVM/cargo-xwin build environment, install
`cargo-xwin` (`cargo install --locked cargo-xwin`) and LLVM tools
(`clang.exe`, `ld.lld.exe`, `llvm-rc.exe`, `llvm-readobj.exe`) on PATH, then run:

```powershell
npm run tauri:build
npm run verify:windows
```

That build copies the installer to `artifacts/windows/`. Verification audits the GUI
subsystem, dynamic imports, embedded icon, root shortcut, FFmpeg resources,
Start Menu entry, isolated clean-PATH launch and uninstall.
It installs/uninstalls a temporary current-user copy and can update app shortcuts.
Run it on a Windows test account rather than during normal use.

Tests use synthetic content and cover migration, deterministic multi-layout,
1000 events, bounds/collisions, both modes on CPU and available GPU, decoded frames,
audio, VFR, rotation, Chinese/space paths and cancellation. CI performs Windows
format/test/frontend/installer builds; interactive install checks run locally.

## Architecture and limits

Tauri 2 + TypeScript/HTML/CSS; Rust coordinates FFmpeg/ffprobe and SQLite.
There is no server, login or video upload. Settings/history/logs live in normal
application data directories. Legacy identity rows and single-watermark histories
are migrated in place; historical text remains even after deleting a saved item.

MP4 H.264/H.265 inputs are the primary path. MOV/MKV/WebM containers are accepted
only when their video codec is supported. Output is H.264 MP4. Compatible audio
is copied; other audio is encoded to AAC. Text subtitle streams are converted to
MP4 text; unsupported subtitle codecs can fail explicitly. Data/attachment streams
are not copied. HDR/Dolby Vision color preservation is not guaranteed.
No face/subtitle avoidance, editing, logo watermarks or piracy scanning is included.

See [project context](docs/PROJECT_CONTEXT.md) and [validation evidence](docs/VALIDATION.md).

## License

FrameMark's original source, documentation and project icon are under the
[MIT License](LICENSE). Third-party components keep their own licenses:
the pinned FFmpeg sidecar is GPLv3 and the Noto font is SIL OFL 1.1.
They are not relicensed as MIT. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
