const TEST_API_GLOBAL_NAME = "__AW_TEST__";
const RENDER_META_GLOBAL_NAME = "__AW_VIEWER_RENDER_META__";
const SOFTWARE_SAFE_RENDER_MODE = "software_safe";
const VIEWER_AUTH_BOOTSTRAP_OBJECT = "__AGENT_WORLD_VIEWER_AUTH_ENV";
const VIEWER_AUTH_SIGNATURE_PREFIX = "awviewauth:v1:";
const DEFAULT_WS_ADDR = "ws://127.0.0.1:5011";
const MAX_EVENTS = 24;
const SOFTWARE_RENDERER_MARKERS = [
  "swiftshader",
  "llvmpipe",
  "software rasterizer",
  "basic render driver",
  "softpipe",
  "lavapipe",
];
const ED25519_PKCS8_PREFIX = new Uint8Array([
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06,
  0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
]);
const textEncoder = new TextEncoder();

const state = {
  connectionStatus: "connecting",
  logicalTime: 0,
  eventSeq: 0,
  tick: 0,
  selectedKind: null,
  selectedId: null,
  errorCount: 0,
  lastError: null,
  eventCount: 0,
  traceCount: 0,
  cameraMode: "software_safe",
  cameraRadius: 0,
  cameraOrthoScale: 0,
  renderMode: SOFTWARE_SAFE_RENDER_MODE,
  rendererClass: "none",
  softwareSafeReason: null,
  renderer: null,
  vendor: null,
  webglVersion: null,
  controlProfile: "playback",
  worldId: null,
  server: null,
  wsUrl: null,
  lastControlFeedback: null,
  lastPromptFeedback: null,
  lastChatFeedback: null,
  snapshot: null,
  metrics: null,
  recentEvents: [],
  chatHistory: [],
  selectedObject: null,
  auth: {
    available: false,
    playerId: null,
    publicKey: null,
    privateKey: null,
    error: null,
  },
  promptDraft: {
    agentId: null,
    currentVersion: 0,
    rollbackTargetVersion: 0,
    updatedBy: "",
    updatedAtTick: 0,
    systemPrompt: "",
    shortTermGoal: "",
    longTermGoal: "",
    dirty: false,
  },
  chatDraft: {
    agentId: null,
    message: "",
    dirty: false,
  },
};

let socket = null;
let reconnectTimer = null;
let requestId = 0;
let authNonceCounter = 0;
let selectedSearch = "";
let semanticSendLoop = null;
const pendingControlFeedback = new Map();
const pendingSemanticCommands = [];
const authKeyCache = new Map();

const elements = {};

function getSearchParams() {
  return new URLSearchParams(window.location.search || "");
}

function normalizeWsAddr(raw) {
  const value = String(raw || "").trim();
  if (!value) return DEFAULT_WS_ADDR;
  if (value.startsWith("ws://") || value.startsWith("wss://")) return value;
  if (value.startsWith("http://")) return `ws://${value.slice("http://".length)}`;
  if (value.startsWith("https://")) return `wss://${value.slice("https://".length)}`;
  return `ws://${value}`;
}

function clone(value) {
  return value == null ? value : JSON.parse(JSON.stringify(value));
}

function detectRendererMeta() {
  const params = getSearchParams();
  const reasonFromQuery = params.get("software_safe_reason");
  const meta = {
    renderMode: SOFTWARE_SAFE_RENDER_MODE,
    rendererClass: "none",
    softwareSafeReason: reasonFromQuery || "forced_fallback",
    renderer: null,
    vendor: null,
    webglVersion: null,
  };

  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl") || canvas.getContext("experimental-webgl");
    if (!gl) {
      meta.rendererClass = "none";
      meta.softwareSafeReason = reasonFromQuery || "webgl_unavailable";
      return meta;
    }
    meta.webglVersion = gl.getParameter(gl.VERSION) || null;
    const debugInfo = gl.getExtension("WEBGL_debug_renderer_info");
    if (debugInfo) {
      meta.renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) || null;
      meta.vendor = gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) || null;
    }
    const rendererText = String(meta.renderer || "").toLowerCase();
    if (SOFTWARE_RENDERER_MARKERS.some((marker) => rendererText.includes(marker))) {
      meta.rendererClass = "software";
      meta.softwareSafeReason = reasonFromQuery || "software_renderer_detected";
    } else {
      meta.rendererClass = "unknown";
      meta.softwareSafeReason = reasonFromQuery || "forced_query";
    }
  } catch (error) {
    meta.rendererClass = "none";
    meta.softwareSafeReason = reasonFromQuery || "webgl_probe_failed";
    meta.renderer = String(error);
  }
  return meta;
}

function resolveAuthBootstrap() {
  const raw = window[VIEWER_AUTH_BOOTSTRAP_OBJECT];
  if (!raw || typeof raw !== "object") {
    return {
      available: false,
      playerId: null,
      publicKey: null,
      privateKey: null,
      error: "viewer auth bootstrap is unavailable",
    };
  }
  const playerId = String(raw.AGENT_WORLD_VIEWER_PLAYER_ID || "").trim();
  const publicKey = String(raw.AGENT_WORLD_VIEWER_AUTH_PUBLIC_KEY || "").trim().toLowerCase();
  const privateKey = String(raw.AGENT_WORLD_VIEWER_AUTH_PRIVATE_KEY || "").trim().toLowerCase();
  if (!playerId || !publicKey || !privateKey) {
    return {
      available: false,
      playerId: playerId || null,
      publicKey: publicKey || null,
      privateKey: privateKey || null,
      error: "viewer auth bootstrap is incomplete",
    };
  }
  return {
    available: true,
    playerId,
    publicKey,
    privateKey,
    error: null,
  };
}

function nextRequestId() {
  requestId += 1;
  return requestId;
}

function nextAuthNonce() {
  authNonceCounter += 1;
  return Date.now() + authNonceCounter;
}

function snapshotControlFeedback(feedback) {
  if (!feedback) return null;
  return {
    id: feedback.id,
    action: feedback.action,
    accepted: feedback.accepted,
    stage: feedback.stage,
    reason: feedback.reason,
    hint: feedback.hint,
    effect: feedback.effect,
    deltaLogicalTime: feedback.deltaLogicalTime || 0,
    deltaEventSeq: feedback.deltaEventSeq || 0,
    deltaTraceCount: feedback.deltaTraceCount || 0,
  };
}

function snapshotSemanticFeedback(feedback) {
  if (!feedback) return null;
  return {
    id: feedback.id,
    kind: feedback.kind,
    action: feedback.action,
    agentId: feedback.agentId || null,
    accepted: feedback.accepted,
    stage: feedback.stage,
    ok: feedback.ok,
    reason: feedback.reason || null,
    effect: feedback.effect || null,
    response: clone(feedback.response) || null,
  };
}

function getState() {
  return {
    connectionStatus: state.connectionStatus,
    logicalTime: state.logicalTime,
    eventSeq: state.eventSeq,
    tick: state.tick,
    selectedKind: state.selectedKind,
    selectedId: state.selectedId,
    errorCount: state.errorCount,
    lastError: state.lastError,
    eventCount: state.eventCount,
    traceCount: state.traceCount,
    cameraMode: state.cameraMode,
    cameraRadius: state.cameraRadius,
    cameraOrthoScale: state.cameraOrthoScale,
    lastControlFeedback: snapshotControlFeedback(state.lastControlFeedback),
    lastPromptFeedback: snapshotSemanticFeedback(state.lastPromptFeedback),
    lastChatFeedback: snapshotSemanticFeedback(state.lastChatFeedback),
    renderMode: state.renderMode,
    rendererClass: state.rendererClass,
    softwareSafeReason: state.softwareSafeReason,
    renderer: state.renderer,
    vendor: state.vendor,
    webglVersion: state.webglVersion,
    controlProfile: state.controlProfile,
    worldId: state.worldId,
    server: state.server,
    wsUrl: state.wsUrl,
    authReady: state.auth.available,
    authPlayerId: state.auth.playerId,
    authPublicKey: state.auth.publicKey,
    authError: state.auth.error,
    selectedPromptVersion: state.promptDraft.currentVersion || 0,
    promptRollbackTargetVersion: state.promptDraft.rollbackTargetVersion || 0,
    chatHistoryCount: state.chatHistory.length,
    chatHistory: clone(state.chatHistory),
  };
}

