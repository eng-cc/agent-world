#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SITE_ROOT="${REPO_ROOT}/site"

if [[ ! -d "${SITE_ROOT}" ]]; then
  echo "error: site root not found: ${SITE_ROOT}" >&2
  exit 1
fi

fail_count=0

while IFS= read -r html_file; do
  while IFS= read -r attr; do
    ref="${attr#*=}"
    ref="${ref#\"}"
    ref="${ref%\"}"

    case "${ref}" in
      ""|\#*|http:*|https:*|mailto:*|javascript:*|tel:*)
        continue
        ;;
    esac

    clean="${ref%%\#*}"
    clean="${clean%%\?*}"
    if [[ -z "${clean}" ]]; then
      continue
    fi

    target="$(realpath -m "$(dirname "${html_file}")/${clean}")"
    if [[ ! -e "${target}" ]]; then
      echo "error: broken local reference in ${html_file}: ${ref} -> ${target}" >&2
      fail_count=$((fail_count + 1))
    fi
  done < <(grep -Eo '(href|src)="[^"]+"' "${html_file}" || true)
done < <(find "${SITE_ROOT}" -type f -name '*.html' | sort)

if [[ "${fail_count}" -gt 0 ]]; then
  echo "error: site link check failed with ${fail_count} broken local reference(s)" >&2
  exit 1
fi

echo "ok: site local href/src references are valid"
