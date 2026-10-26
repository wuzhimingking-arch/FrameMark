import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, existsSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--manifest-path", "src-tauri/Cargo.toml", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"], { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }));
const output = ["Generated dependency notices. Third-party licenses are not replaced by FrameMark's MIT license."];
function add(name, version, license, source, directory, extra) {
  output.push("\n===== " + name + " " + version + " =====\nSPDX: " + (license || "See license files") + "\nSource: " + source);
  const names = readdirSync(directory, { withFileTypes: true }).filter(e => e.isFile() && /^(?:licen[sc]e|copying|notice)(?:$|[._-])/i.test(e.name)).map(e => e.name);
  if (extra && !names.includes(extra) && existsSync(path.resolve(directory, extra))) names.push(extra);
  for (const file of names.sort()) output.push("\n--- " + path.basename(file) + " ---\n" + readFileSync(path.resolve(directory, file), "utf8"));
}
for (const pkg of metadata.packages.filter(p => p.source).sort((a, b) => a.name.localeCompare(b.name))) {
  add(pkg.name, pkg.version, pkg.license, "https://crates.io/api/v1/crates/" + pkg.name + "/" + pkg.version + "/download", path.dirname(pkg.manifest_path), pkg.license_file);
}
const lock = JSON.parse(readFileSync(path.join(root, "package-lock.json"), "utf8"));
for (const [key, pkg] of Object.entries(lock.packages).filter(([key]) => key).sort()) {
  const directory = path.join(root, key);
  if (!existsSync(directory)) continue;
  const manifest = JSON.parse(readFileSync(path.join(directory, "package.json"), "utf8"));
  add(manifest.name, manifest.version, manifest.license, pkg.resolved || manifest.repository?.url || "npm", directory);
}
writeFileSync(path.join(root, "src-tauri/resources/licenses/DEPENDENCIES.txt"), output.join("\n"), "utf8");
console.log("Generated Rust/npm dependency notices for installer resources.");
