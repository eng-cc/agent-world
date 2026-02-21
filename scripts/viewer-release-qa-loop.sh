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
summary_path="$out_dir/release-qa-summary-${stamp}.md"
shot_path="$out_dir/release-qa-${stamp}.png"

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

echo "+ env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- $scenario --bind $live_bind --web-bind $web_bind --tick-ms $tick_ms"
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- "$scenario" --bind "$live_bind" --web-bind "$web_bind" --tick-ms "$tick_ms" >"$live_log" 2>&1 &
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
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const ready = await page.evaluate(() => typeof window.__AW_TEST__ === "object");
    if (ready) break;
    await page.waitForTimeout(200);
  }
  const hasApi = await page.evaluate(() => typeof window.__AW_TEST__ === "object");
  if (!hasApi) fail("__AW_TEST__ is unavailable");

  const initial = await page.evaluate(() => window.__AW_TEST__.getState());
  if (!initial || typeof initial.connectionStatus !== "string" || typeof initial.tick !== "number") {
    fail("getState() missing required fields");
  }

  const connectedDeadline = Date.now() + 15000;
  let connectedState = initial;
  while (Date.now() < connectedDeadline) {
    connectedState = await page.evaluate(() => window.__AW_TEST__.getState());
    if (connectedState.connectionStatus === "connected") break;
    await page.waitForTimeout(250);
  }
  if (connectedState.connectionStatus !== "connected") {
    fail(`viewer not connected, status=${connectedState.connectionStatus}`);
  }

  const controlBefore = await page.evaluate(() => window.__AW_TEST__.getState());
  await page.evaluate(() => window.__AW_TEST__.sendControl("play"));
  await page.waitForTimeout(400);
  await page.evaluate(() => window.__AW_TEST__.sendControl("pause"));
  await page.waitForTimeout(200);

  await page.evaluate(() => window.__AW_TEST__.runSteps("mode=3d;focus=first_location;zoom=0.85;select=first_agent;wait=0.3"));
  await page.waitForTimeout(500);
  const selected = await page.evaluate(() => window.__AW_TEST__.getState());
  if (selected.selectedKind !== "agent") {
    fail(`selection did not resolve to agent (selectedKind=${selected.selectedKind})`);
  }

  const seekTick = Math.max(1, Number(controlBefore.tick || 0));
  await page.evaluate((tick) => window.__AW_TEST__.sendControl("seek", { tick }), seekTick);
  await page.waitForTimeout(300);
  const controlAfter = await page.evaluate(() => window.__AW_TEST__.getState());
  if (controlAfter.connectionStatus !== "connected") {
    fail(`connection dropped after controls, status=${controlAfter.connectionStatus}`);
  }
  if (Number(controlAfter.errorCount || 0) > Number(controlBefore.errorCount || 0)) {
    fail(
      `errorCount increased after controls (before=${controlBefore.errorCount}, after=${controlAfter.errorCount})`,
    );
  }

  return {
    initial,
    controlBefore,
    controlAfter,
    selected,
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
if [[ -s "$shot_path" ]]; then
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
  if [[ -n "$console_path" ]]; then
    echo "- Console dump: \`$console_path\`"
  else
    echo "- Console dump: unavailable"
  fi
  echo "- Screenshot: \`$shot_path\`"
} >"$summary_path"

echo "release qa summary: $summary_path"
if [[ "$overall_pass" -ne 1 ]]; then
  exit 1
fi
