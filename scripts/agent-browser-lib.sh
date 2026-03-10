#!/usr/bin/env bash

require_cmd() {
  local cmd=$1
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: missing required command: $cmd" >&2
    exit 1
  fi
}

ab_require() {
  require_cmd agent-browser
  require_cmd python3
}

ab_cmd() {
  local session=$1
  shift
  AGENT_BROWSER_SESSION="$session" agent-browser "$@"
}

ab_open() {
  local session=$1
  local headed=$2
  local url=$3
  if [[ "$headed" -eq 1 ]]; then
    AGENT_BROWSER_SESSION="$session" agent-browser --headed open "$url"
  else
    AGENT_BROWSER_SESSION="$session" agent-browser open "$url"
  fi
}

ab_eval() {
  local session=$1
  local script=$2
  AGENT_BROWSER_SESSION="$session" agent-browser eval --stdin <<<"$script"
}

json_quote() {
  python3 - "$1" <<'PY'
import json
import sys
print(json.dumps(sys.argv[1]))
PY
}

json_get() {
  python3 - "$1" "$2" <<'PY'
import json
import sys

raw = sys.argv[1]
path = sys.argv[2].split('.') if sys.argv[2] else []
try:
    value = json.loads(raw)
except Exception:
    print("")
    raise SystemExit(0)
for part in path:
    if isinstance(value, dict):
        value = value.get(part)
    else:
        value = None
        break
if value is None:
    print("")
elif isinstance(value, bool):
    print("true" if value else "false")
elif isinstance(value, (dict, list)):
    print(json.dumps(value, ensure_ascii=False))
else:
    print(value)
PY
}

json_to_file() {
  local raw_json=$1
  local out_path=$2
  python3 - "$raw_json" "$out_path" <<'PY'
import json
import pathlib
import sys

raw = sys.argv[1]
out = pathlib.Path(sys.argv[2])
try:
    data = json.loads(raw)
except Exception:
    out.write_text(raw + "\n", encoding="utf-8")
else:
    out.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
}


resolve_viewer_static_dir_for_web_closure() {
  local repo_root=$1
  local requested_dir=$2
  local out_dir=$3

  if [[ "$requested_dir" != "web" ]]; then
    printf '%s
' "$requested_dir"
    return 0
  fi

  local dist_dir="$repo_root/crates/agent_world_viewer/dist"
  local dist_index="$dist_dir/index.html"
  local newest_source=0
  local dist_mtime=0
  local rebuilt_dir

  if [[ "$out_dir" = /* ]]; then
    rebuilt_dir="$out_dir/web-dist"
  else
    rebuilt_dir="$repo_root/$out_dir/web-dist"
  fi

  if [[ -f "$dist_index" ]]; then
    dist_mtime=$(stat -c %Y "$dist_index" 2>/dev/null || echo 0)
  fi

  newest_source=$(find     "$repo_root/crates/agent_world_viewer/src"     "$repo_root/crates/agent_world_proto/src"     -type f \( -name '*.rs' -o -name '*.html' -o -name 'Trunk.toml' -o -name 'Cargo.toml' \)     -printf '%T@
' 2>/dev/null | sort -nr | head -n 1 | cut -d. -f1)
  newest_source=${newest_source:-0}

  if [[ -f "$dist_index" && "$dist_mtime" -ge "$newest_source" ]]; then
    printf '%s
' "$dist_dir"
    return 0
  fi

  require_cmd trunk
  mkdir -p "$rebuilt_dir"
  echo "+ env -u NO_COLOR trunk build --dist $rebuilt_dir" >&2
  (
    cd "$repo_root/crates/agent_world_viewer"
    env -u NO_COLOR trunk build --dist "$rebuilt_dir"
  ) >&2
  printf '%s
' "$rebuilt_dir"
}
