#!/usr/bin/env bash
# Cross-platform test contract: verify Windows shim rejection while preserving Linux/macOS interpreter discovery.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if [[ -n "${OASIS7_TEST_PYTHON:-}" ]]; then
  REAL_PYTHON="$OASIS7_TEST_PYTHON"
else
  REAL_PYTHON="$("$ROOT_DIR/scripts/pm/find-python-with-module.sh" ast)"
fi

if [[ ! -x "$REAL_PYTHON" ]] || ! "$REAL_PYTHON" -c 'import ast; print("ready")' | grep -Fxq ready; then
  echo "find-python-with-module.test: OASIS7_TEST_PYTHON or PATH discovery must provide a functional Python interpreter" >&2
  exit 1
fi

mkdir -p "$TMPDIR/broken-bin" "$TMPDIR/working-bin"
for name in python python3; do
  cat >"$TMPDIR/broken-bin/$name" <<'SH'
#!/usr/bin/env bash
# Simulates the Windows user-level python shim that exits 0 without executing input.
exit 0
SH
  chmod +x "$TMPDIR/broken-bin/$name"
done
for utility in bash tr dirname basename; do
  ln -s "$(command -v "$utility")" "$TMPDIR/broken-bin/$utility"
done
ln -s "$REAL_PYTHON" "$TMPDIR/working-bin/python42"
generic_home="$TMPDIR/no-bundled-home"
mkdir -p "$generic_home"

selected="$(HOME="$generic_home" PATH="$TMPDIR/broken-bin:$TMPDIR/working-bin:$PATH" \
  "$ROOT_DIR/scripts/pm/find-python-with-module.sh" ast)"
if [[ "$selected" != "$TMPDIR/working-bin/python42" ]]; then
  echo "find-python-with-module.test: generic discovery did not select python42: $selected" >&2
  exit 1
fi
if ! "$selected" -c 'import ast; print("selected")' | grep -Fxq selected; then
  echo "find-python-with-module.test: selected interpreter cannot execute Python" >&2
  exit 1
fi

# The Codex bundled runtime uses a Unix bin/python3 layout.  Isolate HOME and
# PATH so this contract exercises the bundled candidate rather than a host
# installation or the generic future-python shim above.
bundled_home="$TMPDIR/bundled-home"
bundled_python_dir="$bundled_home/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin"
mkdir -p "$bundled_python_dir"
ln -s "$REAL_PYTHON" "$bundled_python_dir/python3"
bundled_selected="$(HOME="$bundled_home" PATH="$TMPDIR/broken-bin" \
  "$ROOT_DIR/scripts/pm/find-python-with-module.sh" ast)"
if [[ "$bundled_selected" != "$bundled_python_dir/python3" ]]; then
  echo "find-python-with-module.test: did not select Unix bundled runtime: $bundled_selected" >&2
  exit 1
fi

echo "find-python-with-module.test: OK"
