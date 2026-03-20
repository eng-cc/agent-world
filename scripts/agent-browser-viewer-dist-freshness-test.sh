#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/agent-browser-lib.sh"

tmp_repo="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_repo"
}
trap cleanup EXIT

mkdir -p \
  "$tmp_repo/bin" \
  "$tmp_repo/crates/oasis7_viewer/dist" \
  "$tmp_repo/crates/oasis7_viewer/src" \
  "$tmp_repo/crates/oasis7_viewer/assets" \
  "$tmp_repo/crates/oasis7_proto/src"

printf '<!doctype html>old dist\n' > "$tmp_repo/crates/oasis7_viewer/dist/index.html"
printf 'console.log("safe mode changed");\n' > "$tmp_repo/crates/oasis7_viewer/software_safe.js"
printf '<!doctype html>viewer\n' > "$tmp_repo/crates/oasis7_viewer/index.html"
printf '<!doctype html>software safe\n' > "$tmp_repo/crates/oasis7_viewer/software_safe.html"
printf '[package]\nname = "oasis7_viewer"\nversion = "0.0.0"\n' > "$tmp_repo/crates/oasis7_viewer/Cargo.toml"
printf '[build]\ntarget = "dist"\n' > "$tmp_repo/crates/oasis7_viewer/Trunk.toml"
printf '[package]\nname = "oasis7_proto"\nversion = "0.0.0"\n' > "$tmp_repo/crates/oasis7_proto/Cargo.toml"
printf 'pub const VIEWER_PROTOCOL_VERSION: u32 = 1;\n' > "$tmp_repo/crates/oasis7_proto/src/viewer.rs"
printf '# lock\n' > "$tmp_repo/Cargo.lock"

touch -d '2026-03-16 00:00:00' "$tmp_repo/crates/oasis7_viewer/dist/index.html"
touch -d '2026-03-17 00:00:00' "$tmp_repo/crates/oasis7_viewer/software_safe.js"

cat > "$tmp_repo/bin/trunk" <<'TRUNK'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" != "build" || "$2" != "--dist" ]]; then
  echo "unexpected trunk args: $*" >&2
  exit 1
fi
mkdir -p "$3"
printf '<!doctype html>rebuilt dist\n' > "$3/index.html"
printf '<!doctype html>rebuilt safe\n' > "$3/software_safe.html"
TRUNK
chmod +x "$tmp_repo/bin/trunk"

resolved_dir="$({ PATH="$tmp_repo/bin:$PATH" resolve_viewer_static_dir_for_web_closure "$tmp_repo" web "$tmp_repo/output/check"; } 2>"$tmp_repo/stderr.log")"
expected_dir="$tmp_repo/output/check/web-dist"

if [[ "$resolved_dir" != "$expected_dir" ]]; then
  echo "expected rebuilt dir '$expected_dir', got '$resolved_dir'" >&2
  exit 1
fi

if [[ ! -f "$expected_dir/index.html" ]]; then
  echo "expected rebuilt dist index at $expected_dir/index.html" >&2
  exit 1
fi

if ! grep -Fq 'trunk build --dist' "$tmp_repo/stderr.log"; then
  echo "expected freshness helper to trigger trunk rebuild" >&2
  cat "$tmp_repo/stderr.log" >&2
  exit 1
fi

echo "agent-browser viewer dist freshness tests passed"
