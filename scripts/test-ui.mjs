// Headless frontend regression with an explicit native-IPC fixture; not a desktop screenshot.
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { preview } from "vite";

const server = await preview({ preview: { host: "127.0.0.1", port: 4174, strictPort: true } });
const browser = await chromium.launch({ headless: true, channel: process.env.CI ? undefined : "chrome" });
try {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1050 }, deviceScaleFactor: 1.5 });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.addInitScript(() => {
    const items = [{ id: 1, text: "@FrameMark", enabled: true }, { id: 2, text: "© 2026 示例品牌\nexample.test", enabled: true }, { id: 3, text: "UID:12345678", enabled: true }];
    let settings = { interval: 3, opacity: .45, fontSize: "medium", outputDir: "", randomOpacity: false, randomSize: false, safeMargin: .05, encoder: "auto", selectedIds: [1, 2, 3] };
    window.qaCalls = [];
    window.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
      transformCallback: () => 0,
      convertFileSrc: () => "",
      invoke: async (cmd, args = {}) => {
        window.qaCalls.push({ cmd, args });
        if (cmd === "bootstrap") return { identities: items, settings, histories: [], logDir: "", encodingOptions: [{ id: "auto", label: "自动（优先 GPU）", available: true }, { id: "cpu", label: "CPU", available: true }] };
        if (cmd === "save_settings") { settings = args.settings; return; }
        if (cmd === "save_identity") {
          const item = { ...args.identity, id: args.identity.id || Math.max(0, ...items.map(x => x.id)) + 1, text: args.identity.text.trim() };
          if (!item.text) throw "水印内容不能为空";
          const i = items.findIndex(x => x.id === item.id);
          if (i < 0) items.push(item); else items[i] = item;
          return item;
        }
        if (cmd === "delete_identity") { items.splice(items.findIndex(x => x.id === args.id), 1); return; }
        if (cmd === "reorder_watermarks") { const sorted = args.ids.map(id => items.find(x => x.id === id)); items.splice(0, items.length, ...sorted); return; }
        if (cmd === "plugin:dialog|open") return "synthetic-demo.mp4";
        if (cmd === "inspect_video") return { path: "synthetic-demo.mp4", fileName: "demo.mp4", size: 1000000, duration: 5, width: 1920, height: 1080, fps: 30, fpsText: "30/1", videoCodec: "h264", audioCodec: "aac", hasAudio: true, rotation: 0, hdr: false, hdrDescription: "" };
        if (cmd === "preview_watermarks") return Uint8Array.from(atob("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLbtAAAAABJRU5ErkJggg=="), c => c.charCodeAt(0)).buffer;
        if (cmd === "process_video") return { outputPath: "demo_marked.mp4", historyId: 1, watermarkCount: args.request.mode === "simultaneous" ? 6 : 2, eventCount: 2, seed: "DEMO", encoderLabel: "CPU" };
        if (cmd === "list_histories") return [];
        if (cmd.startsWith("plugin:event|")) return 0;
        throw new Error("Unhandled IPC fixture: " + cmd);
      },
    };
  });
  await page.goto("http://127.0.0.1:4174");
  await page.locator("#drop-zone").click();
  await page.locator("#rendered-preview:not([hidden])").waitFor();
  assert.match(await page.locator("#config-summary").innerText(), /3 条水印.*2 次水印事件.*6 次绘制/);
  await page.locator("#watermark-mode").selectOption("sequential");
  assert.match(await page.locator("#config-summary").innerText(), /部分已选水印可能不会出现/);
  await page.locator("#next-preview").click();
  await page.waitForTimeout(250);
  assert.equal(await page.evaluate(() => window.qaCalls.filter(x => x.cmd === "preview_watermarks").at(-1).args.next), 1);
  await page.locator('[data-move-selected="0"][data-step="1"]').click();
  assert.deepEqual(await page.evaluate(() => window.qaCalls.filter(x => x.cmd === "save_settings").at(-1).args.settings.selectedIds), [2, 1, 3]);
  await page.locator("[data-select-mark='1']").uncheck();
  await page.locator("[data-select-mark='2']").uncheck();
  await page.locator("[data-select-mark='3']").uncheck();
  assert.equal(await page.locator("#start-process").isDisabled(), true);
  await page.locator("[data-select-mark='2']").check();
  await page.locator("#start-process").click();
  await page.locator("#complete-dialog[open]").waitFor();
  assert.match(await page.locator("#complete-dialog").innerText(), /2 次 \/ 2 次/);
  await page.locator("#close-complete").click();
  await page.locator("#manage-watermarks").click();
  assert.equal(await page.getByText("水印信息管理", { exact: true }).count(), 1);
  await page.locator("#new-identity").click();
  await page.locator("textarea[name=text]").fill("繁體中文 https://example.test\n© " + "长文字".repeat(55));
  await page.locator("#save-identity").click();
  assert.equal(await page.locator(".identity-row").count(), 4);
  await page.locator("[data-copy-identity='4']").click();
  assert.equal(await page.locator(".identity-row").count(), 5);
  await page.locator("[data-up-watermark='4']").click();
  assert.equal(await page.locator(".identity-row").count(), 5);
  assert.equal(await page.locator("[name=platform]").count(), 0);
  await page.locator("[data-edit-identity='4']").click();
  assert.match(await page.locator("textarea[name=text]").inputValue(), /繁體中文/);
  await page.locator("textarea[name=text]").fill("edited");
  await page.locator("#save-identity").click();
  page.once("dialog", dialog => dialog.accept());
  await page.locator("[data-delete-identity='4']").click();
  assert.equal(await page.locator(".identity-row").count(), 4);
  assert.equal(await page.locator("body").evaluate(el => el.scrollWidth <= window.innerWidth), true);
  await page.setViewportSize({width:980,height:680});
  assert.equal(await page.locator(".content").evaluate(el=>el.scrollWidth<=el.clientWidth+1),true);
  await page.locator('[data-route="process"]').first().click();
  assert.equal(await page.locator(".content").evaluate(el=>el.scrollWidth<=el.clientWidth+1),true);
  assert.deepEqual(errors, []);
  console.log("Headless frontend passed: selection/order, modes/counts, preview switching, empty-selection guard, CRUD/copy/reorder, multiline and 150% pixel density.");
} finally {
  await browser.close();
  await new Promise(resolve => server.httpServer.close(resolve));
}