function reportFatalError(message, source = "runtime") {
  const text = `${source}: ${String(message || "unknown runtime error")}`.trim();
  if (state.lastError !== text) {
    state.errorCount += 1;
  }
  state.connectionStatus = "error";
  state.lastError = text;
  render();
}

function parseSelectionPayload(payload) {
  if (payload == null) {
    return null;
  }
  if (typeof payload === "string") {
    const trimmed = payload.trim();
    if (!trimmed) return null;
    const parts = trimmed.split(":");
    if (parts.length >= 2) {
      return { kind: parts[0], id: parts.slice(1).join(":") };
    }
    return { kind: "agent", id: trimmed };
  }
  if (typeof payload === "object") {
    const kind = payload.kind || payload.targetKind || payload.type;
    const id = payload.id || payload.targetId || payload.value;
    if (!kind || !id) return null;
    return { kind: String(kind), id: String(id) };
  }
  return null;
}

function entityCollections() {
  const model = state.snapshot?.model || {};
  return {
    agents: Object.values(model.agents || {}),
    locations: Object.values(model.locations || {}),
  };
}

function selectedAgentId() {
  return state.selectedKind === "agent" ? state.selectedId : null;
}

function selectedAgentPromptProfile() {
  const agentId = selectedAgentId();
  if (!agentId) {
    return null;
  }
  return state.snapshot?.model?.agent_prompt_profiles?.[agentId] || {
    agent_id: agentId,
    version: 0,
    updated_at_tick: 0,
    updated_by: "",
    system_prompt_override: null,
    short_term_goal_override: null,
    long_term_goal_override: null,
  };
}

function selectedAgentBindingInfo() {
  const agentId = selectedAgentId();
  if (!agentId) {
    return null;
  }
  return {
    playerId: state.snapshot?.model?.agent_player_bindings?.[agentId] || null,
    publicKey: state.snapshot?.model?.agent_player_public_key_bindings?.[agentId] || null,
  };
}

function syncAgentInteractionDrafts(force = false) {
  const agentId = selectedAgentId();
  const profile = selectedAgentPromptProfile();
  if (force || state.promptDraft.agentId !== agentId || (!state.promptDraft.dirty && agentId)) {
    const currentVersion = Number(profile?.version || 0);
    state.promptDraft = {
      agentId,
      currentVersion,
      rollbackTargetVersion: Math.max(0, currentVersion - 1),
      updatedBy: String(profile?.updated_by || ""),
      updatedAtTick: Number(profile?.updated_at_tick || 0),
      systemPrompt: String(profile?.system_prompt_override || ""),
      shortTermGoal: String(profile?.short_term_goal_override || ""),
      longTermGoal: String(profile?.long_term_goal_override || ""),
      dirty: false,
    };
  }
  if (force || state.chatDraft.agentId !== agentId) {
    state.chatDraft = {
      agentId,
      message: agentId === state.chatDraft.agentId ? state.chatDraft.message : "",
      dirty: false,
    };
  }
}

function applySelection(selection) {
  if (!selection) return null;
  const kind = String(selection.kind || "").toLowerCase();
  const id = String(selection.id || "");
  const { agents, locations } = entityCollections();
  let object = null;
  if (kind === "agent") {
    object = agents.find((entry) => entry.id === id) || null;
  } else if (kind === "location") {
    object = locations.find((entry) => entry.id === id) || null;
  }
  if (!object) {
    return null;
  }
  state.selectedKind = kind;
  state.selectedId = id;
  state.selectedObject = object;
  syncAgentInteractionDrafts(true);
  render();
  return { kind, id };
}

function select(payload) {
  const parsed = parseSelectionPayload(payload);
  if (!parsed) {
    return { ok: false, reason: "invalid selection payload" };
  }
  const applied = applySelection(parsed);
  if (!applied) {
    return { ok: false, reason: `target not found: ${parsed.kind}:${parsed.id}` };
  }
  return { ok: true, ...applied };
}

function focus(payload) {
  return select(payload);
}

function parseStepCount(payload) {
  if (payload == null) return 1;
  if (typeof payload === "number" && Number.isFinite(payload) && payload >= 1) {
    return Math.floor(payload);
  }
  if (typeof payload === "string") {
    const trimmed = payload.trim();
    if (!trimmed || trimmed === "step") return 1;
    const numeric = Number(trimmed);
    if (Number.isFinite(numeric) && numeric >= 1) {
      return Math.floor(numeric);
    }
    const matched = trimmed.match(/step\s*[:=]\s*(\d+)/i);
    if (matched) {
      return Number(matched[1]);
    }
    return null;
  }
  if (typeof payload === "object") {
    const numeric = Number(payload.count);
    if (Number.isFinite(numeric) && numeric >= 1) {
      return Math.floor(numeric);
    }
  }
  return null;
}

function controlActions() {
  return [
    {
      action: "play",
      description: "Start continuous world advancement",
      descriptionZh: "开始连续推进世界",
      examplePayload: null,
    },
    {
      action: "pause",
      description: "Pause continuous advancement",
      descriptionZh: "暂停连续推进",
      examplePayload: null,
    },
    {
      action: "step",
      description: "Advance fixed steps (payload.count)",
      descriptionZh: "推进固定步数（payload.count）",
      examplePayload: { count: 5 },
    },
  ];
}

function describeControls() {
  return {
    controls: controlActions(),
    semanticActions: [
      {
        action: "sendAgentChat",
        description: "Send a player-authenticated chat message to an agent",
      },
      {
        action: "sendPromptControl",
        description: "Preview, apply, or rollback prompt overrides for an agent",
      },
    ],
    usage: "Use fillControlExample(action), sendControl(action), sendAgentChat(agentId, message), sendPromptControl(mode, payload).",
  };
}

function fillControlExample(action) {
  const normalized = String(action || "").trim().toLowerCase();
  return controlActions().find((entry) => entry.action === normalized)?.examplePayload ?? null;
}

function sendJson(payload) {
  if (!socket || socket.readyState !== WebSocket.OPEN) {
    throw new Error("viewer websocket is not connected");
  }
  socket.send(JSON.stringify(payload));
}

function sendViewerControl(action, payload) {
  const normalized = String(action || "").trim().toLowerCase();
  const currentRequestId = nextRequestId();
  const feedback = {
    id: currentRequestId,
    action: normalized,
    accepted: false,
    stage: "rejected",
    reason: null,
    hint: null,
    effect: null,
    baselineLogicalTime: state.logicalTime,
    baselineEventSeq: state.eventSeq,
    deltaLogicalTime: 0,
    deltaEventSeq: 0,
    deltaTraceCount: 0,
    requestId: currentRequestId,
  };

  let mode = null;
  if (normalized === "play") {
    mode = { mode: "play" };
  } else if (normalized === "pause") {
    mode = { mode: "pause" };
  } else if (normalized === "step") {
    const count = parseStepCount(payload);
    if (!count) {
      feedback.reason = "step requires numeric payload.count >= 1";
      feedback.effect = "request rejected before send";
      state.lastControlFeedback = feedback;
      render();
      return snapshotControlFeedback(feedback);
    }
    mode = { mode: "step", count };
  } else {
    feedback.reason = `unsupported action: ${normalized}`;
    feedback.effect = "request rejected before send";
    state.lastControlFeedback = feedback;
    render();
    return snapshotControlFeedback(feedback);
  }

  try {
    if (state.controlProfile === "live") {
      sendJson({ type: "live_control", mode, request_id: currentRequestId });
    } else if (state.controlProfile === "playback") {
      sendJson({ type: "playback_control", mode, request_id: currentRequestId });
    } else {
      sendJson({ type: "control", mode, request_id: currentRequestId });
    }
    feedback.accepted = true;
    feedback.stage = "queued";
    feedback.effect = "queued, check getState().lastControlFeedback for world delta";
    pendingControlFeedback.set(currentRequestId, feedback);
    state.lastControlFeedback = feedback;
    render();
    return snapshotControlFeedback(feedback);
  } catch (error) {
    feedback.reason = String(error);
    feedback.effect = "request send failed";
    state.lastControlFeedback = feedback;
    render();
    return snapshotControlFeedback(feedback);
  }
}

