import { createServer } from "node:http";
import { readFileSync, statSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, extname, join, normalize, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn, spawnSync } from "node:child_process";

// QA-owned S6 browser fixture. It exercises the player-visible Viewer shell
// with a deterministic local bridge and fake world-feed transport. It does
// not start a runtime, provider, chain, or real gameplay session.
const scriptDir = dirname(fileURLToPath(import.meta.url));
const viewerRoot = resolve(scriptDir, "..");
const repoRoot = resolve(viewerRoot, "../..");
const taskUid = "task_872ecd04e2824c02a685e1fd6df63d03";
const runId = new Date().toISOString().replace(/[:.]/g, "-");
const outDir = resolve(repoRoot, ".pm/scratch", taskUid, "browser", runId);
const bundlePath = resolve(viewerRoot, ".software-safe-build/viewer.js");
const session = `player-visual-feedback-${process.pid}`;
const browserBin = process.env.AGENT_BROWSER_BIN || "agent-browser";
const skipBuild = process.argv.includes("--skip-build");
const statuses = ["ready", "replay", "empty", "gap", "unavailable"];
const viewports = [
  { name: "mobile", width: 390, height: 844 },
  { name: "tablet", width: 768, height: 1024 },
  { name: "desktop", width: 1440, height: 1000 },
];
const summary = {
  caseId: "S6-Q1-player-visual-feedback",
  taskUid,
  mode: "viewer_test_api_fixture",
  inputMode: "visible-ui-controls-plus-dom-readback",
  mockDisabled: false,
  fixtureBoundary: "deterministic QA bridge and fake world_feed; no runtime/provider/playability claim",
  status: "running",
  startedAt: new Date().toISOString(),
  viewports: {},
  feedStatuses: {},
  interactions: {},
};

mkdirSync(outDir, { recursive: true });

function assert(condition, message, details = undefined) {
  if (condition) return;
  const suffix = details === undefined ? "" : `\n${JSON.stringify(details, null, 2)}`;
  throw new Error(`${message}${suffix}`);
}

function run(command, args, { input, timeout = 30_000 } = {}) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      rejectRun(new Error(`timed out: ${command} ${args.join(" ")}`));
    }, timeout);
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => { stdout += chunk; });
    child.stderr.on("data", (chunk) => { stderr += chunk; });
    child.on("error", (error) => {
      clearTimeout(timer);
      rejectRun(error);
    });
    child.on("close", (code, signal) => {
      clearTimeout(timer);
      if (code === 0) {
        resolveRun(stdout);
        return;
      }
      rejectRun(new Error([
        `command failed: ${command} ${args.join(" ")}`,
        `exit=${code ?? "null"} signal=${signal ?? "null"}`,
        stdout.trim() ? `stdout:\n${stdout.trim()}` : null,
        stderr.trim() ? `stderr:\n${stderr.trim()}` : null,
      ].filter(Boolean).join("\n")));
    });
    if (input !== undefined) child.stdin.end(input);
    else child.stdin.end();
  });
}

async function browserJson(args, options = {}) {
  const output = await run(browserBin, ["--session", session, "--headed", "--json", ...args], options);
  const parsed = JSON.parse(output);
  if (!parsed.success) throw new Error(parsed.error || `agent-browser failed: ${args.join(" ")}`);
  return parsed.data;
}

async function browserRaw(args, options = {}) {
  return run(browserBin, ["--session", session, "--headed", ...args], options);
}

async function evalJson(source) {
  const data = await browserJson(["eval", "--stdin"], { input: source });
  const result = data.result;
  return typeof result === "string" ? JSON.parse(result) : result;
}

function closeBrowser() {
  spawnSync(browserBin, ["--session", session, "close"], { stdio: "ignore", timeout: 10_000 });
}

function contentType(pathname) {
  const type = extname(pathname);
  if (type === ".html") return "text/html; charset=utf-8";
  if (type === ".js" || type === ".mjs") return "text/javascript; charset=utf-8";
  if (type === ".css") return "text/css; charset=utf-8";
  if (type === ".wasm") return "application/wasm";
  if (type === ".ico") return "image/x-icon";
  return "application/octet-stream";
}

