#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../../.." && pwd)"
script_path="$script_dir/oasis7-run.sh"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

fake_bin="$tmp_dir/bin"
bundle_dir="$tmp_dir/bundle"
child_pid_file="$tmp_dir/child.pid"
child_ready_file="$tmp_dir/child.ready"
wrapper_stdout="$tmp_dir/wrapper.stdout"
wrapper_stderr="$tmp_dir/wrapper.stderr"
mkdir -p "$fake_bin" "$bundle_dir"

cat > "$bundle_dir/run-game.sh" <<'RUN'
#!/usr/bin/env bash
set -euo pipefail
: "${OASIS7_CHILD_PID_FILE:?}"
: "${OASIS7_CHILD_READY_FILE:?}"
bash -c 'trap "" TERM INT HUP; while true; do sleep 1; done' &
child_pid="$!"
printf '%s\n' "$child_pid" > "$OASIS7_CHILD_PID_FILE"
: > "$OASIS7_CHILD_READY_FILE"
wait "$child_pid"
RUN
chmod +x "$bundle_dir/run-game.sh"

cat > "$fake_bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
url="${@: -1}"
case "$url" in
  http://127.0.0.1:5841/v1/provider/health)
    printf '{"status":"ok"}\n'
    ;;
  *)
    echo "unexpected curl url: $url" >&2
    exit 22
    ;;
esac
CURL
chmod +x "$fake_bin/curl"

sanitized_path="$fake_bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
(
  cd "$repo_root"
  export PATH="$sanitized_path"
  export OASIS7_CHILD_PID_FILE="$child_pid_file"
  export OASIS7_CHILD_READY_FILE="$child_ready_file"
  exec bash "$script_path" play     --bundle-dir "$bundle_dir"     --reuse-bridge     --skip-agent-setup     --no-open-browser
) >"$wrapper_stdout" 2>"$wrapper_stderr" &
wrapper_pid="$!"

for _ in $(seq 1 50); do
  if [[ -f "$child_ready_file" && -s "$child_pid_file" ]]; then
    break
  fi
  sleep 0.1
done

if [[ ! -f "$child_ready_file" || ! -s "$child_pid_file" ]]; then
  echo "child process never became ready" >&2
  cat "$wrapper_stdout" >&2 || true
  cat "$wrapper_stderr" >&2 || true
  kill "$wrapper_pid" >/dev/null 2>&1 || true
  wait "$wrapper_pid" >/dev/null 2>&1 || true
  exit 1
fi

child_pid="$(cat "$child_pid_file")"
if ! kill -0 "$child_pid" >/dev/null 2>&1; then
  echo "expected child process to be alive before wrapper termination" >&2
  exit 1
fi

kill "$wrapper_pid" >/dev/null 2>&1 || true
wait "$wrapper_pid" >/dev/null 2>&1 || true

for _ in $(seq 1 50); do
  if ! kill -0 "$child_pid" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

if kill -0 "$child_pid" >/dev/null 2>&1; then
  echo "child process still alive after wrapper termination: $child_pid" >&2
  cat "$wrapper_stdout" >&2 || true
  cat "$wrapper_stderr" >&2 || true
  kill -9 "$child_pid" >/dev/null 2>&1 || true
  exit 1
fi

echo "oasis7-run shutdown tests passed"
