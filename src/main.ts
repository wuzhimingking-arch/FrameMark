import "./styles.css";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";

type Identity = { id: number; platform: string; text: string; enabled: boolean };
type Settings = { interval: number; opacity: number; fontSize: string; outputDir: string; randomOpacity: boolean; randomSize: boolean; safeMargin: number; encoder: string };
type EncodingOption = { id: string; label: string; available: boolean };
type MediaInfo = { path: string; fileName: string; size: number; duration: number; width: number; height: number; fps: number; fpsText: string; videoCodec: string; audioCodec: string; hasAudio: boolean; rotation: number; hdr: boolean; hdrDescription: string };
type HistorySummary = { id: number; originalName: string; outputPath: string; platform: string; watermarkCount: number; createdAt: number; fileExists: boolean };
type Watermark = { number: number; timestamp: number; pts: number | null; frameIndex: number; x: number; y: number; fontSize: number; opacity: number; text: string };
type HistoryDetail = { id: number; originalName: string; originalPath: string; outputName: string; outputPath: string; platform: string; watermarkText: string; interval: number; seed: string; width: number; height: number; duration: number; fps: number; videoCodec: string; audioCodec: string; createdAt: number; fileExists: boolean; watermarks: Watermark[] };
type Bootstrap = { identities: Identity[]; settings: Settings; histories: HistorySummary[]; logDir: string; encodingOptions: EncodingOption[] };
type ProgressEvent = { stage: string; percent: number; processed: number; total: number };
type ProcessResult = { historyId: number; outputPath: string; watermarkCount: number; seed: string; encoderLabel: string };

const root = document.querySelector<HTMLDivElement>("#app")!;
let data: Bootstrap = { identities: [], settings: { interval: 3, opacity: .45, fontSize: "medium", outputDir: "", randomOpacity: false, randomSize: false, safeMargin: .05, encoder: "auto" }, histories: [], logDir: "", encodingOptions: [] };
let media: MediaInfo | null = null;
let route: "process" | "history" | "settings" = "process";
let currentDetail: HistoryDetail | null = null;
let dragging = false;
const isTauri = "__TAURI_INTERNALS__" in window;