function feedPayload(status) {
  const event = {
    event_seq: "1",
    kind: "resource_change",
    summary: "Fixture ambient resource update",
    detail: "QA fixture event; no player action receipt.",
    receipt_ref: null,
  };
  return {
    schema_version: "world_feed/v1",
    world_id: "fixture-world",
    reorg_epoch: "0",
    cursor: status === "empty" ? "" : "1",
    status,
    events: ["ready", "replay"].includes(status) ? [event] : [],
    gap_reason: status === "gap" ? "cursor_gap" : null,
    unavailable_reason: status === "unavailable" ? "source_unavailable" : null,
    snapshot_reload_required: status === "gap",
  };
}

// This adapter only replaces the bridge module in the test server. The DOM
// and Viewer source remain the real bundle under test; the canvas is rendered
// by the repo-owned JS bridge and its derived state is deterministic.
const bridgeModule = String.raw`import { createPixelWorldBevyBridge } from "/software_safe_src/pixel_world_bevy_bridge.js";
export const PIXEL_WORLD_RUNTIME_SOURCE = "qa_browser_fixture_bridge";
export async function createPixelWorldBridge(options) { return createPixelWorldBevyBridge(options); }
const field = (value, snake, camel, fallback = null) => value?.[snake] ?? value?.[camel] ?? fallback;
const pos = (entity) => entity?.pos || { x_cm: 0, y_cm: 0, z_cm: 0 };
export function derivePixelWorldRenderState(input) {
  const snapshot = input?.snapshot || {};
  const model = snapshot.model || {};
  const gameplay = input?.gameplay || snapshot.player_gameplay || {};
  const locations = Object.values(model.locations || {});
  const agents = Object.values(model.agents || {});
  const bounds = snapshot.config?.space || { width_cm: 10000000, depth_cm: 5000000, height_cm: 1000000 };
  const selectedId = input?.selectedId || gameplay.intent_target || agents[0]?.id || null;
  const selectedKind = input?.selectedKind || (selectedId ? "agent" : null);
  const fragments = locations.flatMap((location) => (location.fragment_profile?.blocks?.blocks || []).map((block, index) => ({
    id: "fragment:" + location.id + ":" + index, location_id: location.id,
    pos: { x_cm: pos(location).x_cm + Number(block.origin_cm?.x_cm || 0), y_cm: pos(location).y_cm + Number(block.origin_cm?.z_cm || block.origin_cm?.y_cm || 0), z_cm: pos(location).z_cm + Number(block.origin_cm?.y_cm || 0) },
    dominant_compound: Object.entries(block.compounds?.ppm || {}).sort((a, b) => Number(b[1]) - Number(a[1]))[0]?.[0] || "unknown",
    footprint_cm: Math.max(Number(block.size_cm?.x_cm || 12000), Number(block.size_cm?.z_cm || block.size_cm?.y_cm || 12000)),
  })));
  const normalizedAgents = agents.map((agent, index) => {
    const location = model.locations?.[agent.location_id];
    return { id: agent.id, label: agent.name || agent.id, pos: agent.pos || { x_cm: pos(location).x_cm + 20000 + index * 15000, y_cm: pos(location).y_cm + 10000 + index * 12000, z_cm: pos(location).z_cm }, position_source: agent.pos ? "runtime_agent" : "location_derived" };
  });
  const normalizedLocations = locations.map((location) => ({ id: location.id, label: location.name || location.id, pos: pos(location), marker_role: "logic_anchor", marker_alpha: 0.32 }));
  const firstAction = (gameplay.available_actions || gameplay.availableActions || [])[0] || {};
  const hasReceipt = Boolean(gameplay.recent_feedback || gameplay.recentFeedback || gameplay.last_world_change || gameplay.lastWorldChange);
  const blocker = gameplay.blocker_kind || gameplay.blockerKind || null;
  const blockerDetail = gameplay.blocker_detail || gameplay.blockerDetail || null;
  const objective = gameplay.objective || gameplay.progress_detail || gameplay.progressDetail || "Stabilize the first production line before expanding.";
  return {
    world_bounds: bounds, locations: normalizedLocations, agents: normalizedAgents, fragment_terrain: fragments,
    links: normalizedAgents.filter((agent) => model.locations?.[agents.find((candidate) => candidate.id === agent.id)?.location_id]).map((agent) => ({ id: "link:" + agent.id, kind: "agent_assignment", from: agent.pos, to: pos(model.locations?.[agents.find((candidate) => candidate.id === agent.id)?.location_id]) })),
    selection: selectedKind && selectedId ? { kind: selectedKind, id: selectedId } : null,
    goal_highlight: { title: gameplay.goal_title || gameplay.goalTitle || "Recover sustainable capability", objective },
    blocker_highlight: blocker ? { kind: blocker, label: blocker === "material_shortage" ? "Missing Material" : blocker, detail: blockerDetail } : null,
    // Three read-only fixture explanations exercise blocker/goal/info paths.
    visual_hotspots: [
      { id: "fixture-hotspot-blocker", kind: "blocker", label: "iron input is exhausted", pos: { x_cm: 2900000, y_cm: 3450000, z_cm: 0 } },
      { id: "fixture-hotspot-goal", kind: "goal", label: "stabilize the first production line", pos: { x_cm: 7150000, y_cm: 2200000, z_cm: 0 } },
      { id: "fixture-hotspot-info", kind: "info", label: "read-only world context", pos: { x_cm: 4550000, y_cm: 1200000, z_cm: 0 } },
    ],
    commercial_surface: {
      objective: { title: gameplay.goal_title || gameplay.goalTitle || "Recover sustainable capability", detail: objective, progress_percent: gameplay.progress_percent ?? gameplay.progressPercent ?? 68 },
      next_action: { label: field(firstAction, "label", "label", "Build smelter mk1"), detail: gameplay.intent_summary || gameplay.intentSummary || "Replenish upstream materials, then advance again to confirm the line resumes.", target_agent_id: field(firstAction, "target_agent_id", "targetAgentId", selectedId), execute_kind: field(firstAction, "execute_kind", "executeKind", "gameplay_action") },
      active_agent_id: selectedId,
      player_leverage: { state: gameplay.stage_status || gameplay.stageStatus || "blocked", label: hasReceipt ? "Blocked" : "Waiting for Intent", summary: gameplay.progress_detail || gameplay.progressDetail || "The primary line is blocked by missing material input.", detail: gameplay.next_step_hint || gameplay.nextStepHint || "Replenish upstream materials, then advance again to confirm the line resumes." },
      action_receipt: { present: hasReceipt, state: hasReceipt ? "blocked" : "waiting_for_intent", confidence: hasReceipt ? "world_delta" : "none", title: hasReceipt ? "Action blocked" : "No action receipt yet", summary: hasReceipt ? "Smelter build request reached factory-0; iron shortage blocks construction." : "No receipt", detail: gameplay.last_world_change || gameplay.lastWorldChange || "No player-caused world change has been confirmed yet.", target_agent_id: hasReceipt ? selectedId : null },
      blocker: { label: blocker === "material_shortage" ? "Missing Material" : blocker, detail: gameplay.next_step_hint || gameplay.nextStepHint || blockerDetail },
      world_read: { agents: normalizedAgents.length, routes: normalizedAgents.length, fragments: fragments.length, hotspots: 3 },
    },
  };
}`;