function sendControl(action, payload = null) {
  return sendViewerControl(action, payload);
}

function runSteps(payload) {
  const count = parseStepCount(payload);
  if (!count) {
    return { ok: false, reason: "payload must be non-empty step string or count" };
  }
  const feedback = sendControl("step", { count });
  return { ok: Boolean(feedback?.accepted), count, feedback };
}

function setMode() {
  return {
    ok: false,
    reason: "software_safe viewer does not expose 2d/3d camera modes",
  };
}

function updateControlFeedbackFromProgress() {
  const feedback = state.lastControlFeedback;
  if (!feedback || !feedback.accepted) return;
  const deltaLogicalTime = Math.max(0, state.logicalTime - feedback.baselineLogicalTime);
  const deltaEventSeq = Math.max(0, state.eventSeq - feedback.baselineEventSeq);
  feedback.deltaLogicalTime = deltaLogicalTime;
  feedback.deltaEventSeq = deltaEventSeq;
  if (deltaLogicalTime > 0 || deltaEventSeq > 0) {
    feedback.stage = "completed_advanced";
    feedback.effect = `world advanced: logicalTime +${deltaLogicalTime}, eventSeq +${deltaEventSeq}`;
  }
}

function summarizeEventTitle(event) {
  const kind = event?.kind?.type || "unknown";
  return kind.replace(/_/g, " ");
}

function addRecentEvent(event) {
  state.recentEvents.unshift(event);
  state.recentEvents = state.recentEvents.slice(0, MAX_EVENTS);
  state.eventCount = state.recentEvents.length;
  state.eventSeq = Math.max(state.eventSeq, Number(event?.id || 0));
}

function handleSnapshot(snapshot) {
  state.snapshot = snapshot;
  state.logicalTime = Math.max(state.logicalTime, Number(snapshot?.time || 0));
  state.tick = state.logicalTime;
  const { agents, locations } = entityCollections();
  if (!state.selectedObject) {
    if (agents[0]) {
      applySelection({ kind: "agent", id: agents[0].id });
    } else if (locations[0]) {
      applySelection({ kind: "location", id: locations[0].id });
    }
  } else if (state.selectedKind && state.selectedId) {
    applySelection({ kind: state.selectedKind, id: state.selectedId });
  }
  syncAgentInteractionDrafts(false);
}

function handleMetrics(time, metrics) {
  state.metrics = metrics || null;
  state.traceCount = Number(metrics?.decision_trace_count || 0);
  state.logicalTime = Math.max(state.logicalTime, Number(time || 0), Number(metrics?.total_ticks || 0));
  state.tick = state.logicalTime;
}

function handleControlCompletionAck(ack) {
  const feedback = pendingControlFeedback.get(ack?.request_id) || state.lastControlFeedback;
  if (!feedback) return;
  feedback.deltaLogicalTime = Number(ack?.delta_logical_time || 0);
  feedback.deltaEventSeq = Number(ack?.delta_event_seq || 0);
  feedback.stage = ack?.status === "advanced" ? "completed_advanced" : "completed_timeout";
  feedback.effect =
    ack?.status === "advanced"
      ? `control ack advanced: logicalTime +${feedback.deltaLogicalTime}, eventSeq +${feedback.deltaEventSeq}`
      : "control ack timed out without progress";
  if (ack?.status !== "advanced") {
    feedback.reason = "timeout_no_progress";
  }
  state.lastControlFeedback = feedback;
  pendingControlFeedback.delete(feedback.requestId);
}

function cborHeader(majorType, length) {
  if (!Number.isInteger(length) || length < 0) {
    throw new Error(`invalid CBOR length: ${length}`);
  }
  if (length < 24) {
    return Uint8Array.of((majorType << 5) | length);
  }
  if (length < 0x100) {
    return Uint8Array.of((majorType << 5) | 24, length);
  }
  if (length < 0x10000) {
    return Uint8Array.of((majorType << 5) | 25, (length >> 8) & 0xff, length & 0xff);
  }
  if (length <= 0xffffffff) {
    return Uint8Array.of(
      (majorType << 5) | 26,
      (length >>> 24) & 0xff,
      (length >>> 16) & 0xff,
      (length >>> 8) & 0xff,
      length & 0xff,
    );
  }
  if (length <= Number.MAX_SAFE_INTEGER) {
    const value = BigInt(length);
    return Uint8Array.of(
      (majorType << 5) | 27,
      Number((value >> 56n) & 0xffn),
      Number((value >> 48n) & 0xffn),
      Number((value >> 40n) & 0xffn),
      Number((value >> 32n) & 0xffn),
      Number((value >> 24n) & 0xffn),
      Number((value >> 16n) & 0xffn),
      Number((value >> 8n) & 0xffn),
      Number(value & 0xffn),
    );
  }
  throw new Error("CBOR length exceeds Number.MAX_SAFE_INTEGER");
}

function concatBytes(...parts) {
  const totalLength = parts.reduce((sum, bytes) => sum + bytes.length, 0);
  const out = new Uint8Array(totalLength);
  let offset = 0;
  for (const bytes of parts) {
    out.set(bytes, offset);
    offset += bytes.length;
  }
  return out;
}

function cborEncode(value) {
  if (value === null) {
    return Uint8Array.of(0xf6);
  }
  if (value === false) {
    return Uint8Array.of(0xf4);
  }
  if (value === true) {
    return Uint8Array.of(0xf5);
  }
  if (typeof value === "number") {
    if (!Number.isInteger(value) || value < 0) {
      throw new Error(`unsupported CBOR number: ${value}`);
    }
    return cborHeader(0, value);
  }
  if (typeof value === "string") {
    const bytes = textEncoder.encode(value);
    return concatBytes(cborHeader(3, bytes.length), bytes);
  }
  if (Array.isArray(value)) {
    return concatBytes(cborHeader(4, value.length), ...value.map((entry) => cborEncode(entry)));
  }
  if (value instanceof Uint8Array) {
    return concatBytes(cborHeader(2, value.length), value);
  }
  if (typeof value === "object") {
    const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined);
    const encoded = [cborHeader(5, entries.length)];
    for (const [key, entryValue] of entries) {
      encoded.push(cborEncode(String(key)));
      encoded.push(cborEncode(entryValue));
    }
    return concatBytes(...encoded);
  }
  throw new Error(`unsupported CBOR type: ${typeof value}`);
}

function hexToBytes(raw) {
  const value = String(raw || "").trim().toLowerCase();
  if (!value || value.length % 2 !== 0 || /[^0-9a-f]/.test(value)) {
    throw new Error("invalid hex payload");
  }
  const bytes = new Uint8Array(value.length / 2);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number.parseInt(value.slice(index * 2, index * 2 + 2), 16);
  }
  return bytes;
}