const html = (value: unknown) => String(value ?? "").replace(/[&<>'"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" })[char]!);
const errorText = (error: unknown) => typeof error === "string" ? error : error instanceof Error ? error.message : "操作失败，请查看日志";
const time = (seconds: number, milliseconds = false) => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor(seconds % 3600 / 60);
  const secs = Math.floor(seconds % 60);
  const prefix = hours ? `${String(hours).padStart(2, "0")}:` : "";
  return `${prefix}${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}${milliseconds ? `.${String(Math.round(seconds % 1 * 1000)).padStart(3, "0")}` : ""}`;
};
const size = (bytes: number) => bytes >= 1_073_741_824 ? `${(bytes / 1_073_741_824).toFixed(2)} GB` : `${(bytes / 1_048_576).toFixed(1)} MB`;
const date = (seconds: number) => new Intl.DateTimeFormat("zh-CN", { dateStyle: "medium", timeStyle: "short" }).format(new Date(seconds * 1000));

function shell(content: string) {
  root.innerHTML = `
    <div class="app-shell">
      <aside class="sidebar">
        <div class="logo"><span class="logo-mark">帧</span><div><strong>帧印</strong><small>FrameMark</small></div></div>
        <nav aria-label="主导航">
          <button class="nav-item ${route === "process" ? "active" : ""}" data-route="process"><span>◫</span>处理视频</button>
          <button class="nav-item ${route === "history" ? "active" : ""}" data-route="history"><span>◷</span>历史记录</button>
          <button class="nav-item ${route === "settings" ? "active" : ""}" data-route="settings"><span>⚙</span>设置</button>
        </nav>
        <div class="privacy"><span>●</span><div><strong>100% 本地处理</strong><small>原视频不离开你的电脑</small></div></div>
        <div class="version">V1.0.2</div>
      </aside>
      <main class="content">${content}</main>
    </div>
    <div id="toast" role="status" aria-live="polite"></div>
    <dialog id="identity-dialog"></dialog>
    <dialog id="progress-dialog" class="modal"></dialog>
    <dialog id="complete-dialog" class="modal"></dialog>`;
  bindShell();
}

function bindShell() {
  document.querySelectorAll<HTMLButtonElement>("[data-route]").forEach((button) => button.onclick = () => {
    route = button.dataset.route as typeof route;
    currentDetail = null;
    render();
  });
}

function render() {
  if (route === "history") renderHistory();
  else if (route === "settings") renderSettings();
  else renderProcess();
}

function renderProcess() {
  const enabled = data.identities.filter((identity) => identity.enabled);
  const selected = enabled[0];
  shell(`
    <header class="page-header"><div><p class="eyebrow">单帧版权保护</p><h1>处理视频</h1><p>每个时间区间随机一帧，安全区域内随机坐标。</p></div><span class="local-badge">● 本地处理</span></header>
    <div class="workspace">
      <section class="preview-card card">
        ${media ? `
          <div class="video-stage" id="video-stage">
            <video id="preview-video" src="${html(convertFileSrc(media.path))}" controls preload="metadata"></video>
            <div id="safe-guide" class="safe-guide"></div><div id="preview-mark" class="preview-mark">${html(selected?.text ?? "请添加平台身份")}</div>
          </div>
          <div class="file-summary"><div class="file-icon">▶</div><div><strong>${html(media.fileName)}</strong><span>${media.width} × ${media.height} · ${media.fps.toFixed(2)} FPS · ${html(media.videoCodec.toUpperCase())}</span></div><button id="change-video" class="icon-button" aria-label="更换视频">↻</button></div>
          <div class="media-grid">
            <div><span>时长</span><strong>${time(media.duration)}</strong></div><div><span>文件大小</span><strong>${size(media.size)}</strong></div><div><span>视频编码</span><strong>${html(media.videoCodec.toUpperCase())}</strong></div><div><span>音频编码</span><strong>${html(media.audioCodec.toUpperCase() || "无")}</strong></div>
          </div>
          ${media.hdr ? `<div class="warning">⚠ 检测到 HDR（${html(media.hdrDescription)}）。当前 H.264 管线可能改变 HDR 色彩，开始前会再次确认。</div>` : ""}` : `
          <button id="drop-zone" class="drop-zone ${dragging ? "dragging" : ""}"><span class="upload-icon">＋</span><strong>拖入已经剪辑完成的视频</strong><small>或点击选择文件 · MP4 / MOV / MKV / WebM</small><em>选择视频</em></button>`}
        <div class="preview-toolbar" ${media ? "" : "hidden"}><label class="switch-line"><input id="show-safe" type="checkbox" checked> 显示安全区域</label><button id="random-preview" class="text-button">⤨ 重新随机位置</button></div>
      </section>
      <section class="controls-card card">
        <div class="section-title"><div><h2>水印参数</h2><p>随机位置始终开启</p></div><span class="random-chip">✦ 完全随机</span></div>
        <label>平台身份<select id="identity-select" ${enabled.length ? "" : "disabled"}>${enabled.map((identity) => `<option value="${identity.id}">${html(identity.platform)} · ${html(identity.text)}</option>`).join("")}</select></label>
        ${enabled.length ? `<div id="identity-preview" class="identity-preview"><span>${html(selected?.platform)}</span><strong>${html(selected?.text)}</strong></div>` : `<button id="add-first-identity" class="empty-action">＋ 先添加一个平台身份</button>`}
        <div class="field"><span>水印间隔</span><div class="interval-row"><select id="interval-select"><option value="1">1 秒</option><option value="3" selected>3 秒</option><option value="5">5 秒</option><option value="10">10 秒</option><option value="custom">自定义</option></select><input id="custom-interval" type="number" min="0.5" max="60" step="0.1" value="${data.settings.interval}" hidden><span id="interval-hint">每个 3 秒区间随机 1 帧</span></div></div>
        <div class="field"><span>字体大小</span><div class="segments" id="font-size"><button data-value="small">小</button><button data-value="medium" class="active">中</button><button data-value="large">大</button></div></div>
        <label class="range-field"><span>透明度 <strong id="opacity-value">${Math.round(data.settings.opacity * 100)}%</strong></span><input id="opacity" type="range" min="20" max="100" value="${data.settings.opacity * 100}"></label>
        <div class="inline-fields"><label>文字颜色<select id="color"><option value="white">白色</option><option value="black">黑色</option></select></label><label>文字背景<select id="background"><option value="false">无背景</option><option value="true">半透明背景</option></select></label></div>
        <details><summary>高级设置</summary><div class="advanced"><label class="switch-line"><input id="random-opacity" type="checkbox" ${data.settings.randomOpacity ? "checked" : ""}> 随机透明度 <small>基础值 ±10%</small></label><label class="switch-line"><input id="random-size" type="checkbox" ${data.settings.randomSize ? "checked" : ""}> 随机大小 <small>基础值 ±12.5%</small></label><label class="range-field"><span>安全边距 <strong id="margin-value">${Math.round(data.settings.safeMargin * 100)}%</strong></span><input id="safe-margin" type="range" min="2" max="20" value="${data.settings.safeMargin * 100}"></label><button id="choose-output" class="path-button">输出目录：${html(data.settings.outputDir || "原视频目录")}</button></div></details>
        <button id="start-process" class="primary" ${media && enabled.length ? "" : "disabled"}>开始处理 <span>→</span></button>
        <p class="safe-note">不会修改或覆盖原视频</p>
      </section>
    </div>`);
  bindProcess();
  if (media) requestAnimationFrame(randomPreview);
}

function bindProcess() {
  document.querySelector("#drop-zone")?.addEventListener("click", chooseVideo);
  document.querySelector("#change-video")?.addEventListener("click", chooseVideo);
  document.querySelector("#add-first-identity")?.addEventListener("click", () => showIdentityDialog());
  document.querySelector("#identity-select")?.addEventListener("change", () => {
    const identity = selectedIdentity();
    const preview = document.querySelector("#identity-preview");
    const mark = document.querySelector("#preview-mark");
    if (preview && identity) preview.innerHTML = `<span>${html(identity.platform)}</span><strong>${html(identity.text)}</strong>`;
    if (mark && identity) mark.textContent = identity.text;
    randomPreview();
  });
  const intervalSelect = document.querySelector<HTMLSelectElement>("#interval-select");
  const custom = document.querySelector<HTMLInputElement>("#custom-interval");
  if (intervalSelect && custom) {
    const defaults = [1, 3, 5, 10];
    if (defaults.includes(data.settings.interval)) intervalSelect.value = String(data.settings.interval);
    else { intervalSelect.value = "custom"; custom.hidden = false; }
    intervalSelect.onchange = () => { custom.hidden = intervalSelect.value !== "custom"; intervalHint(); };
    custom.oninput = intervalHint;
    intervalHint();
  }
  document.querySelectorAll<HTMLButtonElement>("#font-size button").forEach((button) => {
    button.classList.toggle("active", button.dataset.value === data.settings.fontSize);
    button.onclick = () => { document.querySelectorAll("#font-size button").forEach((item) => item.classList.remove("active")); button.classList.add("active"); randomPreview(); };
  });
  const opacity = document.querySelector<HTMLInputElement>("#opacity");
  if (opacity) opacity.oninput = () => { document.querySelector("#opacity-value")!.textContent = `${opacity.value}%`; randomPreview(); };
  const margin = document.querySelector<HTMLInputElement>("#safe-margin");
  if (margin) margin.oninput = () => { document.querySelector("#margin-value")!.textContent = `${margin.value}%`; updateSafeGuide(); randomPreview(); };
  document.querySelector("#color")?.addEventListener("change", randomPreview);
  document.querySelector("#background")?.addEventListener("change", randomPreview);
  document.querySelector("#show-safe")?.addEventListener("change", (event) => document.querySelector("#safe-guide")?.classList.toggle("hidden", !(event.target as HTMLInputElement).checked));
  document.querySelector("#random-preview")?.addEventListener("click", randomPreview);
  document.querySelector("#choose-output")?.addEventListener("click", chooseOutput);
  document.querySelector("#start-process")?.addEventListener("click", startProcess);
}

function intervalHint() {
  const value = processInterval();
  const hint = document.querySelector("#interval-hint");
  if (hint) hint.textContent = `每个 ${Number.isFinite(value) ? value : "—"} 秒区间随机 1 帧`;
}

function processInterval() {
  const preset = document.querySelector<HTMLSelectElement>("#interval-select")?.value ?? "3";
  return Number(preset === "custom" ? document.querySelector<HTMLInputElement>("#custom-interval")?.value : preset);
}

function selectedIdentity() {
  const id = Number(document.querySelector<HTMLSelectElement>("#identity-select")?.value);
  return data.identities.find((identity) => identity.id === id);
}

async function chooseVideo() {
  const selected = await open({ multiple: false, filters: [{ name: "视频", extensions: ["mp4", "mov", "mkv", "webm"] }] });
  if (typeof selected === "string") await loadVideo(selected);
}

async function loadVideo(path: string) {
  try {
    toast("正在读取视频信息……");
    media = await invoke<MediaInfo>("inspect_video", { path });
    route = "process";
    render();
  } catch (error) { toast(errorText(error), true); }
}

function updateSafeGuide() {
  const guide = document.querySelector<HTMLElement>("#safe-guide");
  const value = Number(document.querySelector<HTMLInputElement>("#safe-margin")?.value ?? 5);
  if (guide) guide.style.inset = `${value}%`;
}

function randomPreview() {
  const stage = document.querySelector<HTMLElement>("#video-stage");
  const mark = document.querySelector<HTMLElement>("#preview-mark");
  if (!stage || !mark) return;
  updateSafeGuide();
  const margin = Number(document.querySelector<HTMLInputElement>("#safe-margin")?.value ?? 5) / 100;
  const opacity = Number(document.querySelector<HTMLInputElement>("#opacity")?.value ?? 45) / 100;
  const sizeName = document.querySelector<HTMLButtonElement>("#font-size .active")?.dataset.value ?? "medium";
  const ratio = sizeName === "small" ? .035 : sizeName === "large" ? .055 : .045;
  mark.style.fontSize = `${Math.max(12, stage.clientHeight * ratio)}px`;
  mark.style.opacity = String(opacity);
  mark.style.color = document.querySelector<HTMLSelectElement>("#color")?.value === "black" ? "#080b10" : "white";
  mark.classList.toggle("with-background", document.querySelector<HTMLSelectElement>("#background")?.value === "true");
  const left = stage.clientWidth * margin;
  const top = stage.clientHeight * margin;
  const maxX = Math.max(left, stage.clientWidth * (1 - margin) - mark.offsetWidth);
  const maxY = Math.max(top, stage.clientHeight * (1 - margin) - mark.offsetHeight);
  mark.style.left = `${left + Math.random() * (maxX - left)}px`;
  mark.style.top = `${top + Math.random() * (maxY - top)}px`;
}

async function chooseOutput() {
  const selected = await open({ directory: true, multiple: false, defaultPath: data.settings.outputDir || undefined });
  if (typeof selected === "string") {
    data.settings.outputDir = selected;
    const button = document.querySelector("#choose-output");
    if (button) button.textContent = `输出目录：${selected}`;
  }
}

async function startProcess() {
  const identity = selectedIdentity();
  if (!media || !identity) return;
  const interval = processInterval();
  if (!Number.isFinite(interval) || interval < .5 || interval > 60) { toast("水印间隔必须在 0.5 到 60 秒之间", true); return; }
  if (media.hdr && !confirm("检测到 HDR 视频。当前版本使用 H.264 高质量输出，HDR 色彩保持有限，处理后可能转换为 SDR。仍要继续吗？")) return;
  const dialog = document.querySelector<HTMLDialogElement>("#progress-dialog")!;
  dialog.innerHTML = `<div class="modal-body"><div class="spinner">✦</div><h2>正在处理视频</h2><p id="progress-file">${html(media.fileName)}</p><div class="progress-track"><div id="progress-bar"></div></div><div class="progress-meta"><strong id="progress-percent">0%</strong><span id="progress-time">00:00 / ${time(media.duration)}</span></div><p id="progress-stage">正在读取视频帧……</p><button id="cancel-process" class="danger-button">取消任务</button></div>`;
  dialog.showModal();
  document.querySelector("#cancel-process")?.addEventListener("click", async () => { await invoke("cancel_process"); });
  const request = {
    inputPath: media.path, outputDir: data.settings.outputDir, identityId: identity.id, interval,
    opacity: Number(document.querySelector<HTMLInputElement>("#opacity")!.value) / 100,
    fontSize: document.querySelector<HTMLButtonElement>("#font-size .active")?.dataset.value ?? "medium",
    color: document.querySelector<HTMLSelectElement>("#color")!.value,
    background: document.querySelector<HTMLSelectElement>("#background")!.value === "true",
    randomOpacity: document.querySelector<HTMLInputElement>("#random-opacity")!.checked,
    randomSize: document.querySelector<HTMLInputElement>("#random-size")!.checked,
    safeMargin: Number(document.querySelector<HTMLInputElement>("#safe-margin")!.value) / 100,
    encoder: data.settings.encoder,
  };
  try {
    const result = await invoke<ProcessResult>("process_video", { request });
    dialog.close();
    data.histories = await invoke("list_histories");
    showComplete(result, identity);
  } catch (error) {
    dialog.close();
    if (errorText(error) !== "任务已取消") toast(errorText(error), true);
    else toast("任务已取消，未保留不完整输出");
  }
}

function showComplete(result: ProcessResult, identity: Identity) {
  const dialog = document.querySelector<HTMLDialogElement>("#complete-dialog")!;
  dialog.innerHTML = `<div class="modal-body complete"><div class="success">✓</div><h2>处理完成</h2><p>${html(result.outputPath.split(/[\\/]/).pop())}</p><div class="result-grid"><div><span>版权标记</span><strong>${result.watermarkCount} 个</strong></div><div><span>视频编码</span><strong>${html(result.encoderLabel)}</strong></div><div><span>使用身份</span><strong>${html(identity.platform)} · ${html(identity.text)}</strong></div><div><span>随机 Seed</span><strong>${result.seed}</strong></div></div><div class="modal-actions"><button id="play-output" class="primary">播放视频</button><button id="reveal-output" class="secondary">打开文件位置</button></div><button id="close-complete" class="text-button">再次处理</button></div>`;
  dialog.showModal();
  document.querySelector("#play-output")?.addEventListener("click", () => openPath(result.outputPath));
  document.querySelector("#reveal-output")?.addEventListener("click", () => revealItemInDir(result.outputPath));
  document.querySelector("#close-complete")?.addEventListener("click", () => dialog.close());
}

function renderHistory() {
  shell(`<header class="page-header"><div><p class="eyebrow">本地处理记录</p><h1>历史记录</h1><p>精确时间、坐标和 Seed 永久保留，即使视频被移动。</p></div></header>
    <div class="history-layout"><section class="history-list card">
      ${data.histories.length ? data.histories.map((item) => `<button class="history-item ${currentDetail?.id === item.id ? "active" : ""}" data-history="${item.id}"><span class="history-thumb">▶</span><span class="history-copy"><strong>${html(item.originalName)}</strong><small>${html(item.platform)} · ${item.watermarkCount} 个标记 · ${date(item.createdAt)}</small></span><em class="${item.fileExists ? "ok" : "missing"}">${item.fileExists ? "文件正常" : "已移动或删除"}</em><b>›</b></button>`).join("") : `<div class="empty-state"><span>◷</span><h2>还没有处理记录</h2><p>完成第一个视频后，水印详情会显示在这里。</p><button data-route="process" class="primary">处理视频</button></div>`}
    </section><aside class="history-detail card" id="history-detail">${currentDetail ? detailHtml(currentDetail) : `<div class="detail-placeholder">选择一条记录查看水印时间与坐标</div>`}</aside></div>`);
  document.querySelectorAll<HTMLButtonElement>("[data-history]").forEach((button) => button.onclick = async () => {
    try { currentDetail = await invoke("history_detail", { id: Number(button.dataset.history) }); renderHistory(); } catch (error) { toast(errorText(error), true); }
  });
  document.querySelector("#open-history")?.addEventListener("click", () => currentDetail && openPath(currentDetail.outputPath));
  document.querySelector("#reveal-history")?.addEventListener("click", () => currentDetail && revealItemInDir(currentDetail.outputPath));
}

function detailHtml(item: HistoryDetail) {
  return `<div class="detail-head"><div><span>${html(item.platform)}</span><h2>${html(item.originalName)}</h2><p>${html(item.watermarkText)}</p></div><em class="${item.fileExists ? "ok" : "missing"}">${item.fileExists ? "输出文件正常" : "文件已移动或删除"}</em></div>
    <div class="detail-stats"><div><span>标记数量</span><strong>${item.watermarks.length}</strong></div><div><span>间隔</span><strong>${item.interval} 秒</strong></div><div><span>视频</span><strong>${item.width}×${item.height}</strong></div><div><span>Seed</span><strong>${item.seed}</strong></div></div>
    <div class="detail-actions"><button id="open-history" class="primary" ${item.fileExists ? "" : "disabled"}>播放视频</button><button id="reveal-history" class="secondary" ${item.fileExists ? "" : "disabled"}>打开文件夹</button></div>
    <h3>水印位置列表</h3><div class="watermark-table"><table><thead><tr><th>#</th><th>时间</th><th>帧</th><th>X</th><th>Y</th><th>字号</th><th>透明度</th></tr></thead><tbody>${item.watermarks.map((mark) => `<tr><td>${mark.number}</td><td>${time(mark.timestamp, true)}</td><td>${mark.frameIndex}</td><td>${mark.x}</td><td>${mark.y}</td><td>${mark.fontSize}</td><td>${Math.round(mark.opacity * 100)}%</td></tr>`).join("")}</tbody></table></div>`;
}

function renderSettings() {
  shell(`<header class="page-header"><div><p class="eyebrow">本地偏好</p><h1>设置</h1><p>默认参数会用于新的视频任务。</p></div></header>
    <div class="settings-grid"><section class="card settings-card"><div class="section-title"><div><h2>平台身份</h2><p>启用的身份可在处理页选择</p></div><button id="new-identity" class="secondary">＋ 新增身份</button></div>
      <div class="identity-list">${data.identities.length ? data.identities.map((identity) => `<div class="identity-row ${identity.enabled ? "" : "disabled"}"><span class="platform-avatar">${html(identity.platform.slice(0, 1))}</span><div><strong>${html(identity.platform)}</strong><small>${html(identity.text)}</small></div><label class="toggle"><input type="checkbox" data-toggle-identity="${identity.id}" ${identity.enabled ? "checked" : ""}><span></span></label><button data-edit-identity="${identity.id}" class="icon-button">✎</button><button data-delete-identity="${identity.id}" class="icon-button danger">×</button></div>`).join("") : `<div class="small-empty">尚未添加平台身份</div>`}</div>
    </section><form id="defaults-form" class="card settings-card"><div class="section-title"><div><h2>默认水印参数</h2><p>处理页仍可单独调整</p></div></div>
      <div class="form-grid"><label>默认间隔（秒）<input name="interval" type="number" min="0.5" max="60" step="0.1" value="${data.settings.interval}"></label><label>默认字体大小<select name="fontSize"><option value="small">小</option><option value="medium">中</option><option value="large">大</option></select></label><label>默认透明度（%）<input name="opacity" type="number" min="20" max="100" value="${data.settings.opacity * 100}"></label><label>安全边距（%）<input name="safeMargin" type="number" min="2" max="20" value="${data.settings.safeMargin * 100}"></label></div>
      <label class="switch-line"><input name="randomOpacity" type="checkbox" ${data.settings.randomOpacity ? "checked" : ""}> 默认开启随机透明度</label><label class="switch-line"><input name="randomSize" type="checkbox" ${data.settings.randomSize ? "checked" : ""}> 默认开启随机大小</label>
      <label>视频编码加速<select name="encoder">${data.encodingOptions.map((option) => `<option value="${option.id}" ${option.id === data.settings.encoder ? "selected" : ""} ${option.available ? "" : "disabled"}>${html(option.label)}${option.available ? "" : "（当前设备不可用）"}</option>`).join("")}</select><small>自动模式优先 NVIDIA、Intel、AMD，硬件初始化失败会自动回退 CPU。</small></label>
      <div class="output-setting"><span>默认输出目录</span><button id="settings-output" type="button" class="path-button">${html(data.settings.outputDir || "原视频目录")}</button></div><button class="primary" type="submit">保存默认设置</button>
    </form><section class="card settings-card about-card"><h2>隐私与日志</h2><p>视频、截图、平台 ID 和历史记录均不会上传。运行日志仅保存在本机。</p><button id="open-logs" class="secondary">打开日志目录</button><div class="about-line"><span>帧印</span><strong>V1.0.2</strong></div></section></div>`);
  const fontSelect = document.querySelector<HTMLSelectElement>("[name=fontSize]"); if (fontSelect) fontSelect.value = data.settings.fontSize;
  bindSettings();
}

function bindSettings() {
  document.querySelector("#new-identity")?.addEventListener("click", () => showIdentityDialog());
  document.querySelectorAll<HTMLInputElement>("[data-toggle-identity]").forEach((input) => input.onchange = async () => {
    const identity = data.identities.find((item) => item.id === Number(input.dataset.toggleIdentity));
    if (identity) await persistIdentity({ ...identity, enabled: input.checked });
  });
  document.querySelectorAll<HTMLButtonElement>("[data-edit-identity]").forEach((button) => button.onclick = () => showIdentityDialog(data.identities.find((item) => item.id === Number(button.dataset.editIdentity))));
  document.querySelectorAll<HTMLButtonElement>("[data-delete-identity]").forEach((button) => button.onclick = async () => {
    const id = Number(button.dataset.deleteIdentity);
    if (confirm("删除这个平台身份？已有历史记录不会受影响。")) { await invoke("delete_identity", { id }); data.identities = data.identities.filter((item) => item.id !== id); renderSettings(); }
  });
  document.querySelector("#settings-output")?.addEventListener("click", async () => { await chooseOutput(); renderSettings(); });
  document.querySelector("#open-logs")?.addEventListener("click", () => openPath(data.logDir));
  document.querySelector<HTMLFormElement>("#defaults-form")!.onsubmit = async (event) => {
    event.preventDefault(); const form = new FormData(event.currentTarget as HTMLFormElement);
    const settings: Settings = { interval: Number(form.get("interval")), opacity: Number(form.get("opacity")) / 100, fontSize: String(form.get("fontSize")), safeMargin: Number(form.get("safeMargin")) / 100, outputDir: data.settings.outputDir, randomOpacity: form.get("randomOpacity") === "on", randomSize: form.get("randomSize") === "on", encoder: String(form.get("encoder") ?? "auto") };
    try { await invoke("save_settings", { settings }); data.settings = settings; toast("默认设置已保存"); } catch (error) { toast(errorText(error), true); }
  };
}

function showIdentityDialog(identity?: Identity) {
  const dialog = document.querySelector<HTMLDialogElement>("#identity-dialog")!;
  dialog.innerHTML = `<form method="dialog" id="identity-form" class="dialog-form"><h2>${identity ? "编辑" : "新增"}平台身份</h2><label>平台名称<select name="platform"><option>抖音</option><option>Bilibili</option><option>小红书</option><option>YouTube</option><option>快手</option><option>TikTok</option><option>自定义</option></select></label><label>水印文字<input name="text" maxlength="80" required placeholder="例如 UID:12345678 或 @PeakEdge" value="${html(identity?.text ?? "")}"></label><div class="modal-actions"><button value="cancel" class="secondary">取消</button><button id="save-identity" value="default" class="primary">保存</button></div></form>`;
  const platform = dialog.querySelector<HTMLSelectElement>("[name=platform]")!;
  if (identity && [...platform.options].some((item) => item.value === identity.platform)) platform.value = identity.platform;
  dialog.showModal();
  dialog.querySelector("#save-identity")?.addEventListener("click", async (event) => {
    event.preventDefault(); const form = new FormData(dialog.querySelector<HTMLFormElement>("form")!);
    try { await persistIdentity({ id: identity?.id ?? 0, platform: String(form.get("platform")), text: String(form.get("text")), enabled: identity?.enabled ?? true }); dialog.close(); render(); } catch (error) { toast(errorText(error), true); }
  });
}

async function persistIdentity(identity: Identity) {
  const saved = await invoke<Identity>("save_identity", { identity });
  const index = data.identities.findIndex((item) => item.id === saved.id);
  if (index >= 0) data.identities[index] = saved; else data.identities.push(saved);
  render();
}

function toast(message: string, isError = false) {
  const element = document.querySelector<HTMLElement>("#toast");
  if (!element) return;
  element.textContent = message; element.className = isError ? "show error" : "show";
  window.setTimeout(() => element.classList.remove("show"), 3600);
}

if (isTauri) {
  await listen<ProgressEvent>("processing-progress", ({ payload }) => {
    const bar = document.querySelector<HTMLElement>("#progress-bar"); const percent = document.querySelector("#progress-percent"); const stage = document.querySelector("#progress-stage"); const elapsed = document.querySelector("#progress-time");
    if (bar) bar.style.width = `${payload.percent}%`; if (percent) percent.textContent = `${Math.round(payload.percent)}%`; if (stage) stage.textContent = payload.stage; if (elapsed) elapsed.textContent = `${time(payload.processed)} / ${time(payload.total)}`;
  });
  await getCurrentWebview().onDragDropEvent(async (event) => {
    dragging = event.payload.type === "enter" || event.payload.type === "over";
    document.querySelector("#drop-zone")?.classList.toggle("dragging", dragging);
    if (event.payload.type === "drop" && event.payload.paths[0]) await loadVideo(event.payload.paths[0]);
  });
  try { data = await invoke<Bootstrap>("bootstrap"); render(); } catch (error) { root.innerHTML = `<main class="fatal"><h1>帧印无法启动</h1><p>${html(errorText(error))}</p></main>`; }
} else {
  // Browser-only development preview; the packaged app always uses Rust/SQLite data.
  data.identities = [{ id: 1, platform: "Bilibili", text: "UID:12345678", enabled: true }];
  data.encodingOptions = [{ id: "auto", label: "自动（优先 GPU）", available: true }, { id: "cpu", label: "CPU（软件编码）", available: true }];
  render();
}