const fakeWebSocket = String.raw`(() => {
  const status = new URLSearchParams(location.search).get("feed_status") || "ready";
  const listeners = new WeakMap();
  const emit = (socket, type, payload = {}) => {
    const callbacks = listeners.get(socket)?.[type] || [];
    for (const callback of callbacks) callback({ type, ...payload });
  };
  const feed = (value) => ({ schema_version: "world_feed/v1", world_id: "fixture-world", reorg_epoch: "0", cursor: value === "empty" ? "" : "1", status: value, events: ["ready", "replay"].includes(value) ? [{ event_seq: "1", kind: "resource_change", summary: "Fixture ambient resource update", detail: "QA fixture event; no player action receipt.", receipt_ref: null }] : [], gap_reason: value === "gap" ? "cursor_gap" : null, unavailable_reason: value === "unavailable" ? "source_unavailable" : null, snapshot_reload_required: value === "gap" });
  class FixtureWebSocket {
    static OPEN = 1;
    static CONNECTING = 0;
    static CLOSING = 2;
    static CLOSED = 3;
    constructor(url) { this.url = url; this.readyState = FixtureWebSocket.CONNECTING; listeners.set(this, {}); queueMicrotask(() => { this.readyState = FixtureWebSocket.OPEN; emit(this, "open"); }); }
    addEventListener(type, callback) { const table = listeners.get(this); table[type] ||= []; table[type].push(callback); }
    removeEventListener(type, callback) { const list = listeners.get(this)?.[type] || []; const index = list.indexOf(callback); if (index >= 0) list.splice(index, 1); }
    send(raw) { const message = JSON.parse(raw); if (message.type === "hello") queueMicrotask(() => emit(this, "message", { data: JSON.stringify({ type: "hello_ack", server: "qa-fixture", world_id: "fixture-world", control_profile: "fixture" }) })); if (message.type === "request_world_feed") queueMicrotask(() => emit(this, "message", { data: JSON.stringify({ type: "world_feed", feed: feed(status) }) })); }
    close() { this.readyState = FixtureWebSocket.CLOSED; emit(this, "close"); }
  }
  window.WebSocket = FixtureWebSocket;
})();`;

