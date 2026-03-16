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
mkdir -p "$fake_bin"
archive_payload="$tmp_dir/archive.payload"
cache_dir="$tmp_dir/cache"
mkdir -p "$cache_dir"
printf 'fake oasis7 archive payload\n' > "$archive_payload"
archive_sha256="$(sha256sum "$archive_payload" | awk '{print $1}')"

cat > "$fake_bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
output_path=""
args=("$@")
for ((i=0; i<${#args[@]}; i+=1)); do
  if [[ "${args[$i]}" == "-o" ]]; then
    output_path="${args[$((i + 1))]}"
    break
  fi
done
[[ -n "$output_path" ]] || { echo "missing -o output path" >&2; exit 2; }
url="${args[-1]}"
case "$url" in
  https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-linux-x64.tar.gz)
    sleep 2
    cp "$FAKE_ARCHIVE_PAYLOAD" "$output_path"
    ;;
  https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-checksums.txt)
    printf '%s  agent-world-linux-x64.tar.gz\n' "$FAKE_ARCHIVE_SHA256" > "$output_path"
    ;;
  *)
    echo "unexpected curl url: $url" >&2
    exit 22
    ;;
esac
CURL
chmod +x "$fake_bin/curl"

cat > "$fake_bin/tar" <<'TAR'
#!/usr/bin/env bash
set -euo pipefail
extract_root=""
args=("$@")
for ((i=0; i<${#args[@]}; i+=1)); do
  if [[ "${args[$i]}" == "-C" ]]; then
    extract_root="${args[$((i + 1))]}"
    break
  fi
done
[[ -n "$extract_root" ]] || { echo "missing tar extract root" >&2; exit 2; }
mkdir -p "$extract_root/agent-world-linux-x64/bin"
cat > "$extract_root/agent-world-linux-x64/run-game.sh" <<'RUN'
#!/usr/bin/env bash
exit 0
RUN
chmod +x "$extract_root/agent-world-linux-x64/run-game.sh"
TAR
chmod +x "$fake_bin/tar"

sanitized_path="$fake_bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
first_stderr="$tmp_dir/download.stderr"
first_stdout="$tmp_dir/download.stdout"
(
  cd "$repo_root"
  PATH="$sanitized_path" \
  FAKE_ARCHIVE_PAYLOAD="$archive_payload" \
  FAKE_ARCHIVE_SHA256="$archive_sha256" \
  OASIS7_DOWNLOAD_HEARTBEAT_SECS=1 \
  bash "$script_path" download --download-dir "$cache_dir" >"$first_stdout" 2>"$first_stderr"
)

bundle_dir="$(tr -d '\n' < "$first_stdout")"
expected_bundle="$cache_dir/eng-cc-agent-world/latest/linux-x64/bundle"
if [[ "$bundle_dir" != "$expected_bundle" ]]; then
  echo "expected bundle dir '$expected_bundle', got '$bundle_dir'" >&2
  exit 1
fi
[[ -x "$bundle_dir/run-game.sh" ]] || { echo "bundle missing run-game.sh" >&2; exit 1; }

for needle in \
  "Downloading release asset:" \
  "Downloading release asset… (elapsed=1s)" \
  "Downloaded archive:" \
  "Fetching release checksums:" \
  "Verified SHA256:" \
  "Extracting bundle archive into:" \
  "Preparing bundle directory:" \
  "Bundle ready:"; do
  if ! grep -Fq "$needle" "$first_stderr"; then
    echo "missing expected log line: $needle" >&2
    cat "$first_stderr" >&2
    exit 1
  fi
done

second_stderr="$tmp_dir/reuse.stderr"
second_stdout="$tmp_dir/reuse.stdout"
(
  cd "$repo_root"
  PATH="$sanitized_path" \
  bash "$script_path" download --download-dir "$cache_dir" >"$second_stdout" 2>"$second_stderr"
)
if ! grep -Fq "Reusing cached release bundle:" "$second_stderr"; then
  echo "missing cached bundle reuse log" >&2
  cat "$second_stderr" >&2
  exit 1
fi
if [[ "$(tr -d '\n' < "$second_stdout")" != "$expected_bundle" ]]; then
  echo "cached bundle stdout mismatch" >&2
  exit 1
fi

echo "oasis7-run download tests passed"
