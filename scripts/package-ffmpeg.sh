#!/usr/bin/env bash
set -euo pipefail
root=${1:-/opt/framemark-ffmpeg}
project=$(cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
out="$root/corresponding-source"
mkdir -p "$out/sources" "$out/licenses" "$out/logs"
# AMF is header-only in this build; retain all included headers, omit unrelated SDK sample videos.
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -czf sources/amf-headers.tar.gz -C amf LICENSE.txt amf/public/include
for source in zlib ffmpeg x264 freetype harfbuzz nv-codec-headers libvpl amf-headers; do cp "sources/$source.tar.gz" "$out/sources/"; done
cp "$project/scripts/build-ffmpeg.sh" "$project/scripts/ffmpeg-sources.json" "$project/docs/build/FFMPEG_WINDOWS.md" "$out/"
cp ffmpeg/ffbuild/config.log "$out/logs/ffmpeg-config.log"
cp ffmpeg/config.h "$out/logs/ffmpeg-config.h"
x86_64-w64-mingw32-gcc --version > "$out/logs/compiler.txt"
dpkg-query -W gcc-mingw-w64-x86-64 mingw-w64-common nasm cmake make > "$out/logs/toolchain.txt"
sha256sum prefix/bin/ffmpeg.exe prefix/bin/ffprobe.exe > "$out/logs/BINARY_SHA256SUMS"
cp ffmpeg/COPYING.GPLv3 "$out/licenses/FFmpeg-GPLv3.txt"
cp ffmpeg/LICENSE.md "$out/licenses/FFmpeg-LICENSE.md"
cp x264/COPYING "$out/licenses/x264-GPLv2.txt"
cp zlib/LICENSE "$out/licenses/zlib-LICENSE.txt"
cp freetype/LICENSE.TXT "$out/licenses/FreeType-LICENSE.txt"
cp freetype/docs/FTL.TXT "$out/licenses/FreeType-FTL.txt"
cp freetype/docs/GPLv2.TXT "$out/licenses/FreeType-GPLv2.txt"
cp harfbuzz/COPYING "$out/licenses/HarfBuzz-COPYING.txt"
cp amf/LICENSE.txt "$out/licenses/AMF-MIT.txt"
cp libvpl/LICENSE "$out/licenses/oneVPL-MIT.txt"
cp nv-codec-headers/include/ffnvcodec/nvEncodeAPI.h "$out/licenses/NVIDIA-nvEncodeAPI-header.txt"
cp /usr/share/doc/mingw-w64-common/copyright "$out/licenses/MinGW-w64-copyright.txt"
cp /usr/share/doc/gcc-mingw-w64-x86-64/copyright "$out/licenses/GCC-copyright.txt"
(cd "$out"; find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -czf FrameMark-1.1.1-ffmpeg-corresponding-source.tar.gz -C "$out" sources licenses logs build-ffmpeg.sh ffmpeg-sources.json FFMPEG_WINDOWS.md SHA256SUMS
cp "$out/licenses/"* "$project/src-tauri/resources/licenses/"
mkdir -p "$project/artifacts/compliance/FrameMark-1.1.1"
cp FrameMark-1.1.1-ffmpeg-corresponding-source.tar.gz "$project/artifacts/compliance/FrameMark-1.1.1/"