// The deterministic repo-owned JS bridge below paints with a 2D context,
// while the production host performs a WebGL2 surface probe first. A real
// canvas cannot claim both context types, so this fixture satisfies only the
// host probe and leaves the bridge's 2D context untouched. This keeps the
// browser coverage focused on the real Viewer DOM/shell without implying a
// live WebGPU/WebGL renderer or playability.
const fixtureCanvasCompatibility = String.raw`(() => {
  const nativeGetContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function getContext(type, ...args) {
    if (type === "webgl2") return { __qaWebgl2ProbeOnly: true };
    return nativeGetContext.call(this, type, ...args);
  };
  document.body.setAttribute("data-viewer-visual-fixture", "shell_selected_blocker");
})();`;

function serveFile(request, response) {
  const requestUrl = new URL(request.url || "/", "http://127.0.0.1");
  const pathname = requestUrl.pathname;
  if (pathname === "/pixel-world-bridge/pixel_world_bridge.js") {
    response.writeHead(200, { "Content-Type": "text/javascript; charset=utf-8", "Cache-Control": "no-store" });
    response.end(bridgeModule);
    return;
  }
  const rawPath = decodeURIComponent(pathname === "/" ? "/viewer.html" : pathname);
  const normalized = normalize(rawPath).replace(/^(\.\.(\/|\\|$))+/, "");
  const filePath = pathname === "/viewer.js" && statSafe(bundlePath)
    ? bundlePath
    : resolve(viewerRoot, `.${normalized}`);
  if (!relative(viewerRoot, filePath) || relative(viewerRoot, filePath).startsWith("..")) {
    response.writeHead(403); response.end("forbidden"); return;
  }
  try {
    let body = readFileSync(filePath, "utf8");
    if (pathname === "/viewer.html" && requestUrl.searchParams.get("browser_fixture") === "1") {
      body = body.replace('<script type="module" src="./viewer.js"></script>', `<script>${fixtureCanvasCompatibility}</script><script>${fakeWebSocket}</script><script type="module" src="./viewer.js"></script>`);
    }
    response.writeHead(200, { "Content-Type": contentType(filePath), "Cache-Control": "no-store" });
    response.end(body);
  } catch {
    response.writeHead(404); response.end("not found");
  }
}

function statSafe(path) {
  try { return statSync(path).isFile(); } catch { return false; }
}

function fixtureUrl(port, { status = "ready", locale = "en" } = {}) {
  const params = new URLSearchParams({
    browser_fixture: "1", test_api: "1", connect: "1", hosted_bootstrap: "0", ws: "ws://qa-fixture",
    locale, feed_status: status, viewer_visual_fixture: "shell_selected_blocker", pixel_world_visual_fixture: "selected_blocker", t: Date.now().toString(),
  });
  return `http://127.0.0.1:${port}/viewer.html?${params}`;
}

