#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/submit-module-release-attestation.sh [options]

Purpose:
  Submit a prepared `submit_request.json` to the node-side
  `/v1/chain/module-release/attestation/submit` endpoint.

Options:
  --request-json <path>   Prepared submit_request.json path (required)
  --base-url <url>        Chain runtime base URL (default: http://127.0.0.1:5121)
  --out <path>            Optional response output path
  --dry-run               Print curl command only
  -h, --help              Show help
USAGE
}

request_json=""
base_url="http://127.0.0.1:5121"
out_path=""
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --request-json)
      request_json=${2:-}
      shift 2
      ;;
    --base-url)
      base_url=${2:-}
      shift 2
      ;;
    --out)
      out_path=${2:-}
      shift 2
      ;;
    --dry-run)
      dry_run=1
      shift
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

if [[ -z "$request_json" ]]; then
  echo "error: --request-json is required" >&2
  usage >&2
  exit 2
fi

if [[ ! -f "$request_json" ]]; then
  echo "error: request json not found: $request_json" >&2
  exit 2
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "error: curl is required" >&2
  exit 2
fi

endpoint="${base_url%/}/v1/chain/module-release/attestation/submit"
curl_cmd=(
  curl
  -sS
  -X POST
  -H "Content-Type: application/json"
  --data-binary "@$request_json"
  "$endpoint"
)

if [[ "$dry_run" -eq 1 ]]; then
  printf '%q ' "${curl_cmd[@]}"
  printf '\n'
  exit 0
fi

response="$("${curl_cmd[@]}")"
if [[ -n "$out_path" ]]; then
  mkdir -p "$(dirname "$out_path")"
  printf '%s\n' "$response" > "$out_path"
fi
printf '%s\n' "$response"