function bytesToHex(bytes) {
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

async function importEd25519SigningKey(privateKeyHex) {
  if (!window.crypto?.subtle) {
    throw new Error("Web Crypto subtle API is unavailable");
  }
  if (!authKeyCache.has(privateKeyHex)) {
    const rawPrivateKey = hexToBytes(privateKeyHex);
    if (rawPrivateKey.length !== 32) {
      throw new Error(`viewer auth private key length mismatch: expected 32 bytes, got ${rawPrivateKey.length}`);
    }
    const pkcs8 = concatBytes(ED25519_PKCS8_PREFIX, rawPrivateKey);
    authKeyCache.set(
      privateKeyHex,
      window.crypto.subtle.importKey("pkcs8", pkcs8, { name: "Ed25519" }, false, ["sign"]),
    );
  }
  return authKeyCache.get(privateKeyHex);
}

async function signAuthPayload(signingPayloadBytes, auth) {
  const key = await importEd25519SigningKey(auth.privateKey);
  const signature = await window.crypto.subtle.sign({ name: "Ed25519" }, key, signingPayloadBytes);
  return `${VIEWER_AUTH_SIGNATURE_PREFIX}${bytesToHex(new Uint8Array(signature))}`;
}

function buildAuthEnvelope(payload) {
  return cborEncode({
    version: 1,
    payload,
  });
}

async function buildAgentChatAuthProof(request, auth) {
  const nonce = nextAuthNonce();
  const payload = {
    operation: "agent_chat",
    agent_id: request.agent_id,
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    message: request.message,
  };
  if (request.intent_tick != null) {
    payload.intent_tick = request.intent_tick;
  }
  if (request.intent_seq != null) {
    payload.intent_seq = request.intent_seq;
  }
  const signingPayload = buildAuthEnvelope(payload);
  return {
    scheme: "ed25519",
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    signature: await signAuthPayload(signingPayload, auth),
  };
}

function promptPatchFromDraft(currentValue, draftValue) {
  const current = currentValue == null ? "" : String(currentValue);
  const draft = String(draftValue ?? "");
  if (draft === current) {
    return { mode: "unchanged" };
  }
  if (draft.length === 0) {
    return currentValue == null ? { mode: "unchanged" } : { mode: "clear" };
  }
  return { mode: "set", value: draft };
}

async function buildPromptControlAuthProof(mode, request, auth) {
  const nonce = nextAuthNonce();
  const payload = {
    operation: mode === "preview" ? "prompt_control_preview" : "prompt_control_apply",
    agent_id: request.agent_id,
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    expected_version: request.expected_version ?? null,
    updated_by: request.updated_by ?? null,
    system_prompt_override: request.system_prompt_override,
    short_term_goal_override: request.short_term_goal_override,
    long_term_goal_override: request.long_term_goal_override,
  };
  const signingPayload = buildAuthEnvelope(payload);
  return {
    scheme: "ed25519",
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    signature: await signAuthPayload(signingPayload, auth),
  };
}

async function buildPromptRollbackAuthProof(request, auth) {
  const nonce = nextAuthNonce();
  const payload = {
    operation: "prompt_control_rollback",
    agent_id: request.agent_id,
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    to_version: request.to_version,
    expected_version: request.expected_version ?? null,
    updated_by: request.updated_by ?? null,
  };
  const signingPayload = buildAuthEnvelope(payload);
  return {
    scheme: "ed25519",
    player_id: auth.playerId,
    public_key: auth.publicKey,
    nonce,
    signature: await signAuthPayload(signingPayload, auth),
  };
}

function buildPromptRequestFromDraft(agentId, draftOverrides) {
  const currentProfile = selectedAgentPromptProfile();
  if (!agentId || !currentProfile) {
    throw new Error("select an agent before editing prompt overrides");
  }
  return {
    agent_id: agentId,
    player_id: state.auth.playerId,
    public_key: state.auth.publicKey,
    expected_version: Number(currentProfile.version || 0),
    updated_by: state.auth.playerId,
    system_prompt_override: promptPatchFromDraft(currentProfile.system_prompt_override, draftOverrides.systemPrompt),
    short_term_goal_override: promptPatchFromDraft(currentProfile.short_term_goal_override, draftOverrides.shortTermGoal),
    long_term_goal_override: promptPatchFromDraft(currentProfile.long_term_goal_override, draftOverrides.longTermGoal),
  };
}

function encodePromptRequestForJson(request) {
  const encodePatch = (patch) => {
    if (!patch || patch.mode === "unchanged") {
      return undefined;
    }
    if (patch.mode === "clear") {
      return null;
    }
    return patch.value;
  };
  return {
    agent_id: request.agent_id,
    player_id: request.player_id,
    public_key: request.public_key,
    expected_version: request.expected_version,
    updated_by: request.updated_by,
    system_prompt_override: encodePatch(request.system_prompt_override),
    short_term_goal_override: encodePatch(request.short_term_goal_override),
    long_term_goal_override: encodePatch(request.long_term_goal_override),
  };
}

function buildPromptRollbackRequest(agentId, toVersion) {
  const profile = selectedAgentPromptProfile();
  const targetVersion = Number(toVersion);
  if (!agentId || !profile) {
    throw new Error("select an agent before rolling back prompt overrides");
  }
  if (!Number.isInteger(targetVersion) || targetVersion < 0) {
    throw new Error("prompt rollback requires integer toVersion >= 0");
  }
  return {
    agent_id: agentId,
    player_id: state.auth.playerId,
    public_key: state.auth.publicKey,
    to_version: targetVersion,
    expected_version: Number(profile.version || 0),
    updated_by: state.auth.playerId,
  };
}

function pushChatHistory(entry) {
  if (!entry) {
    return;
  }
  state.chatHistory.unshift({
    id: entry.id || `${entry.source || "chat"}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    source: entry.source || "event",
    agentId: entry.agentId || null,
    locationId: entry.locationId || null,
    message: String(entry.message || ""),
    tick: Number(entry.tick || 0),
    speaker: entry.speaker || null,
    playerId: entry.playerId || null,
    targetAgentId: entry.targetAgentId || null,
    intentSeq: entry.intentSeq || null,
  });
  state.chatHistory = state.chatHistory.slice(0, 40);
}

function extractAgentSpokeEntry(event) {
  const kind = event?.kind;
  if (kind?.type !== "agent_spoke") {
    return null;
  }
  const data = kind.data || {};
  return {
    id: `event-${event.id}`,
    source: "event",
    agentId: data.agent_id || null,
    locationId: data.location_id || null,
    message: data.message || "",
    tick: Number(event.time || 0),
    speaker: data.agent_id || null,
    targetAgentId: data.target_agent_id || null,
  };
}

function requestSnapshotSafe() {
  try {
    sendJson({ type: "request_snapshot" });
  } catch (_) {
  }
}

function createSemanticFeedback(kind, action, agentId, extra = {}) {
  return {
    id: nextRequestId(),
    kind,
    action,
    agentId,
    accepted: true,
    ok: false,
    stage: "queued",
    reason: null,
    effect: null,
    response: null,
    ...extra,
  };
}

function enqueueSemanticCommand(command) {
  pendingSemanticCommands.push(command);
  if (!semanticSendLoop) {
    semanticSendLoop = processSemanticCommands();
  }
}

async function processSemanticCommands() {
  try {
    while (pendingSemanticCommands.length > 0) {
      const command = pendingSemanticCommands.shift();
      try {
        await command.execute();
      } catch (error) {
        command.feedback.stage = "error";
        command.feedback.ok = false;
        command.feedback.reason = String(error);
        command.feedback.effect = "request build/send failed";
        if (command.kind === "chat") {
          state.lastChatFeedback = command.feedback;
        } else {
          state.lastPromptFeedback = command.feedback;
        }
        render();
      }
    }
  } finally {
    semanticSendLoop = null;
    if (pendingSemanticCommands.length > 0) {
      semanticSendLoop = processSemanticCommands();
    }
  }
}

function assertAuthAvailable() {
  if (!state.auth.available) {
    throw new Error(state.auth.error || "viewer auth bootstrap is unavailable");
  }
}

function sendAgentChat(agentIdOrPayload, maybeMessage) {
  let agentId = null;
  let message = null;
  if (typeof agentIdOrPayload === "object" && agentIdOrPayload !== null) {
    agentId = String(agentIdOrPayload.agentId || agentIdOrPayload.agent_id || selectedAgentId() || "");
    message = String(agentIdOrPayload.message || "");
  } else {
    agentId = String(agentIdOrPayload || selectedAgentId() || "");
    message = String(maybeMessage || "");
  }
  if (!agentId) {
    return { ok: false, reason: "agent chat requires a selected agent or explicit agentId" };
  }
  if (!message.trim()) {
    return { ok: false, reason: "agent chat message cannot be empty" };
  }
  const feedback = createSemanticFeedback("chat", "agent_chat", agentId, {
    effect: "queued for signing and send",
    pendingMessage: message,
    pendingPlayerId: state.auth.playerId || null,
  });
  state.lastChatFeedback = feedback;
  enqueueSemanticCommand({
    kind: "chat",
    feedback,
    execute: async () => {
      assertAuthAvailable();
      feedback.stage = "signing";
      feedback.effect = "building auth proof";
      render();
      const request = {
        agent_id: agentId,
        message,
        player_id: state.auth.playerId,
        public_key: state.auth.publicKey,
      };
      request.auth = await buildAgentChatAuthProof(request, state.auth);
      feedback.stage = "sent";
      feedback.effect = "agent_chat request sent; waiting for ack";
      state.lastChatFeedback = feedback;
      sendJson({ type: "agent_chat", request });
      state.chatDraft.message = "";
      state.chatDraft.dirty = false;
      render();
    },
  });
  render();
  return { ok: true, feedback: snapshotSemanticFeedback(feedback) };
}

function sendPromptControl(mode, payload = null) {
  const normalizedMode = String(mode || "").trim().toLowerCase();
  if (!["preview", "apply", "rollback"].includes(normalizedMode)) {
    return { ok: false, reason: "prompt control mode must be preview, apply, or rollback" };
  }
  const selectedId = selectedAgentId();
  const agentId = String(payload?.agentId || payload?.agent_id || selectedId || "");
  if (!agentId) {
    return { ok: false, reason: "prompt control requires a selected agent or explicit agentId" };
  }
  let request;
  try {
    if (normalizedMode === "rollback") {
      const currentVersion = Number(state.promptDraft.currentVersion || selectedAgentPromptProfile()?.version || 0);
      const fallbackVersion = Math.max(0, currentVersion - 1);
      const toVersion = payload?.toVersion ?? payload?.to_version ?? fallbackVersion;
      request = buildPromptRollbackRequest(agentId, toVersion);
    } else {
      request = buildPromptRequestFromDraft(agentId, {
        systemPrompt: payload?.systemPrompt ?? payload?.system_prompt_override ?? state.promptDraft.systemPrompt,
        shortTermGoal: payload?.shortTermGoal ?? payload?.short_term_goal_override ?? state.promptDraft.shortTermGoal,
        longTermGoal: payload?.longTermGoal ?? payload?.long_term_goal_override ?? state.promptDraft.longTermGoal,
      });
    }
  } catch (error) {
    return { ok: false, reason: String(error) };
  }

  const feedback = createSemanticFeedback("prompt", `prompt_${normalizedMode}`, agentId, {
    effect: "queued for signing and send",
    toVersion: request.to_version ?? null,
  });
  state.lastPromptFeedback = feedback;
  enqueueSemanticCommand({
    kind: "prompt",
    feedback,
    execute: async () => {
      assertAuthAvailable();
      feedback.stage = "signing";
      feedback.effect = "building auth proof";
      render();
      let commandRequest;
      if (normalizedMode === "rollback") {
        commandRequest = {
          ...request,
          auth: await buildPromptRollbackAuthProof(request, state.auth),
        };
      } else {
        commandRequest = encodePromptRequestForJson(request);
        commandRequest.auth = await buildPromptControlAuthProof(normalizedMode, request, state.auth);
      }
      feedback.stage = "sent";
      feedback.effect = `prompt ${normalizedMode} request sent; waiting for ack`;
      state.lastPromptFeedback = feedback;
      sendJson({
        type: "prompt_control",
        command: {
          mode: normalizedMode,
          request: commandRequest,
        },
      });
      render();
    },
  });
  render();
  return { ok: true, feedback: snapshotSemanticFeedback(feedback) };
}

function applyPromptAckLocally(ack) {
  const agentId = ack?.agent_id;
  if (!agentId || !state.snapshot?.model) {
    return;
  }
  if (!state.snapshot.model.agent_prompt_profiles) {
    state.snapshot.model.agent_prompt_profiles = {};
  }
  const current = state.snapshot.model.agent_prompt_profiles[agentId] || { agent_id: agentId };
  const nextProfile = {
    ...current,
    agent_id: agentId,
    version: Number(ack.version || current.version || 0),
    updated_at_tick: Number(ack.updated_at_tick || state.logicalTime),
    updated_by: state.auth.playerId || current.updated_by || "",
  };
  if (!ack.preview) {
    nextProfile.system_prompt_override = state.promptDraft.systemPrompt || null;
    nextProfile.short_term_goal_override = state.promptDraft.shortTermGoal || null;
    nextProfile.long_term_goal_override = state.promptDraft.longTermGoal || null;
  }
  state.snapshot.model.agent_prompt_profiles[agentId] = nextProfile;
  if (selectedAgentId() === agentId) {
    state.promptDraft = {
      agentId,
      currentVersion: nextProfile.version,
      rollbackTargetVersion: Math.max(0, Number(nextProfile.version || 0) - 1),
      updatedBy: nextProfile.updated_by,
      updatedAtTick: nextProfile.updated_at_tick,
      systemPrompt: String(nextProfile.system_prompt_override || ""),
      shortTermGoal: String(nextProfile.short_term_goal_override || ""),
      longTermGoal: String(nextProfile.long_term_goal_override || ""),
      dirty: false,
    };
  }
}

function handlePromptControlAck(ack) {
  const feedback = state.lastPromptFeedback || createSemanticFeedback("prompt", "prompt_ack", ack?.agent_id || null);
  const operation = String(ack?.operation || (ack?.preview ? "preview" : "apply"));
  feedback.stage = ack?.preview ? "preview_ack" : operation === "rollback" ? "rollback_ack" : "apply_ack";
  feedback.ok = true;
  feedback.accepted = true;
  feedback.reason = null;
  feedback.effect = ack?.preview
    ? `prompt preview ready: version=${ack.version}`
    : operation === "rollback"
      ? `prompt rolled back via version=${ack.version} → target=${Number(ack?.rolled_back_to_version || 0)}`
      : `prompt applied: version=${ack.version}`;
  feedback.response = clone(ack);
  state.lastPromptFeedback = feedback;
  if (ack?.preview) {
    return;
  }
  if (operation === "rollback") {
    state.promptDraft.currentVersion = Number(ack?.version || state.promptDraft.currentVersion || 0);
    state.promptDraft.rollbackTargetVersion = Math.max(0, state.promptDraft.currentVersion - 1);
    state.promptDraft.dirty = false;
    requestSnapshotSafe();
    return;
  }
  applyPromptAckLocally(ack);
}

function handlePromptControlError(error) {
  const feedback = state.lastPromptFeedback || createSemanticFeedback("prompt", "prompt_error", error?.agent_id || selectedAgentId());
  feedback.stage = "error";
  feedback.ok = false;
  feedback.accepted = false;
  feedback.reason = error?.message || error?.code || "prompt control failed";
  feedback.effect = error?.code || "prompt control error";
  feedback.response = clone(error);
  state.lastPromptFeedback = feedback;
}

function handleAgentChatAck(ack) {
  const feedback = state.lastChatFeedback || createSemanticFeedback("chat", "agent_chat", ack?.agent_id || null);
  feedback.stage = "ack";
  feedback.ok = true;
  feedback.accepted = true;
  feedback.reason = null;
  feedback.effect = `chat accepted at tick ${Number(ack?.accepted_at_tick || state.logicalTime)}`;
  feedback.response = clone(ack);
  state.lastChatFeedback = feedback;
  pushChatHistory({
    id: `chat-ack-${feedback.id}`,
    source: "player",
    agentId: ack?.agent_id || feedback.agentId || null,
    message: feedback.pendingMessage || "",
    tick: Number(ack?.accepted_at_tick || state.logicalTime || 0),
    speaker: feedback.pendingPlayerId || state.auth.playerId || null,
    playerId: feedback.pendingPlayerId || state.auth.playerId || null,
    targetAgentId: ack?.agent_id || feedback.agentId || null,
    intentSeq: ack?.intent_seq || null,
  });
}

function handleAgentChatError(error) {
  const feedback = state.lastChatFeedback || createSemanticFeedback("chat", "agent_chat", error?.agent_id || selectedAgentId());
  feedback.stage = "error";
  feedback.ok = false;
  feedback.accepted = false;
  feedback.reason = error?.message || error?.code || "agent chat failed";
  feedback.effect = error?.code || "agent chat error";
  feedback.response = clone(error);
  state.lastChatFeedback = feedback;
}

function handleViewerMessage(message) {
  switch (message?.type) {
    case "hello_ack":
      state.server = message.server || null;
      state.worldId = message.world_id || null;
      state.controlProfile = message.control_profile || "playback";
      break;
    case "snapshot":
      handleSnapshot(message.snapshot);
      break;
    case "event": {
      addRecentEvent(message.event);
      const chatEntry = extractAgentSpokeEntry(message.event);
      if (chatEntry) {
        pushChatHistory(chatEntry);
      }
      state.logicalTime = Math.max(state.logicalTime, Number(message.event?.time || 0));
      state.tick = state.logicalTime;
      break;
    }
    case "metrics":
      handleMetrics(message.time, message.metrics);
      break;
    case "control_completion_ack":
      handleControlCompletionAck(message.ack);
      break;
    case "prompt_control_ack":
      handlePromptControlAck(message.ack);
      break;
    case "prompt_control_error":
      handlePromptControlError(message.error);
      break;
    case "agent_chat_ack":
      handleAgentChatAck(message.ack);
      break;
    case "agent_chat_error":
      handleAgentChatError(message.error);
      break;
    case "error":
      reportFatalError(message.message, "viewer");
      break;
    default:
      break;
  }
  updateControlFeedbackFromProgress();
  render();
}

function attachSocket(ws) {
  ws.addEventListener("open", () => {
    state.connectionStatus = "connected";
    state.lastError = null;
    sendJson({ type: "hello", client: "software_safe_viewer", version: 1 });
    sendJson({ type: "subscribe", streams: ["snapshot", "events", "metrics"], event_kinds: [] });
    sendJson({ type: "request_snapshot" });
    render();
  });

  ws.addEventListener("message", (event) => {
    try {
      const message = JSON.parse(String(event.data || "null"));
      handleViewerMessage(message);
    } catch (error) {
      reportFatalError(String(error), "viewer.parse");
    }
  });

  ws.addEventListener("error", () => {
    reportFatalError("websocket error", "viewer.ws");
  });

  ws.addEventListener("close", () => {
    state.connectionStatus = "connecting";
    render();
    if (reconnectTimer) {
      window.clearTimeout(reconnectTimer);
    }
    reconnectTimer = window.setTimeout(connect, 1200);
  });
}

function connect() {
  if (socket) {
    try {
      socket.close();
    } catch (_) {
    }
  }
  const params = getSearchParams();
  state.wsUrl = normalizeWsAddr(params.get("ws") || params.get("addr") || DEFAULT_WS_ADDR);
  state.connectionStatus = "connecting";
  render();
  socket = new WebSocket(state.wsUrl);
  attachSocket(socket);
}

function resourceSummary(resources) {
  if (!resources || typeof resources !== "object") {
    return "-";
  }
  return Object.entries(resources)
    .map(([key, value]) => {
      if (value && typeof value === "object") {
        return `${key}:${JSON.stringify(value)}`;
      }
      return `${key}:${value}`;
    })
    .join(" · ") || "-";
}

function modelLists() {
  const { agents, locations } = entityCollections();
  const keyword = selectedSearch.trim().toLowerCase();
  const filter = (entry, label) => {
    if (!keyword) return true;
    return String(label).toLowerCase().includes(keyword);
  };
  return {
    agents: agents
      .filter((agent) => filter(agent, `${agent.id} ${agent.location_id}`))
      .sort((a, b) => String(a.id).localeCompare(String(b.id))),
    locations: locations
      .filter((location) => filter(location, `${location.id} ${location.name}`))
      .sort((a, b) => String(a.id).localeCompare(String(b.id))),
  };
}

function connectionBadgeClass() {
  if (state.connectionStatus === "connected") return "badge badge--good";
  if (state.connectionStatus === "error") return "badge badge--bad";
  return "badge badge--warn";
}

function feedbackBadgeClass(feedback) {
  if (!feedback) return "badge";
  if (feedback.stage === "error") return "badge badge--bad";
  if (feedback.ok) return "badge badge--good";
  return "badge badge--warn";
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function renderLists() {
  const { agents, locations } = modelLists();
  const renderItem = (kind, entry, title, meta) => {
    const selected = state.selectedKind === kind && state.selectedId === entry.id;
    return `
      <button class="list-item" data-select-kind="${kind}" data-select-id="${escapeHtml(entry.id)}" data-selected="${selected}">
        <div class="list-item__title">${escapeHtml(title)}</div>
        <div class="list-item__meta">${escapeHtml(meta)}</div>
      </button>
    `;
  };

  elements.leftPanel.innerHTML = `
    <div class="stack">
      <div class="field">
        <label for="entity-search">Filter targets</label>
        <input id="entity-search" type="search" placeholder="Search agents or locations" value="${escapeHtml(selectedSearch)}" />
      </div>
      <div>
        <div class="panel__title" style="margin-bottom:10px;">Agents</div>
        <div class="list">
          ${agents.length
            ? agents
                .map((agent) =>
                  renderItem(
                    "agent",
                    agent,
                    agent.id,
                    `location=${agent.location_id} · resources=${resourceSummary(agent.resources)}`,
                  ),
                )
                .join("")
            : '<div class="empty">No agents in current snapshot.</div>'}
        </div>
      </div>
      <div>
        <div class="panel__title" style="margin-bottom:10px;">Locations</div>
        <div class="list">
          ${locations.length
            ? locations
                .map((location) =>
                  renderItem(
                    "location",
                    location,
                    location.name || location.id,
                    `id=${location.id} · resources=${resourceSummary(location.resources)}`,
                  ),
                )
                .join("")
            : '<div class="empty">No locations in current snapshot.</div>'}
        </div>
      </div>
    </div>
  `;
}

function renderSummary() {
  const controlFeedback = snapshotControlFeedback(state.lastControlFeedback);
  const promptFeedback = snapshotSemanticFeedback(state.lastPromptFeedback);
  const chatFeedback = snapshotSemanticFeedback(state.lastChatFeedback);
  const authBadgeClass = state.auth.available ? "badge badge--good" : "badge badge--warn";
  elements.centerPanel.innerHTML = `
    <div class="stack">
      <div class="badge-row">
        <span class="badge badge--accent">software_safe</span>
        <span class="${connectionBadgeClass()}">${escapeHtml(state.connectionStatus)}</span>
        <span class="badge">rendererClass=${escapeHtml(state.rendererClass)}</span>
        <span class="badge">controlProfile=${escapeHtml(state.controlProfile)}</span>
      </div>
      <div class="summary-grid">
        <div class="metric"><div class="metric__label">Logical Time</div><div class="metric__value">${state.logicalTime}</div></div>
        <div class="metric"><div class="metric__label">Event Seq</div><div class="metric__value">${state.eventSeq}</div></div>
        <div class="metric"><div class="metric__label">World</div><div class="metric__value">${escapeHtml(state.worldId || "-")}</div></div>
        <div class="metric"><div class="metric__label">Viewer Server</div><div class="metric__value">${escapeHtml(state.server || "-")}</div></div>
      </div>
      <div class="badge-row">
        <span class="badge">ws=${escapeHtml(state.wsUrl || "-")}</span>
        <span class="badge">reason=${escapeHtml(state.softwareSafeReason || "-")}</span>
        <span class="badge">renderer=${escapeHtml(state.renderer || "n/a")}</span>
      </div>
      <div class="badge-row">
        <span class="${authBadgeClass}">auth=${state.auth.available ? "ready" : "missing"}</span>
        <span class="badge">player=${escapeHtml(state.auth.playerId || "-")}</span>
        <span class="badge">pubkey=${escapeHtml(state.auth.publicKey ? `${state.auth.publicKey.slice(0, 10)}…` : "-")}</span>
      </div>
      <div class="panel panel--nested" style="background:rgba(255,255,255,0.02);">
        <div class="panel__header"><div class="panel__title">Playback Controls</div></div>
        <div class="panel__body stack">
          <div class="toolbar">
            <button data-action="play">Play</button>
            <button data-action="pause">Pause</button>
            <button data-action="step">Step x1</button>
          </div>
          <div class="control-grid">
            <div class="field">
              <label for="step-count">Step count</label>
              <input id="step-count" type="number" min="1" step="1" value="3" />
            </div>
            <div class="field" style="align-self:end;">
              <button data-action="step-count">Step custom count</button>
            </div>
          </div>
          ${controlFeedback
            ? `<div class="badge-row">
                <span class="badge">action=${escapeHtml(controlFeedback.action)}</span>
                <span class="badge">stage=${escapeHtml(controlFeedback.stage)}</span>
                <span class="badge">Δtick=${controlFeedback.deltaLogicalTime}</span>
                <span class="badge">Δevent=${controlFeedback.deltaEventSeq}</span>
              </div>
              <pre class="json">${escapeHtml(JSON.stringify(controlFeedback, null, 2))}</pre>`
            : '<div class="empty">No control feedback yet.</div>'}
        </div>
      </div>
      <div class="summary-grid">
        <div class="metric">
          <div class="metric__label">Prompt Feedback</div>
          <div class="metric__value">${escapeHtml(promptFeedback?.stage || "idle")}</div>
          ${promptFeedback ? `<div class="badge-row" style="margin-top:8px;"><span class="${feedbackBadgeClass(promptFeedback)}">${escapeHtml(promptFeedback.effect || promptFeedback.reason || "ready")}</span></div>` : ""}
        </div>
        <div class="metric">
          <div class="metric__label">Chat Feedback</div>
          <div class="metric__value">${escapeHtml(chatFeedback?.stage || "idle")}</div>
          ${chatFeedback ? `<div class="badge-row" style="margin-top:8px;"><span class="${feedbackBadgeClass(chatFeedback)}">${escapeHtml(chatFeedback.effect || chatFeedback.reason || "ready")}</span></div>` : ""}
        </div>
      </div>
      <div>
        <div class="panel__title" style="margin-bottom:10px;">Recent Events</div>
        <div class="event-list">
          ${state.recentEvents.length
            ? state.recentEvents
                .map(
                  (event) => `
                    <div class="event-card">
                      <div class="event-card__title">
                        <span>${escapeHtml(summarizeEventTitle(event))}</span>
                        <span>#${Number(event.id || 0)}</span>
                      </div>
                      <div class="event-card__meta">time=${Number(event.time || 0)}</div>
                      <pre class="json">${escapeHtml(JSON.stringify(event.kind, null, 2))}</pre>
                    </div>`,
                )
                .join("")
            : '<div class="empty">Waiting for live events…</div>'}
        </div>
      </div>
    </div>
  `;
}

function renderInteractionPanel() {
  const agentId = selectedAgentId();
  if (!agentId) {
    return '<div class="empty">Select an agent to unlock prompt/chat controls.</div>';
  }
  const binding = selectedAgentBindingInfo();
  const promptFeedback = snapshotSemanticFeedback(state.lastPromptFeedback);
  const chatFeedback = snapshotSemanticFeedback(state.lastChatFeedback);
  const authReady = state.auth.available;
  const authNotice = authReady
    ? `<div class="badge-row"><span class="badge badge--good">auth bootstrap ready</span><span class="badge">player=${escapeHtml(state.auth.playerId)}</span></div>`
    : `<div class="empty">Prompt/chat require viewer auth bootstrap. Current status: ${escapeHtml(state.auth.error || "missing")}</div>`;
  const chatHistory = state.chatHistory
    .filter((entry) => entry.agentId === agentId || entry.targetAgentId === agentId)
    .slice(0, 12);

  return `
    <div class="stack">
      <div class="badge-row">
        <span class="badge badge--accent">Agent Interaction</span>
        <span class="badge">agent=${escapeHtml(agentId)}</span>
        <span class="badge">promptVersion=${Number(state.promptDraft.currentVersion || 0)}</span>
      </div>
      ${authNotice}
      <div class="badge-row">
        <span class="badge">boundPlayer=${escapeHtml(binding?.playerId || "-")}</span>
        <span class="badge">boundKey=${escapeHtml(binding?.publicKey ? `${binding.publicKey.slice(0, 10)}…` : "-")}</span>
      </div>
      <div class="panel panel--nested" style="background:rgba(255,255,255,0.02);">
        <div class="panel__header"><div class="panel__title">Prompt Overrides</div></div>
        <div class="panel__body stack">
          <div class="field">
            <label for="prompt-system">System Prompt Override</label>
            <textarea id="prompt-system" rows="4" ${authReady ? "" : "disabled"}>${escapeHtml(state.promptDraft.systemPrompt)}</textarea>
          </div>
          <div class="field">
            <label for="prompt-short">Short-Term Goal Override</label>
            <textarea id="prompt-short" rows="3" ${authReady ? "" : "disabled"}>${escapeHtml(state.promptDraft.shortTermGoal)}</textarea>
          </div>
          <div class="field">
            <label for="prompt-long">Long-Term Goal Override</label>
            <textarea id="prompt-long" rows="3" ${authReady ? "" : "disabled"}>${escapeHtml(state.promptDraft.longTermGoal)}</textarea>
          </div>
          <div class="toolbar">
            <button data-prompt-action="preview" ${authReady ? "" : "disabled"}>Preview Prompt</button>
            <button data-prompt-action="apply" ${authReady ? "" : "disabled"}>Apply Prompt</button>
          </div>
          <div class="toolbar">
            <div class="field" style="margin:0; min-width:180px; flex:1;">
              <label for="prompt-rollback-version">Rollback Target Version</label>
              <input id="prompt-rollback-version" type="number" min="0" step="1" value="${Number(state.promptDraft.rollbackTargetVersion || 0)}" ${authReady ? "" : "disabled"} />
            </div>
            <button data-prompt-action="rollback" ${authReady ? "" : "disabled"}>Rollback Prompt</button>
          </div>
          ${promptFeedback
            ? `<div class="badge-row"><span class="${feedbackBadgeClass(promptFeedback)}">${escapeHtml(promptFeedback.stage)}</span></div>
               <pre class="json">${escapeHtml(JSON.stringify(promptFeedback, null, 2))}</pre>`
            : '<div class="empty">No prompt feedback yet.</div>'}
        </div>
      </div>
      <div class="panel panel--nested" style="background:rgba(255,255,255,0.02);">
        <div class="panel__header"><div class="panel__title">Agent Chat</div></div>
        <div class="panel__body stack">
          <div class="field">
            <label for="agent-chat-message">Message</label>
            <textarea id="agent-chat-message" rows="4" placeholder="Send a message to the selected agent" ${authReady ? "" : "disabled"}>${escapeHtml(state.chatDraft.message)}</textarea>
          </div>
          <div class="toolbar">
            <button data-chat-send="1" ${authReady ? "" : "disabled"}>Send Chat</button>
          </div>
          ${chatFeedback
            ? `<div class="badge-row"><span class="${feedbackBadgeClass(chatFeedback)}">${escapeHtml(chatFeedback.stage)}</span></div>
               <pre class="json">${escapeHtml(JSON.stringify(chatFeedback, null, 2))}</pre>`
            : '<div class="empty">No chat feedback yet.</div>'}
          <div>
            <div class="panel__title" style="margin-bottom:10px;">Message Flow</div>
            <div class="event-list">
              ${chatHistory.length
                ? chatHistory
                    .map(
                      (entry) => `
                        <div class="event-card">
                          <div class="event-card__title">
                            <span>${escapeHtml(entry.source === "player" ? `player → ${entry.targetAgentId || entry.agentId || "agent"}` : `${entry.agentId || "agent"} spoke`)}</span>
                            <span>tick=${Number(entry.tick || 0)}</span>
                          </div>
                          <div class="event-card__meta">speaker=${escapeHtml(entry.speaker || entry.playerId || "-")} · location=${escapeHtml(entry.locationId || "-")}</div>
                          <pre class="json">${escapeHtml(JSON.stringify(entry, null, 2))}</pre>
                        </div>`,
                    )
                    .join("")
                : '<div class="empty">No chat history for this agent yet.</div>'}
            </div>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderDetails() {
  const selectedLabel = state.selectedKind && state.selectedId
    ? `${state.selectedKind}:${state.selectedId}`
    : "nothing selected";
  elements.rightPanel.innerHTML = `
    <div class="stack">
      <div class="badge-row">
        <span class="badge badge--accent">Selected</span>
        <span class="badge">${escapeHtml(selectedLabel)}</span>
      </div>
      ${renderInteractionPanel()}
      ${state.selectedObject
        ? `<pre class="json">${escapeHtml(JSON.stringify(clone(state.selectedObject), null, 2))}</pre>`
        : '<div class="empty">Select an agent or location from the left list.</div>'}
      <div>
        <div class="panel__title" style="margin-bottom:10px;">Snapshot Summary</div>
        <pre class="json">${escapeHtml(
          JSON.stringify(
            {
              config: state.snapshot?.config || null,
              counts: {
                agents: Object.keys(state.snapshot?.model?.agents || {}).length,
                locations: Object.keys(state.snapshot?.model?.locations || {}).length,
                promptProfiles: Object.keys(state.snapshot?.model?.agent_prompt_profiles || {}).length,
              },
              metrics: state.metrics,
            },
            null,
            2,
          ),
        )}</pre>
      </div>
      ${state.lastError
        ? `<div>
            <div class="panel__title" style="margin-bottom:10px; color: var(--bad);">Last Error</div>
            <pre class="json">${escapeHtml(state.lastError)}</pre>
          </div>`
        : ""}
    </div>
  `;
}

function bindEvents() {
  const searchInput = document.getElementById("entity-search");
  if (searchInput) {
    searchInput.addEventListener("input", (event) => {
      selectedSearch = String(event.target.value || "");
      renderLists();
      bindEvents();
    });
  }

  document.querySelectorAll("[data-select-kind][data-select-id]").forEach((button) => {
    button.addEventListener("click", () => {
      applySelection({
        kind: button.getAttribute("data-select-kind"),
        id: button.getAttribute("data-select-id"),
      });
    });
  });

  document.querySelectorAll("[data-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const action = button.getAttribute("data-action");
      if (action === "step-count") {
        const value = Number(document.getElementById("step-count")?.value || 1);
        sendControl("step", { count: Math.max(1, Math.floor(value || 1)) });
        return;
      }
      sendControl(action, null);
    });
  });

  const promptSystem = document.getElementById("prompt-system");
  if (promptSystem) {
    promptSystem.addEventListener("input", (event) => {
      state.promptDraft.systemPrompt = String(event.target.value || "");
      state.promptDraft.dirty = true;
    });
  }
  const promptShort = document.getElementById("prompt-short");
  if (promptShort) {
    promptShort.addEventListener("input", (event) => {
      state.promptDraft.shortTermGoal = String(event.target.value || "");
      state.promptDraft.dirty = true;
    });
  }
  const promptLong = document.getElementById("prompt-long");
  if (promptLong) {
    promptLong.addEventListener("input", (event) => {
      state.promptDraft.longTermGoal = String(event.target.value || "");
      state.promptDraft.dirty = true;
    });
  }
  const promptRollbackVersion = document.getElementById("prompt-rollback-version");
  if (promptRollbackVersion) {
    promptRollbackVersion.addEventListener("input", (event) => {
      const nextValue = Number(event.target.value || 0);
      state.promptDraft.rollbackTargetVersion = Math.max(0, Math.floor(nextValue || 0));
    });
  }
  document.querySelectorAll("[data-prompt-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const action = button.getAttribute("data-prompt-action");
      if (action === "rollback") {
        sendPromptControl("rollback", {
          toVersion: Number(state.promptDraft.rollbackTargetVersion || 0),
        });
        return;
      }
      sendPromptControl(action, null);
    });
  });

  const chatMessage = document.getElementById("agent-chat-message");
  if (chatMessage) {
    chatMessage.addEventListener("input", (event) => {
      state.chatDraft.message = String(event.target.value || "");
      state.chatDraft.dirty = true;
    });
  }
  document.querySelectorAll("[data-chat-send]").forEach((button) => {
    button.addEventListener("click", () => {
      sendAgentChat(selectedAgentId(), state.chatDraft.message);
    });
  });
}

