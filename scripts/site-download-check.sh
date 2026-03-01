#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SITE_ENTRIES=(
  "${REPO_ROOT}/site/index.html"
  "${REPO_ROOT}/site/en/index.html"
)

RELEASE_ASSET_URLS=(
  "https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-windows-x64.zip"
  "https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-macos-x64.tar.gz"
  "https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-linux-x64.tar.gz"
  "https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-checksums.txt"
)

for entry in "${SITE_ENTRIES[@]}"; do
  [[ -f "${entry}" ]] || { echo "error: missing site entry: ${entry}" >&2; exit 1; }

  for url in "${RELEASE_ASSET_URLS[@]}"; do
    if ! rg -Fq -- "${url}" "${entry}"; then
      echo "error: missing release asset url in ${entry}: ${url}" >&2
      exit 1
    fi
  done

  if ! rg -Fq -- "data-release-tag" "${entry}"; then
    echo "error: missing data-release-tag in ${entry}" >&2
    exit 1
  fi
  if ! rg -Fq -- "data-release-date" "${entry}"; then
    echo "error: missing data-release-date in ${entry}" >&2
    exit 1
  fi
  if ! rg -Fq -- "data-release-notes-link" "${entry}"; then
    echo "error: missing data-release-notes-link in ${entry}" >&2
    exit 1
  fi
done

if ! rg -Fq -- "https://api.github.com/repos/eng-cc/agent-world/releases/latest" "${REPO_ROOT}/site/assets/app.js"; then
  echo "error: missing latest release api endpoint in site/assets/app.js" >&2
  exit 1
fi

echo "ok: site download entry and release links are present"