const probe = String.raw`(() => {
  const rect = (element) => { if (!element) return null; const r = element.getBoundingClientRect(); return { x: Math.round(r.x), y: Math.round(r.y), right: Math.round(r.right), bottom: Math.round(r.bottom), width: Math.round(r.width), height: Math.round(r.height) }; };
  const text = (selector, root = document) => root.querySelector(selector)?.textContent.trim() || null;
  const intersects = (a, b) => Boolean(a && b && a.right > b.x && a.x < b.right && a.bottom > b.y && a.y < b.bottom);
  const stackAt = (box) => {
    if (!box) return [];
    const x = Math.max(1, Math.min(innerWidth - 1, box.x + Math.max(1, box.width / 2)));
    const y = Math.max(1, Math.min(innerHeight - 1, box.y + Math.max(1, box.height / 2)));
    return document.elementsFromPoint(x, y).slice(0, 6).map((node) => ({ tag: node.tagName, id: node.id || null, className: node.className || null, overlay: node.getAttribute?.('data-viewer-overlay') || null, text: String(node.textContent || '').trim().slice(0, 160) }));
  };
  const feed = document.querySelector('[data-viewer-overlay="feed"]');
  const command = document.querySelector('[data-viewer-overlay="next-move"]');
  const primary = command?.querySelector('[data-shell-region="next-move-primary"]') || command;
  const supporting = command?.querySelector('[data-shell-region="supporting-context"]');
  const receipt = document.querySelector('.pixel-world-action-receipt');
  const selection = document.querySelector('.pixel-world-canvas__selection');
  const readout = document.querySelector('.pixel-world-readout');
  const hotspotNodes = [...document.querySelectorAll('[data-hotspot-kind]')];
  return JSON.stringify({
    runtime: window.__AW_TEST__?.getState?.() || null,
    feed: feed ? { status: feed.dataset.worldFeedStatus || null, open: feed.open, text: feed.textContent.trim(), rect: rect(feed), statusRow: text('.world-feed__status-row', feed), scrollTop: feed.scrollTop, scrollHeight: feed.scrollHeight, clientHeight: feed.clientHeight } : null,
    readout: readout ? { text: readout.textContent.trim(), classes: [...readout.querySelectorAll('.badge')].map((node) => ({ text: node.textContent.trim(), className: node.className })) } : null,
    selection: selection ? { text: selection.textContent.trim(), rect: rect(selection) } : null,
    primary: primary ? { text: primary.textContent.trim(), rect: rect(primary), visible: getComputedStyle(primary).display !== 'none' && getComputedStyle(primary).visibility !== 'hidden' } : null,
    supporting: supporting ? { text: supporting.textContent.trim(), rect: rect(supporting), visible: getComputedStyle(supporting).display !== 'none' } : null,
    receipt: receipt ? { present: receipt.dataset.receiptPresent, state: receipt.dataset.receiptState, confidence: receipt.dataset.receiptConfidence, text: receipt.textContent.trim(), rect: rect(receipt), visible: getComputedStyle(receipt).display !== 'none', scrollTop: receipt.scrollTop, scrollHeight: receipt.scrollHeight, clientHeight: receipt.clientHeight } : null,
    tooltip: document.querySelector('[data-hotspot-tooltip]') ? { text: text('[data-hotspot-tooltip]'), rect: rect(document.querySelector('[data-hotspot-tooltip]')) } : null,
    hotspots: hotspotNodes.map((node) => ({ tag: node.tagName, kind: node.dataset.hotspotKind, label: node.getAttribute('aria-label'), title: node.getAttribute('title'), role: node.getAttribute('role'), tabIndex: node.tabIndex, text: node.textContent.trim(), rect: rect(node) })),
    viewport: { width: innerWidth, height: innerHeight, clientWidth: document.documentElement.clientWidth, scrollWidth: document.documentElement.scrollWidth, overflowX: Math.max(0, document.documentElement.scrollWidth - document.documentElement.clientWidth) },
    hitTest: { selection: stackAt(selection ? rect(selection) : null), feed: stackAt(feed ? rect(feed) : null) },
    bodyText: document.body.innerText,
  });
})()`;

const waitReady = String.raw`(async () => { const end = Date.now() + 15000; while (Date.now() < end) { const state = window.__AW_TEST__?.getState?.() || {}; if (state.pixelWorldRuntimeStatus === "ready" && document.querySelector('[data-viewer-overlay="feed"]')) return JSON.stringify(state); await new Promise((resolve) => setTimeout(resolve, 100)); } throw new Error(JSON.stringify(window.__AW_TEST__?.getState?.() || null)); })()`;

function visible(textValue) { return textValue && textValue.trim().length > 0; }

function intersects(a, b) {
  return Boolean(a && b && a.right > b.x && a.x < b.right && a.bottom > b.y && a.y < b.bottom);
}

async function waitForFeedStatus(status) {
  const end = Date.now() + 10_000;
  while (Date.now() < end) {
    const result = await evalJson(probe);
    if (result.feed?.status === status && result.runtime?.connectionStatus === "connected") return result;
    await browserRaw(["wait", "100"]);
  }
  return evalJson(probe);
}

async function openFixture(port, options) {
  await browserJson(["open", fixtureUrl(port, options)], { timeout: 45_000 });
  await browserJson(["set", "viewport", String(options.width || 390), String(options.height || 844)]);
  await evalJson(waitReady);
}

