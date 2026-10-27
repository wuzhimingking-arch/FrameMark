import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const manifest = JSON.parse(readFileSync(join(root, "docs/compliance/ffmpeg-7.1.1/manifest.json"), "utf8"));
const dir = resolve(process.argv[2] || join(root, "artifacts/compliance/FrameMark-1.1.0"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
function verify(path, expected) {
  const bytes = readFileSync(path);
  assert.equal(hash(bytes), expected, `SHA-256 mismatch: ${path}`);
  return bytes;
}

// Tauri stamps this one bundle-type marker when creating NSIS; no other difference is allowed.
function normalizeBundleType(bytes, type) {
  const marker = Buffer.from(`__TAURI_BUNDLE_TYPE_VAR_${type}`);
  const offset = bytes.indexOf(marker);
  assert.ok(offset >= 0, "Tauri bundle marker missing");
  assert.equal(bytes.indexOf(marker, offset + 1), -1, "Ambiguous Tauri bundle marker");
  const result = Buffer.from(bytes);
  result.write("UNK", offset + marker.length - 3, "ascii");
  return result;
}

// Self-check: the normalization must not conceal unrelated changes.
const fixture = Buffer.from("prefix__TAURI_BUNDLE_TYPE_VAR_NSSsuffix");
assert.deepEqual(normalizeBundleType(fixture, "NSS"), Buffer.from("prefix__TAURI_BUNDLE_TYPE_VAR_UNKsuffix"));
assert.throws(() => normalizeBundleType(Buffer.from("no marker"), "NSS"));
assert.throws(() => normalizeBundleType(Buffer.concat([fixture, fixture]), "NSS"));
assert.notDeepEqual(normalizeBundleType(fixture, "NSS"), Buffer.from("other!__TAURI_BUNDLE_TYPE_VAR_UNKsuffix"));

verify(join(root, "artifacts/windows", manifest.installer.file), manifest.installer.sha256);
verify(join(dir, manifest.upstreamArchive.file), manifest.upstreamArchive.sha256);
for (const source of manifest.collectedSourceArchives) verify(join(dir, source.file), source.sha256);
assert.deepEqual(
  readFileSync(join(root, "docs/compliance/ffmpeg-7.1.1/UPSTREAM-README.txt")),
  readFileSync(join(dir, "upstream/ffmpeg-7.1.1-essentials_build/README.txt")),
  "Upstream README must be preserved without edits",
);
const installed = join(dir, "installer-image");
for (const [name, sha256] of Object.entries(manifest.binaries)) verify(join(installed, name), sha256);
for (const name of ["ffmpeg", "ffprobe"]) {
  const sha256 = manifest.binaries[`${name}.exe`];
  verify(join(dir, "upstream/ffmpeg-7.1.1-essentials_build/bin", `${name}.exe`), sha256);
  verify(join(root, "src-tauri/binaries", `${name}-x86_64-pc-windows-msvc.exe`), sha256);
}
assert.deepEqual(
  normalizeBundleType(readFileSync(join(installed, "frame-mark.exe")), "NSS"),
  normalizeBundleType(readFileSync(join(root, "src-tauri/target/x86_64-pc-windows-msvc/release/frame-mark.exe")), "UNK"),
  "Installed application differs from local release beyond Tauri's bundle marker",
);
for (const name of ["FFmpeg-GPLv3.txt", "Noto-OFL.txt", "DEPENDENCIES.txt"]) {
  assert.deepEqual(readFileSync(join(installed, "resources/licenses", name)), readFileSync(join(root, "src-tauri/resources/licenses", name)));
}
assert.deepEqual(readFileSync(join(root, "docs/compliance/ffmpeg-7.1.1/COPYING.GPLv3")), readFileSync(join(installed, "resources/licenses/FFmpeg-GPLv3.txt")));
assert.deepEqual(readFileSync(join(installed, "resources/NotoSansSC-Regular.ttf")), readFileSync(join(root, "src-tauri/resources/NotoSansSC-Regular.ttf")));
const run = (name, arg) => execFileSync(join(installed, `${name}.exe`), [arg], {
  encoding: "utf8", windowsHide: true, stdio: ["ignore", "pipe", "pipe"],
});
const license = run("ffmpeg", "-L");
assert.match(license, /version 3.*or.*later/s);
const configuration = run("ffmpeg", "-buildconf");
assert.match(configuration, /--enable-gpl/);
assert.match(configuration, /--enable-version3/);
assert.doesNotMatch(configuration, /--enable-nonfree/);
assert.equal(
  configuration.trim().replace(/\s+/g, " "),
  readFileSync(join(root, "docs/compliance/ffmpeg-7.1.1/OBSERVED-BUILDCONF.txt"), "utf8").trim().replace(/\s+/g, " "),
  "Recorded build configuration differs from the shipped executable",
);
assert.equal(manifest.status, "BLOCKED");
assert.ok(manifest.blockers.length > 0);
const result = {
  checkedAt: new Date().toISOString(),
  dossierVersion: manifest.dossierVersion,
  integrityChecks: "PASS",
  sourceLinkage: "Matches archived release except Tauri bundle marker; not an independent reproducible-build proof",
  distributionReady: false,
  blockers: manifest.blockers,
  ffmpegVersion: run("ffmpeg", "-version"),
  ffprobeVersion: run("ffprobe", "-version"),
  configuration,
  license,
};
writeFileSync(join(dir, "verification.json"), JSON.stringify(result, null, 2) + "\n", "utf8");
console.log("Integrity checks PASS; public binary distribution BLOCKED. See manifest blockers and verification.json.");
// A successful integrity audit must never look like permission to publish.
process.exitCode = 2;
