#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<'USAGE'
Usage: ./scripts/llm-switch-coverage-diff.sh --log <run.log> --switch-tick <n> [--out <path>]

Compute action coverage before/after a prompt switch tick from world_llm_agent_demo run log.

Options:
  --log <path>          Path to run log file (required)
  --switch-tick <n>     Prompt switch tick (1-based, required)
  --out <path>          Optional output file (default: stdout)
  -h, --help            Show help
USAGE
}

ensure_positive_int() {
  local name=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" == "0" ]]; then
    echo "invalid integer for $name: $value" >&2
    exit 2
  fi
}

format_counts_inline() {
  local file_path=$1
  if [[ ! -s "$file_path" ]]; then
    echo "none"
    return 0
  fi
  awk -F'\t' '
    BEGIN { sep="" }
    {
      printf "%s%s:%s", sep, $1, $2
      sep=","
    }
    END {
      if (NR == 0) {
        printf "none"
      }
    }
  ' "$file_path"
}

list_or_none() {
  local file_path=$1
  if [[ ! -s "$file_path" ]]; then
    echo "none"
    return 0
  fi
  paste -sd, "$file_path"
}

log_file=""
switch_tick=""
out_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --log)
      log_file=${2:-}
      shift 2
      ;;
    --switch-tick)
      switch_tick=${2:-}
      shift 2
      ;;
    --out)
      out_file=${2:-}
      shift 2
      ;;
    -h|--help)
      print_help
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      print_help
      exit 2
      ;;
  esac
done

if [[ -z "$log_file" || -z "$switch_tick" ]]; then
  print_help
  exit 2
fi

ensure_positive_int "--switch-tick" "$switch_tick"

if [[ ! -f "$log_file" ]]; then
  echo "run log not found: $log_file" >&2
  exit 3
fi

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

pre_counts="$tmp_dir/pre.tsv"
post_counts="$tmp_dir/post.tsv"
pre_kinds="$tmp_dir/pre_kinds.txt"
post_kinds="$tmp_dir/post_kinds.txt"
new_after_file="$tmp_dir/new_after.txt"
dropped_after_file="$tmp_dir/dropped_after.txt"

awk -v switch_tick="$switch_tick" -v pre_out="$pre_counts" -v post_out="$post_counts" '
  function kind_to_snake(raw, i, ch, out) {
    out = ""
    for (i = 1; i <= length(raw); i++) {
      ch = substr(raw, i, 1)
      if (ch ~ /[A-Z]/) {
        if (i > 1) {
          out = out "_"
        }
        out = out tolower(ch)
      } else {
        out = out ch
      }
    }
    return out
  }
  {
    if ($0 ~ /^tick=[0-9]+/ && $0 ~ / action=[A-Za-z0-9_]+/) {
      line = $0
      sub(/^tick=/, "", line)
      split(line, tick_parts, /[^0-9]/)
      tick = tick_parts[1] + 0

      action_part = $0
      sub(/^.* action=/, "", action_part)
      split(action_part, action_parts, /[^A-Za-z0-9_]/)
      kind = kind_to_snake(action_parts[1])
      if (tick < switch_tick) {
        pre[kind]++
      } else {
        post[kind]++
      }
    }
  }
  END {
    for (kind in pre) {
      printf "%s\t%d\n", kind, pre[kind] >> pre_out
    }
    for (kind in post) {
      printf "%s\t%d\n", kind, post[kind] >> post_out
    }
  }
' "$log_file"

if [[ -f "$pre_counts" ]]; then
  sort -k1,1 "$pre_counts" -o "$pre_counts"
fi
if [[ -f "$post_counts" ]]; then
  sort -k1,1 "$post_counts" -o "$post_counts"
fi

if [[ -f "$pre_counts" ]]; then
  cut -f1 "$pre_counts" >"$pre_kinds"
else
  : >"$pre_kinds"
fi
if [[ -f "$post_counts" ]]; then
  cut -f1 "$post_counts" >"$post_kinds"
else
  : >"$post_kinds"
fi

comm -13 "$pre_kinds" "$post_kinds" >"$new_after_file" || true
comm -23 "$pre_kinds" "$post_kinds" >"$dropped_after_file" || true

pre_total=$(wc -l <"$pre_kinds" | tr -d ' ')
post_total=$(wc -l <"$post_kinds" | tr -d ' ')

pre_counts_inline=$(format_counts_inline "$pre_counts")
post_counts_inline=$(format_counts_inline "$post_counts")
new_after=$(list_or_none "$new_after_file")
dropped_after=$(list_or_none "$dropped_after_file")

{
  echo "log_file=$log_file"
  echo "switch_tick=$switch_tick"
  echo "pre_action_kinds_total=$pre_total"
  echo "pre_action_kind_counts=$pre_counts_inline"
  echo "post_action_kinds_total=$post_total"
  echo "post_action_kind_counts=$post_counts_inline"
  echo "new_after_switch=$new_after"
  echo "dropped_after_switch=$dropped_after"
} >"$tmp_dir/output.txt"

if [[ -n "$out_file" ]]; then
  mkdir -p "$(dirname "$out_file")"
  cp "$tmp_dir/output.txt" "$out_file"
else
  cat "$tmp_dir/output.txt"
fi
