#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-release-qa-loop.sh [options]

Options:
  --scenario <name>          world_viewer_live scenario (default: llm_bootstrap)
  --live-bind <host:port>    live tcp bind (default: 127.0.0.1:5023)
  --web-bind <host:port>     web bridge bind (default: 127.0.0.1:5011)
  --viewer-host <host>       web viewer host (default: 127.0.0.1)
  --viewer-port <port>       web viewer port (default: 4173)
  --tick-ms <ms>             live tick interval (default: 300)
  --out-dir <path>           artifact output dir (default: output/playwright/viewer)
  --with-consensus-gate      keep world_viewer_live consensus gate/topology defaults
  --skip-visual-baseline     skip scripts/viewer-visual-baseline.sh
  --headed                   open browser in headed mode
  -h, --help                 show this help

Artifacts:
  <out-dir>/release-qa-*.log
  <out-dir>/release-qa-*.png
  <out-dir>/release-qa-summary-*.md
USAGE
}

require_cmd() {
  local cmd=$1
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: missing required command: $cmd" >&2
    exit 1
  fi
}

wait_for_port() {
  local host=$1
  local port=$2
  local timeout_secs=$3
  local step
  for step in $(seq 1 "$timeout_secs"); do
    if nc -z "$host" "$port" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

wait_for_http() {
  local url=$1
  local timeout_secs=$2
  local step
  for step in $(seq 1 "$timeout_secs"); do
    if curl -sf "$url" >/dev/null; then
      return 0
    fi
    sleep 1
  done
  return 1
}

scenario="llm_bootstrap"
live_bind="127.0.0.1:5023"
web_bind="127.0.0.1:5011"
viewer_host="127.0.0.1"
viewer_port="4173"
tick_ms="300"
out_dir="output/playwright/viewer"
with_consensus_gate=0
skip_visual_baseline=0
headed=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --live-bind)
      live_bind=${2:-}
      shift 2
      ;;
    --web-bind)
      web_bind=${2:-}
      shift 2
      ;;
    --viewer-host)
      viewer_host=${2:-}
      shift 2
      ;;
    --viewer-port)
      viewer_port=${2:-}
      shift 2
      ;;
    --tick-ms)
      tick_ms=${2:-}
      shift 2
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
      ;;
    --with-consensus-gate)
      with_consensus_gate=1
      shift 1
      ;;
    --skip-visual-baseline)
      skip_visual_baseline=1
      shift 1
      ;;
    --headed)
      headed=1
      shift 1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! "$viewer_port" =~ ^[0-9]+$ ]]; then
  echo "error: --viewer-port must be an integer" >&2
  exit 2
fi
if [[ ! "$tick_ms" =~ ^[0-9]+$ ]]; then
  echo "error: --tick-ms must be an integer" >&2
  exit 2
fi
if [[ "$live_bind" != *:* ]]; then
  echo "error: --live-bind must be host:port" >&2
  exit 2
fi
if [[ "$web_bind" != *:* ]]; then
  echo "error: --web-bind must be host:port" >&2
  exit 2
fi

require_cmd nc
require_cmd curl
require_cmd npx
require_cmd cargo

mkdir -p "$out_dir"
stamp=$(date +%Y%m%d-%H%M%S)
live_log="$out_dir/release-qa-live-${stamp}.log"
web_log="$out_dir/release-qa-web-${stamp}.log"
pw_log="$out_dir/release-qa-playwright-${stamp}.log"
semantic_log="$out_dir/release-qa-semantic-${stamp}.log"
zoom_log="$out_dir/release-qa-zoom-${stamp}.log"
summary_path="$out_dir/release-qa-summary-${stamp}.md"
shot_path="$out_dir/release-qa-${stamp}.png"
zoom_shot_near="$out_dir/release-qa-zoom-near-${stamp}.png"
zoom_shot_mid="$out_dir/release-qa-zoom-mid-${stamp}.png"
zoom_shot_far="$out_dir/release-qa-zoom-far-${stamp}.png"