async function clickVisible(selector) {
  await browserJson(["scrollintoview", selector]);
  await browserJson(["click", selector]);
  return { selector };
}

async function waitShort() { await browserRaw(["wait", "180"]); }

function assertBase(label, result, expectedStatus) {
  assert(result.runtime?.pixelWorldRuntimeStatus === "ready", `${label}: runtime not ready`, result);
  assert(result.runtime?.renderMode === "viewer" || result.runtime?.renderMode === "software_safe", `${label}: unexpected render mode`, result.runtime);
  assert(result.feed?.status === expectedStatus, `${label}: expected feed status ${expectedStatus}`, result.feed);
  assert(result.selection?.rect?.width > 0 && visible(result.selection?.text), `${label}: selected object confirmation missing`, result.selection);
  assert(!/^.*agent-[a-z0-9_-]+.*$/i.test(result.selection?.text || ""), `${label}: raw selected agent id leaked`, result.selection);
  assert(result.primary?.visible && result.primary?.rect?.width > 0, `${label}: Next Move is not visible`, result.primary);
  assert(result.receipt?.visible && result.receipt?.rect?.width > 0, `${label}: Action Receipt is not visible`, result.receipt);
  assert(result.viewport.overflowX <= 2, `${label}: horizontal overflow ${result.viewport.overflowX}px`, result.viewport);
  if (result.feed && !result.feed.open) {
    assert(!intersects(result.feed.rect, result.selection.rect), `${label}: collapsed Feed overlaps selected-object confirmation`, {
      feed: result.feed.rect,
      selection: result.selection.rect,
      topHit: result.hitTest?.selection?.[0],
    });
  }
}

async function runStatusMatrix(port) {
  for (const status of statuses) {
    await openFixture(port, { status, locale: "en", width: 390, height: 844 });
    const result = await waitForFeedStatus(status);
    assertBase(`A2/${status}`, result, status);
    const readout = result.readout?.text || "";
    const expected = { ready: "LIVE", replay: "REPLAY", empty: "NO EVENTS", gap: "GAP", unavailable: "UNAVAILABLE" }[status];
    assert(readout.includes(expected), `A2/${status}: readout missing ${expected}`, result.readout);
    if (status !== "ready") assert(!readout.includes("LIVE"), `A2/${status}: context incorrectly labeled LIVE`, result.readout);
    const screenshot = join(outDir, `a2-${status}-mobile.png`);
    await browserRaw(["screenshot", screenshot], { timeout: 20_000 });
    summary.feedStatuses[status] = { screenshot, result };
  }
}

