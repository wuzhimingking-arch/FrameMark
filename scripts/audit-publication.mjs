import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
const objects = git("rev-list", "--objects", "--all").trim().split("\n");
const files = git("ls-files", "-z").split("\0").filter(Boolean);
const forbidden = /(?:^|\/)(?:node_modules|target|artifacts|dist|\.idea)\/|\.(?:mp4|mov|mkv|webm|sqlite3(?:-wal|-shm)?|db|log|keystore|pem|pfx)$|(?:^|\/)(?:\.env|local\.properties)$/i;
const credential = /(?:gh[pousr]_[A-Za-z0-9_]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[A-Z0-9]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|(?:api[_-]?key|password|token)\s*[:=]\s*["'][^"'\s]{16,}["'])/i;
let failures = 0;
const report = (kind, name) => { console.error(kind + ": " + name); failures++; };
for (const file of files) {
  if (forbidden.test(file)) report("forbidden tracked file", file);
  const bytes = readFileSync(file);
  if (!bytes.includes(0) && credential.test(bytes.toString("utf8"))) report("possible credential", file);
}
for (const entry of objects) {
  const space = entry.indexOf(" ");
  if (space < 0) continue;
  const sha = entry.slice(0, space), name = entry.slice(space + 1);
  if (forbidden.test(name)) report("forbidden historical path", name);
  const type = git("cat-file", "-t", sha).trim();
  if (type !== "blob") continue;
  const bytes = execFileSync("git", ["cat-file", "blob", sha], { maxBuffer: 32 * 1024 * 1024 });
  if (!bytes.includes(0) && credential.test(bytes.toString("utf8"))) report("possible historical credential", name + "@" + sha.slice(0, 8));
}
for (const file of ["README.md", "README.zh-CN.md"]) {
  if (/[A-Z]:[\\/]Users[\\/]|\/Users\/[^/]+\/|\/home\/[^/]+\//i.test(readFileSync(file, "utf8"))) report("personal path in README", file);
}
if (failures) process.exitCode = 1;
else console.log(`Publication scan passed: ${files.length} tracked files, ${objects.length} historical objects. Pattern scan does not replace manual review.`);
