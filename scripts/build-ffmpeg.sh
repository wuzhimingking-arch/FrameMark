#!/usr/bin/env bash
set -euo pipefail
# Build in a Linux filesystem; preserve the inputs and this script with the binaries.
root=${1:-/opt/framemark-ffmpeg}
cd "$root"
script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
mkdir -p sources
python3 - "$script_dir/ffmpeg-sources.json" <<'PY'
import hashlib, json, pathlib, subprocess, sys
for entry in json.load(open(sys.argv[1])):
    if entry.get('headersArchive') and pathlib.Path('sources', entry['headersArchive']).exists():
        subset = pathlib.Path('sources', entry['headersArchive'])
        if hashlib.sha256(subset.read_bytes()).hexdigest() != entry['headersSha256']:
            raise SystemExit('AMF headers checksum mismatch')
        pathlib.Path('amf').mkdir(exist_ok=True)
        subprocess.run(['tar', '-xzf', str(subset), '-C', 'amf'], check=True)
        continue
    path = pathlib.Path('sources') / (entry['name'] + '.tar.gz')
    if not path.exists():
        subprocess.run(['curl', '-fL', '--retry', '3', '-o', str(path), entry['url']], check=True)
    if hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
        raise SystemExit('Source checksum mismatch: ' + str(path))
PY
for n in zlib freetype harfbuzz nv-codec-headers amf libvpl ffmpeg x264; do
    if ! test -d "$n"; then
        mkdir "$n"
        tar -xzf "sources/$n.tar.gz" -C "$n" --strip-components=1
    fi
done
prefix="$root/prefix"
export PKG_CONFIG_LIBDIR="$prefix/lib/pkgconfig" PKG_CONFIG_PATH="$prefix/lib/pkgconfig"
mkdir -p "$prefix" logs
cat > mingw.cmake <<EOF
set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_C_COMPILER x86_64-w64-mingw32-gcc)
set(CMAKE_CXX_COMPILER x86_64-w64-mingw32-g++)
set(CMAKE_RC_COMPILER x86_64-w64-mingw32-windres)
set(CMAKE_FIND_ROOT_PATH $prefix)
EOF
cmake -S zlib -B zlib-build -G Ninja -DCMAKE_TOOLCHAIN_FILE="$root/mingw.cmake" -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_BUILD_TYPE=Release -DZLIB_BUILD_EXAMPLES=OFF
cmake --build zlib-build -j 8
cmake --install zlib-build
cp "$prefix/lib/libzlibstatic.a" "$prefix/lib/libz.a"
cmake -S freetype -B freetype-build -G Ninja -DCMAKE_TOOLCHAIN_FILE="$root/mingw.cmake" -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DFT_DISABLE_ZLIB=ON -DFT_DISABLE_BZIP2=ON -DFT_DISABLE_PNG=ON -DFT_DISABLE_HARFBUZZ=ON -DFT_DISABLE_BROTLI=ON
cmake --build freetype-build -j 8
cmake --install freetype-build
cmake -S harfbuzz -B harfbuzz-build -G Ninja -DCMAKE_TOOLCHAIN_FILE="$root/mingw.cmake" -DCMAKE_PREFIX_PATH="$prefix" -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DHB_BUILD_UTILS=OFF -DHB_HAVE_FREETYPE=ON -DHB_HAVE_GLIB=OFF -DHB_HAVE_ICU=OFF -DCMAKE_CXX_FLAGS="-static-libgcc -static-libstdc++"
cmake --build harfbuzz-build -j 8
cmake --install harfbuzz-build
(cd x264; ./configure --host=x86_64-w64-mingw32 --cross-prefix=x86_64-w64-mingw32- --prefix="$prefix" --enable-static --disable-cli --disable-opencl; make -j 8; make install)
make -C nv-codec-headers PREFIX="$prefix" install
mkdir -p "$prefix/include/AMF"
cp -r amf/amf/public/include/* "$prefix/include/AMF/"
cmake -S libvpl -B vpl-build -G Ninja -DCMAKE_TOOLCHAIN_FILE="$root/mingw.cmake" -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DBUILD_TOOLS=OFF -DBUILD_EXAMPLES=OFF -DBUILD_TESTS=OFF -DINSTALL_EXAMPLE_CODE=OFF
cmake --build vpl-build -j 8
cmake --install vpl-build
(cd ffmpeg; ./configure --prefix="$prefix" --target-os=mingw32 --arch=x86_64 --cross-prefix=x86_64-w64-mingw32- --enable-cross-compile --pkg-config=pkg-config --pkg-config-flags=--static --extra-cflags="-I$prefix/include" --extra-ldflags="-L$prefix/lib -static" --extra-libs="-lstdc++" --disable-autodetect --disable-shared --enable-static --disable-network --disable-doc --disable-debug --enable-gpl --enable-version3 --enable-zlib --enable-libx264 --enable-libfreetype --enable-libharfbuzz --enable-ffnvcodec --enable-nvenc --enable-nvdec --enable-amf --enable-libvpl; make -j 8; make install)
sha256sum sources/* "$prefix/bin/ffmpeg.exe" "$prefix/bin/ffprobe.exe" > SHA256SUMS
cp ffmpeg/ffbuild/config.log logs/ffmpeg-config.log
cp ffmpeg/config.h logs/ffmpeg-config.h
x86_64-w64-mingw32-gcc --version > logs/compiler.txt