async function runInteractions(port) {
  await openFixture(port, { status: "ready", locale: "en", width: 390, height: 844 });
  const before = await evalJson(probe);
  await clickVisible("button[data-pixel-world-agent-marker='true']");
  await waitShort();
  const afterAgent = await evalJson(probe);
  assert(afterAgent.selection?.rect?.width > 0 && visible(afterAgent.selection?.text), "A1/agent: selection confirmation missing after visible click", afterAgent.selection);
  await clickVisible("button[data-pixel-world-location-marker]");
  await waitShort();
  const afterLocation = await evalJson(probe);
  assert(afterLocation.selection?.rect?.width > 0 && visible(afterLocation.selection?.text), "A1/location: selection confirmation missing after visible click", afterLocation.selection);

  const feedBefore = afterLocation;
  await clickVisible('[data-viewer-overlay="feed"] > summary');
  await waitShort();
  const feedOpen = await evalJson(probe);
  assert(feedOpen.feed?.open === true, "A3: Feed did not open through visible summary control", feedOpen.feed);
  assert(feedOpen.primary?.visible && feedOpen.receipt?.visible, "A3: Next Move or Action Receipt hidden with Feed open", { primary: feedOpen.primary, receipt: feedOpen.receipt });
  assert(!intersects(feedOpen.feed.rect, feedOpen.primary.rect), "A3: Feed overlaps Next Move", { feed: feedOpen.feed.rect, primary: feedOpen.primary.rect });
  assert(!intersects(feedOpen.feed.rect, feedOpen.receipt.rect), "A3: Feed overlaps Action Receipt", { feed: feedOpen.feed.rect, receipt: feedOpen.receipt.rect });
  await browserRaw(["scroll", "down", "320", "--selector", '[data-viewer-overlay="feed"]']);
  await waitShort();
  const feedScrolled = await evalJson(probe);
  assert(feedScrolled.feed?.scrollTop > 0 || feedScrolled.feed?.scrollHeight <= feedScrolled.feed?.clientHeight, "A3: expanded Feed could not scroll its long detail surface", feedScrolled.feed);
  await browserRaw(["scroll", "down", "320", "--selector", '[data-viewer-overlay="receipt"]']);
  await waitShort();
  const receiptScrolled = await evalJson(probe);
  assert(receiptScrolled.receipt?.scrollTop > 0 || receiptScrolled.receipt?.scrollHeight <= receiptScrolled.receipt?.clientHeight, "A3: Action Receipt could not scroll its detail surface", receiptScrolled.receipt);
  const feedScreenshot = join(outDir, "a3-feed-open-mobile.png");
  await browserRaw(["screenshot", feedScreenshot], { timeout: 20_000 });
  await clickVisible('[data-viewer-overlay="feed"] > summary');

  const hotspots = feedOpen.hotspots || [];
  assert(hotspots.length >= 2, "A4: fixture did not expose blocker/goal hotspots", hotspots);
  for (const hotspot of hotspots) {
    assert(hotspot.tabIndex >= 0 || hotspot.role === "button" || hotspot.tag === "BUTTON" || visible(hotspot.label), `A4: hotspot ${hotspot.kind} has no keyboard/accessibility affordance`, hotspot);
  }
  const beforeInteractionState = before.runtime || {};
  await clickVisible('[data-hotspot-kind="goal"]');
  await waitShort();
  const afterPointerHotspot = await evalJson(probe);
  assert(afterPointerHotspot.tooltip?.text?.includes("Goal: stabilize the first production line"), "A4: visible hotspot click did not expose a goal explanation", afterPointerHotspot.tooltip);
  await clickVisible('.pixel-world-canvas__hotspot-tooltip-close');
  await waitShort();
  const afterPointerClose = await evalJson(probe);
  assert(!afterPointerClose.tooltip, "A4: visible tooltip close control did not dismiss the explanation", afterPointerClose);
  await evalJson(`(() => { const node = document.querySelector('[data-hotspot-kind="blocker"]'); if (!node) throw new Error("missing blocker hotspot"); node.scrollIntoView({ block: "center" }); node.focus?.(); return JSON.stringify({ active: document.activeElement === node, tag: node.tagName }); })()`);
  await browserRaw(["press", "Enter"]);
  await waitShort();
  const afterHotspot = await evalJson(probe);
  const explanation = afterHotspot.bodyText.includes("Blocker: iron input is exhausted") || afterHotspot.bodyText.includes("阻塞") || afterHotspot.bodyText.includes("iron input");
  assert(explanation, "A4: blocker hotspot did not expose a readable explanation after keyboard activation", { hotspots: afterHotspot.hotspots, bodyText: afterHotspot.bodyText.slice(-2500) });
  const afterHotspotState = afterHotspot.runtime || {};
  assert(afterHotspotState.logicalTime === beforeInteractionState.logicalTime && afterHotspotState.eventSeq === beforeInteractionState.eventSeq, "A4: hotspot inspection changed runtime time/event state", { before: beforeInteractionState, after: afterHotspotState });
  const hotspotScreenshot = join(outDir, "a4-hotspot-keyboard-mobile.png");
  await browserRaw(["screenshot", hotspotScreenshot], { timeout: 20_000 });
  await browserRaw(["press", "Escape"]);
  await waitShort();
  const afterEscape = await evalJson(probe);
  assert(!afterEscape.tooltip, "A4: Escape did not dismiss the keyboard-opened explanation", afterEscape);
  summary.interactions = { before: feedBefore, afterAgent, afterLocation, feedOpen: { ...feedOpen, screenshot: feedScreenshot, feedScrolled: feedScrolled.feed, receiptScrolled: receiptScrolled.receipt }, hotspot: { ...afterHotspot, screenshot: hotspotScreenshot }, pointerHotspot: afterPointerHotspot, pointerClose: afterPointerClose, afterEscape };
}