live_pid=""
web_pid=""

cleanup() {
  set +e
  if [[ -n "$web_pid" ]]; then
    kill "$web_pid" >/dev/null 2>&1 || true
    wait "$web_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$live_pid" ]]; then
    kill "$live_pid" >/dev/null 2>&1 || true
    wait "$live_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "${PWCLI:-}" ]]; then
    bash "$PWCLI" close >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if [[ "$skip_visual_baseline" -eq 0 ]]; then
  echo "+ ./scripts/viewer-visual-baseline.sh"
  ./scripts/viewer-visual-baseline.sh
fi

live_host=${live_bind%:*}
live_port=${live_bind##*:}
web_host=${web_bind%:*}
web_port=${web_bind##*:}
viewer_url="http://${viewer_host}:${viewer_port}/?ws=ws://${web_bind}&test_api=1"

live_args=("$scenario" "--bind" "$live_bind" "--web-bind" "$web_bind" "--tick-ms" "$tick_ms")
if [[ "$with_consensus_gate" -eq 0 ]]; then
  # Release QA loop uses single topology + no gate by default to avoid
  # triad consensus readiness from masking viewer semantic regressions.
  live_args+=("--topology" "single" "--viewer-no-consensus-gate" "--no-node")
fi

echo "+ env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- ${live_args[*]}"
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- "${live_args[@]}" >"$live_log" 2>&1 &
live_pid=$!

echo "+ wait for bridge $web_host:$web_port"
if ! wait_for_port "$web_host" "$web_port" 90; then
  echo "error: web bridge did not come up on $web_host:$web_port" >&2
  exit 1
fi

echo "+ env -u NO_COLOR ./scripts/run-viewer-web.sh --address $viewer_host --port $viewer_port"
env -u NO_COLOR ./scripts/run-viewer-web.sh --address "$viewer_host" --port "$viewer_port" >"$web_log" 2>&1 &
web_pid=$!

echo "+ wait for viewer $viewer_url"
if ! wait_for_http "http://${viewer_host}:${viewer_port}/" 180; then
  echo "error: viewer web server did not become ready: $viewer_host:$viewer_port" >&2
  exit 1
fi

if [[ -s "$HOME/.nvm/nvm.sh" ]]; then
  # Keep Playwright runtime stable with the repo's preferred Node toolchain.
  source "$HOME/.nvm/nvm.sh"
  nvm use 24 >/dev/null || true
fi

export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
PWCLI="$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh"
if [[ ! -f "$PWCLI" ]]; then
  echo "error: playwright wrapper not found: $PWCLI" >&2
  exit 1
fi

open_args=(open "$viewer_url")
if [[ "$headed" -eq 1 ]]; then
  open_args+=(--headed)
fi

{
  echo "+ bash \"$PWCLI\" ${open_args[*]}"
  bash "$PWCLI" "${open_args[@]}"
  echo "+ bash \"$PWCLI\" snapshot"
  bash "$PWCLI" snapshot
} | tee "$pw_log"

semantic_code=$(cat <<'JS'
async (page) => {
  const fail = (message) => {
    throw new Error(message);
  };
  const waitForApi = async () => {
    const deadline = Date.now() + 20000;
    while (Date.now() < deadline) {
      const ready = await page.evaluate(() => typeof window.__AW_TEST__ === "object");
      if (ready) {
        return;
      }
      await page.waitForTimeout(200);
    }
    fail("__AW_TEST__ is unavailable");
  };

  const waitForConnected = async (label, timeoutMs = 15000) => {
    const deadline = Date.now() + timeoutMs;
    let state = null;
    while (Date.now() < deadline) {
      state = await page.evaluate(() => window.__AW_TEST__.getState());
      if (state?.connectionStatus === "connected") {
        return state;
      }
      await page.waitForTimeout(250);
    }
    fail(
      `${label}: not connected (status=${state?.connectionStatus}, lastError=${state?.lastError}, errorCount=${state?.errorCount})`,
    );
  };

  const waitForTickAdvance = async (label, baselineTick, timeoutMs = 6000) => {
    const deadline = Date.now() + timeoutMs;
    let state = null;
    while (Date.now() < deadline) {
      state = await page.evaluate(() => window.__AW_TEST__.getState());
      if (state?.connectionStatus === "connected" && Number(state.tick || 0) > baselineTick) {
        return state;
      }
      await page.waitForTimeout(250);
    }
    fail(
      `${label}: tick did not advance (baseline=${baselineTick}, finalTick=${state?.tick}, status=${state?.connectionStatus}, lastError=${state?.lastError})`,
    );
  };

  await waitForApi();

  const initial = await page.evaluate(() => window.__AW_TEST__.getState());
  if (!initial || typeof initial.connectionStatus !== "string" || typeof initial.tick !== "number") {
    fail("getState() missing required fields");
  }
  const connectedState = await waitForConnected("initial connection");
  const controlBefore = connectedState;
  const tickBefore = Number(controlBefore.tick || 0);

  await page.evaluate(() => window.__AW_TEST__.sendControl("play"));
  let afterPlay = null;
  try {
    afterPlay = await waitForTickAdvance("after play", tickBefore, 3500);
  } catch (_playErr) {
    // Some scenarios stay idle under play; force one deterministic tick move via seek.
    await page.evaluate(
      (nextTick) => window.__AW_TEST__.sendControl("seek", { tick: nextTick }),
      Math.max(1, Math.floor(tickBefore) + 1),
    );
    afterPlay = await waitForTickAdvance("after play/seek fallback", tickBefore, 6000);
  }

  await page.evaluate(() => window.__AW_TEST__.sendControl("pause"));
  await page.waitForTimeout(450);
  const paused = await waitForConnected("after pause");
  await page.waitForTimeout(600);
  const pausedFollowup = await page.evaluate(() => window.__AW_TEST__.getState());
  if (pausedFollowup.connectionStatus !== "connected") {
    fail(
      `connection dropped after pause settle (status=${pausedFollowup.connectionStatus}, lastError=${pausedFollowup.lastError})`,
    );
  }
  if (Number(pausedFollowup.tick || 0) > Number(paused.tick || 0) + 2) {
    fail(
      `pause control failed to stabilize tick (paused=${paused.tick}, followup=${pausedFollowup.tick})`,
    );
  }

  await page.evaluate(() => window.__AW_TEST__.runSteps("mode=3d;focus=first_location;zoom=0.85;select=first_agent;wait=0.3"));
  const selectionDeadline = Date.now() + 6000;
  let selected = await page.evaluate(() => window.__AW_TEST__.getState());
  while (Date.now() < selectionDeadline && selected.selectedKind !== "agent") {
    await page.waitForTimeout(250);
    selected = await page.evaluate(() => window.__AW_TEST__.getState());
  }
  if (selected.selectedKind !== "agent") {
    fail(`selection did not resolve to agent (selectedKind=${selected.selectedKind})`);
  }
  const final = await waitForConnected("after semantic actions");
  if (final.lastError) {
    fail(`connected but lastError is not cleared: ${final.lastError}`);
  }

  return {
    initial,
    controlBefore,
    afterPlay,
    paused,
    pausedFollowup,
    selected,
    final,
  };
}
JS
)

semantic_ok=1
semantic_output=""
if ! semantic_output=$(bash "$PWCLI" run-code "$semantic_code" 2>&1); then
  semantic_ok=0
fi
if printf "%s\n" "$semantic_output" | rg -q "^### Error"; then
  semantic_ok=0
fi
printf "%s\n" "$semantic_output" | tee "$semantic_log" | tee -a "$pw_log" >/dev/null

zoom_ok=1
zoom_output=""
zoom_code=$(cat <<JS
async (page) => {
  const fail = (message) => {
    throw new Error(message);
  };

  const waitForApi = async () => {
    const deadline = Date.now() + 20000;
    while (Date.now() < deadline) {
      const ready = await page.evaluate(() => typeof window.__AW_TEST__ === "object");
      if (ready) {
        return;
      }
      await page.waitForTimeout(200);
    }
    fail("__AW_TEST__ unavailable after reload");
  };

  const waitForConnected = async (label, timeoutMs = 20000) => {
    const deadline = Date.now() + timeoutMs;
    let state = null;
    while (Date.now() < deadline) {
      state = await page.evaluate(() => window.__AW_TEST__.getState());
      if (state?.connectionStatus === "connected") {
        return state;
      }
      await page.waitForTimeout(250);
    }
    fail(
      \`\${label}: not connected (status=\${state?.connectionStatus}, lastError=\${state?.lastError}, errorCount=\${state?.errorCount})\`,
    );
  };

  const screenshotMetrics = async (base64) =>
    page.evaluate(async (encoded) => {
      const src = \`data:image/png;base64,\${encoded}\`;
      const image = await new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = () => reject(new Error("screenshot decode failed"));
        img.src = src;
      });

      // Exclude right-side panel area so zoom deltas are measured on the 3D scene.
      const panelInset = Math.min(420, Math.max(240, Math.floor(image.width * 0.26)));
      const sceneWidth = Math.max(64, image.width - panelInset);
      const sceneHeight = image.height;
      const sampleWidth = Math.max(64, Math.min(480, sceneWidth));
      const sampleHeight = Math.max(64, Math.min(300, sceneHeight));
      const probe = document.createElement("canvas");
      probe.width = sampleWidth;
      probe.height = sampleHeight;
      const ctx = probe.getContext("2d", { willReadFrequently: true });
      if (!ctx) {
        throw new Error("2d context unavailable");
      }
      ctx.drawImage(image, 0, 0, sceneWidth, sceneHeight, 0, 0, sampleWidth, sampleHeight);
      const pixels = ctx.getImageData(0, 0, sampleWidth, sampleHeight).data;

      let count = 0;
      let nonDark = 0;
      let sum = 0;
      let sum2 = 0;
      let edgeSum = 0;
      let edgeCount = 0;
      const buckets = new Set();
      const rowStride = sampleWidth * 4;

      for (let i = 0; i + 3 < pixels.length; i += 16) {
        const r = pixels[i];
        const g = pixels[i + 1];
        const b = pixels[i + 2];
        const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        sum += luma;
        sum2 += luma * luma;
        count += 1;
        if (luma > 12) nonDark += 1;
        buckets.add(
          [
            Math.floor(r / 32),
            Math.floor(g / 32),
            Math.floor(b / 32),
          ].join(":"),
        );
      }

      for (let y = 1; y < sampleHeight - 1; y += 4) {
        for (let x = 1; x < sampleWidth - 1; x += 4) {
          const index = y * rowStride + x * 4;
          const right = index + 4;
          const down = index + rowStride;
          const luma = 0.2126 * pixels[index] + 0.7152 * pixels[index + 1] + 0.0722 * pixels[index + 2];
          const lumaRight =
            0.2126 * pixels[right] + 0.7152 * pixels[right + 1] + 0.0722 * pixels[right + 2];
          const lumaDown =
            0.2126 * pixels[down] + 0.7152 * pixels[down + 1] + 0.0722 * pixels[down + 2];
          edgeSum += Math.abs(luma - lumaRight) + Math.abs(luma - lumaDown);
          edgeCount += 1;
        }
      }

      if (count < 196) {
        throw new Error(\`insufficient sampled pixels (count=\${count}, sample=\${sampleWidth}x\${sampleHeight})\`);
      }

      const mean = sum / count;
      const variance = Math.max(0, sum2 / count - mean * mean);
      const nonDarkRatio = nonDark / count;
      const detailScore = edgeCount > 0 ? edgeSum / edgeCount : 0;
      const signatureWidth = 16;
      const signatureHeight = 12;
      const signatureCanvas = document.createElement("canvas");
      signatureCanvas.width = signatureWidth;
      signatureCanvas.height = signatureHeight;
      const signatureCtx = signatureCanvas.getContext("2d", { willReadFrequently: true });
      if (!signatureCtx) {
        throw new Error("signature context unavailable");
      }
      signatureCtx.drawImage(
        image,
        0,
        0,
        sceneWidth,
        sceneHeight,
        0,
        0,
        signatureWidth,
        signatureHeight,
      );
      const signaturePixels = signatureCtx
        .getImageData(0, 0, signatureWidth, signatureHeight)
        .data;
      const signature = [];
      for (let i = 0; i + 3 < signaturePixels.length; i += 4) {
        signature.push(
          0.2126 * signaturePixels[i] +
            0.7152 * signaturePixels[i + 1] +
            0.0722 * signaturePixels[i + 2],
        );
      }

      return {
        imageWidth: image.width,
        imageHeight: image.height,
        sceneWidth,
        sceneHeight,
        sampleWidth,
        sampleHeight,
        mean,
        variance,
        nonDarkRatio,
        bucketCount: buckets.size,
        detailScore,
        signature,
      };
    }, base64);

  const stages = [
    {
      name: "near",
      zoomFactor: 0.28,
      steps: "focus=first_location;select=first_agent;zoom=0.28;wait=0.45",
      shot: "$zoom_shot_near",
    },
    {
      name: "mid",
      zoomFactor: 1.8,
      steps: "focus=first_location;select=first_agent;zoom=1.8;wait=0.45",
      shot: "$zoom_shot_mid",
    },
    {
      name: "far",
      zoomFactor: 4.2,
      steps: "focus=first_location;select=first_agent;zoom=4.2;wait=0.45",
      shot: "$zoom_shot_far",
    },
  ];

  const signatureDistance = (left, right) => {
    if (!Array.isArray(left) || !Array.isArray(right) || left.length === 0 || left.length !== right.length) {
      return 0;
    }
    let delta = 0;
    for (let index = 0; index < left.length; index += 1) {
      delta += Math.abs(left[index] - right[index]);
    }
    return delta / left.length;
  };

  const results = [];
  for (const stage of stages) {
    await page.reload({ waitUntil: "domcontentloaded" });
    await waitForApi();
    await waitForConnected(\`zoom stage \${stage.name} initial\`);
    await page.evaluate(() => window.__AW_TEST__.setMode("3d"));
    const modeDeadline = Date.now() + 10000;
    let modeState = await page.evaluate(() => window.__AW_TEST__.getState());
    while (Date.now() < modeDeadline) {
      const modeRadius = Number(modeState.cameraRadius || 0);
      if (
        modeState.connectionStatus === "connected" &&
        modeState.cameraMode === "3d" &&
        Number.isFinite(modeRadius) &&
        modeRadius >= 40
      ) {
        break;
      }
      await page.waitForTimeout(250);
      modeState = await page.evaluate(() => window.__AW_TEST__.getState());
    }
    if (modeState.connectionStatus !== "connected" || modeState.cameraMode !== "3d") {
      fail(
        \`zoom stage \${stage.name} failed to settle 3d mode (status=\${modeState.connectionStatus}, mode=\${modeState.cameraMode})\`,
      );
    }
    await page.evaluate(
      () => window.__AW_TEST__.runSteps("focus=first_location;select=first_agent;wait=0.35"),
    );
    await page.waitForTimeout(550);
    const beforeState = await page.evaluate(() => window.__AW_TEST__.getState());
    const beforeRadius = Number(beforeState.cameraRadius || 0);
    if (!Number.isFinite(beforeRadius) || beforeRadius <= 0) {
      fail(\`zoom stage \${stage.name} invalid baseline radius: \${beforeState.cameraRadius}\`);
    }
    await page.evaluate((steps) => window.__AW_TEST__.runSteps(steps), stage.steps);
    const radiusDeadline = Date.now() + 10000;
    let state = await waitForConnected(\`zoom stage \${stage.name} after steps\`);
    let cameraRadius = Number(state.cameraRadius || 0);
    while (Date.now() < radiusDeadline) {
      if (state.cameraMode === "3d") {
        if (
          (stage.zoomFactor < 1.0 && cameraRadius < beforeRadius * 0.9) ||
          (stage.zoomFactor > 1.0 && cameraRadius > beforeRadius * 1.1)
        ) {
          break;
        }
      }
      await page.waitForTimeout(250);
      state = await page.evaluate(() => window.__AW_TEST__.getState());
      cameraRadius = Number(state.cameraRadius || 0);
    }
    if (state.cameraMode !== "3d") {
      fail(\`zoom stage \${stage.name} expected 3d camera mode, got \${state.cameraMode}\`);
    }
    if (!Number.isFinite(cameraRadius) || cameraRadius <= 0) {
      fail(\`zoom stage \${stage.name} invalid camera radius: \${state.cameraRadius}\`);
    }
    if (stage.zoomFactor < 1.0 && cameraRadius >= beforeRadius * 0.9) {
      fail(
        \`zoom stage \${stage.name} radius did not shrink as expected (before=\${beforeRadius.toFixed(3)}, after=\${cameraRadius.toFixed(3)})\`,
      );
    }
    if (stage.zoomFactor > 1.0 && cameraRadius <= beforeRadius * 1.1) {
      fail(
        \`zoom stage \${stage.name} radius did not expand as expected (before=\${beforeRadius.toFixed(3)}, after=\${cameraRadius.toFixed(3)})\`,
      );
    }
    if (state.selectedKind !== "agent") {
      fail(\`zoom stage \${stage.name} missing agent selection: \${state.selectedKind}\`);
    }
    const screenshot = await page.screenshot({
      path: stage.shot,
      scale: "css",
      type: "png",
    });
    const metricsWithSignature = await screenshotMetrics(screenshot.toString("base64"));
    const { signature, ...metrics } = metricsWithSignature;
    if (metrics.nonDarkRatio < 0.003) {
      fail(\`zoom stage \${stage.name} non-dark ratio too low: \${metrics.nonDarkRatio}\`);
    }
    if (metrics.bucketCount < 8) {
      fail(\`zoom stage \${stage.name} color buckets too low: \${metrics.bucketCount}\`);
    }
    if (metrics.detailScore < 1.2) {
      fail(\`zoom stage \${stage.name} detail score too low: \${metrics.detailScore}\`);
    }
    results.push({
      stage: stage.name,
      shot: stage.shot,
      cameraRadius,
      metrics,
      signature,
    });
  }

  const detailScores = results.map((item) => item.metrics.detailScore);
  const minDetail = Math.min(...detailScores);
  const maxDetail = Math.max(...detailScores);
  if (!Number.isFinite(minDetail) || !Number.isFinite(maxDetail) || maxDetail <= 0) {
    fail(\`invalid detail scores across zoom stages: \${JSON.stringify(detailScores)}\`);
  }
  if (minDetail / maxDetail < 0.12) {
    fail(
      \`detail collapse across zoom stages (min=\${minDetail.toFixed(2)}, max=\${maxDetail.toFixed(2)})\`,
    );
  }

  const nearRadius = Number(results[0]?.cameraRadius || 0);
  const farRadius = Number(results[2]?.cameraRadius || 0);
  if (!(farRadius > nearRadius * 1.2)) {
    fail(
      \`camera radius did not expand across zoom stages (near=\${nearRadius.toFixed(3)}, far=\${farRadius.toFixed(3)})\`,
    );
  }

  const nearFarDelta = signatureDistance(results[0]?.signature, results[2]?.signature);
  if (nearFarDelta < 0.5) {
    fail(\`zoom visual delta too small between near/far stages: \${nearFarDelta.toFixed(3)}\`);
  }

  return results.map(({ signature, ...rest }) => rest);
}
JS
)
if ! zoom_output=$(bash "$PWCLI" run-code "$zoom_code" 2>&1); then
  zoom_ok=0
fi
if printf "%s\n" "$zoom_output" | rg -q "^### Error"; then
  zoom_ok=0
fi
printf "%s\n" "$zoom_output" | tee "$zoom_log" | tee -a "$pw_log" >/dev/null

console_output=$(bash "$PWCLI" console 2>&1 | tee -a "$pw_log")
printf "%s\n" "$console_output"
console_path=$(printf "%s\n" "$console_output" | sed -n 's/.*\[Console\](\([^)]*\)).*/\1/p' | tail -n 1)

bash "$PWCLI" screenshot --filename "$shot_path" | tee -a "$pw_log"
bash "$PWCLI" close | tee -a "$pw_log"

bevy_error_count=0
if [[ -n "$console_path" && -f "$console_path" ]]; then
  bevy_error_count=$(python3 - "$console_path" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8", errors="replace")
patterns = [r"\[ERROR\]", r"%cERROR%c"]
count = 0
for pattern in patterns:
    count += len(re.findall(pattern, text))
print(count)
PY
)
fi

screenshot_ok=0
if [[ -s "$shot_path" && -s "$zoom_shot_near" && -s "$zoom_shot_mid" && -s "$zoom_shot_far" ]]; then
  screenshot_ok=1
fi

visual_baseline_status="passed"
if [[ "$skip_visual_baseline" -eq 1 ]]; then
  visual_baseline_status="skipped"
fi

overall_pass=1
if [[ "$semantic_ok" -ne 1 ]]; then
  overall_pass=0
fi
if [[ "$zoom_ok" -ne 1 ]]; then
  overall_pass=0
fi
if [[ "$screenshot_ok" -ne 1 ]]; then
  overall_pass=0
fi
if [[ "$bevy_error_count" -gt 0 ]]; then
  overall_pass=0
fi

{
  echo "# Viewer Release QA Summary"
  echo ""
  echo "- Timestamp: $(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "- Scenario: \`$scenario\`"
  echo "- Viewer URL: \`$viewer_url\`"
  echo "- Visual baseline: $visual_baseline_status"
  if [[ "$semantic_ok" -eq 1 ]]; then
    echo "- Semantic web gate: passed"
  else
    echo "- Semantic web gate: failed"
  fi
  if [[ "$zoom_ok" -eq 1 ]]; then
    echo "- Zoom texture gate: passed"
  else
    echo "- Zoom texture gate: failed"
  fi
  if [[ "$screenshot_ok" -eq 1 ]]; then
    echo "- Screenshot artifact: passed"
  else
    echo "- Screenshot artifact: failed"
  fi
  echo "- Bevy \`[ERROR]\` logs in console dump: $bevy_error_count"
  if [[ "$overall_pass" -eq 1 ]]; then
    echo "- Overall: PASS"
  else
    echo "- Overall: FAIL"
  fi
  echo ""
  echo "## Artifacts"
  echo "- Live log: \`$live_log\`"
  echo "- Web log: \`$web_log\`"
  echo "- Playwright log: \`$pw_log\`"
  echo "- Semantic gate log: \`$semantic_log\`"
  echo "- Zoom gate log: \`$zoom_log\`"
  if [[ -n "$console_path" ]]; then
    echo "- Console dump: \`$console_path\`"
  else
    echo "- Console dump: unavailable"
  fi
  echo "- Screenshot: \`$shot_path\`"
  echo "- Zoom near screenshot: \`$zoom_shot_near\`"
  echo "- Zoom mid screenshot: \`$zoom_shot_mid\`"
  echo "- Zoom far screenshot: \`$zoom_shot_far\`"
} >"$summary_path"

echo "release qa summary: $summary_path"
if [[ "$overall_pass" -ne 1 ]]; then
  exit 1
fi