function render() {
  renderLists();
  renderSummary();
  renderDetails();
  bindEvents();
}

function mountApp() {
  const app = document.getElementById("app");
  app.innerHTML = `
    <section class="panel"><div class="panel__header"><div class="panel__title">Targets</div></div><div id="left-panel" class="panel__body"></div></section>
    <section class="panel"><div class="panel__header"><div class="panel__title">World Summary</div></div><div id="center-panel" class="panel__body"></div></section>
    <section class="panel"><div class="panel__header"><div class="panel__title">Details</div></div><div id="right-panel" class="panel__body"></div></section>
  `;
  elements.leftPanel = document.getElementById("left-panel");
  elements.centerPanel = document.getElementById("center-panel");
  elements.rightPanel = document.getElementById("right-panel");
}

function installTestApi() {
  window[TEST_API_GLOBAL_NAME] = {
    getState,
    describeControls,
    fillControlExample,
    sendControl,
    runSteps,
    setMode,
    focus,
    select,
    sendAgentChat,
    sendPromptControl,
    reportFatalError,
  };
}

function bootstrap() {
  Object.assign(state, detectRendererMeta());
  state.auth = resolveAuthBootstrap();
  window[RENDER_META_GLOBAL_NAME] = Object.freeze({
    renderMode: state.renderMode,
    rendererClass: state.rendererClass,
    softwareSafeReason: state.softwareSafeReason,
    renderer: state.renderer,
    vendor: state.vendor,
    webglVersion: state.webglVersion,
  });
  mountApp();
  installTestApi();
  render();
  connect();
}

window.addEventListener("error", (event) => {
  const message = event?.message || event?.error?.message || "window error";
  reportFatalError(message, "window.error");
});
window.addEventListener("unhandledrejection", (event) => {
  const message = event?.reason?.message || String(event?.reason || "unhandled rejection");
  reportFatalError(message, "window.unhandledrejection");
});

bootstrap();