async function runViewportMatrix(port) {
  for (const viewport of viewports) {
    await openFixture(port, { status: "ready", locale: "en", width: viewport.width, height: viewport.height });
    const result = await waitForFeedStatus("ready");
    assertBase(`A1/A3/A5/${viewport.name}`, result, "ready");
    assert(result.bodyText.includes("Replenish upstream materials") || result.bodyText.includes("Missing Material"), `A5/${viewport.name}: blocking/next-action copy not present`, result.bodyText.slice(-2500));
    const screenshot = join(outDir, `a1-a3-a5-${viewport.name}.png`);
    await browserRaw(["screenshot", screenshot], { timeout: 20_000 });
    summary.viewports[viewport.name] = { viewport, screenshot, result };
  }
  await openFixture(port, { status: "ready", locale: "zh-CN", width: 390, height: 844 });
  const cjk = await waitForFeedStatus("ready");
  assertBase("A5/CJK/mobile", cjk, "ready");
  assert(cjk.bodyText.includes("目标") || cjk.bodyText.includes("玩家杠杆") || cjk.bodyText.includes("行动回执"), "A5/CJK: localized hierarchy labels missing", cjk.bodyText.slice(-2500));
  const screenshot = join(outDir, "a5-cjk-mobile.png");
  await browserRaw(["screenshot", screenshot], { timeout: 20_000 });
  summary.viewports.cjk = { viewport: { width: 390, height: 844 }, screenshot, result: cjk };

  await openFixture(port, { status: "ready", locale: "en", width: 390, height: 844 });
  const resized = {};
  for (const viewport of [
    { name: "resize-tablet", width: 768, height: 1024 },
    { name: "resize-desktop", width: 1440, height: 1000 },
    { name: "resize-mobile", width: 390, height: 844 },
  ]) {
    await browserJson(["set", "viewport", String(viewport.width), String(viewport.height)]);
    await waitShort();
    const result = await waitForFeedStatus("ready");
    assertBase(`A1/resize/${viewport.name}`, result, "ready");
    const screenshot = join(outDir, `a1-resize-${viewport.name}.png`);
    await browserRaw(["screenshot", screenshot], { timeout: 20_000 });
    resized[viewport.name] = { viewport, screenshot, result };
  }
  summary.viewports.resize = resized;
}

async function main() {
  if (!skipBuild) {
    const build = spawnSync("npm", ["--prefix", "crates/oasis7_viewer", "run", "build:viewer:bundle"], { cwd: repoRoot, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
    writeFileSync(join(outDir, "bundle-build.log"), `${build.stdout || ""}${build.stderr || ""}`, "utf8");
    assert(build.status === 0, "Viewer bundle build failed; see bundle-build.log", { status: build.status, signal: build.signal });
  }
  assert(statSafe(bundlePath), `missing ${bundlePath}; build:viewer:bundle did not produce the test bundle`);
  assert(spawnSync(browserBin, ["--version"], { stdio: "ignore" }).status === 0, `missing browser automation command: ${browserBin}`);
  const server = createServer(serveFile);
  await new Promise((resolveServer) => server.listen(0, "127.0.0.1", resolveServer));
  const address = server.address();
  const port = address.port;
  try {
    closeBrowser();
    await runStatusMatrix(port);
    await runViewportMatrix(port);
    await runInteractions(port);
    const consoleOutput = await browserRaw(["console"]);
    writeFileSync(join(outDir, "browser-console.log"), consoleOutput, "utf8");
    assert(!/\[(?:error|pageerror)\]|\b(?:fatal|CONTEXT_LOST_WEBGL)\b/i.test(consoleOutput), "browser console contains runtime errors", { consoleOutput });
    summary.console = { path: join(outDir, "browser-console.log"), errors: 0 };
    summary.status = "passed";
  } catch (error) {
    summary.status = "failed";
    summary.failure = String(error?.stack || error);
    try {
      writeFileSync(join(outDir, "failure-state.json"), JSON.stringify(await evalJson(probe), null, 2), "utf8");
      await browserRaw(["screenshot", join(outDir, "failure.png")], { timeout: 20_000 });
      writeFileSync(join(outDir, "browser-console.log"), await browserRaw(["console"]), "utf8");
    } catch (diagnosticError) {
      summary.failureDiagnostics = String(diagnosticError?.stack || diagnosticError);
    }
    throw error;
  } finally {
    summary.completedAt = new Date().toISOString();
    writeFileSync(join(outDir, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`, "utf8");
    closeBrowser();
    await new Promise((resolveServer) => server.close(resolveServer));
  }
  console.log(`player visual feedback browser smoke passed: ${outDir}`);
}

main().catch((error) => {
  console.error(error?.stack || error);
  process.exitCode = 1;
});
