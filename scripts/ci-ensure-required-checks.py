#!/usr/bin/env python3
"""Ensure required status checks are configured for a GitHub branch protection."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

DEFAULT_BRANCH = "main"
DEFAULT_STRICT = True
DEFAULT_REQUIRED_CHECK = "Builtin Wasm m1 Multi Runner / verify-m1-multi-runner-summary"


def fail(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def run_cmd(
    args: list[str],
    *,
    capture: bool = True,
    check: bool = True,
    input_text: str | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        text=True,
        input=input_text,
        capture_output=capture,
        check=check,
    )


def require_cmd(name: str) -> None:
    if shutil.which(name) is None:
        fail(f"missing required command: {name}")


def parse_bool(raw: str) -> bool:
    if raw == "true":
        return True
    if raw == "false":
        return False
    fail(f"invalid boolean '{raw}', expected true|false")


def resolve_repo_from_origin() -> str:
    proc = run_cmd(["git", "config", "--get", "remote.origin.url"], check=False)
    remote = proc.stdout.strip()
    if not remote:
        fail("--repo not provided and origin remote is missing")

    if remote.startswith("git@github.com:"):
        suffix = remote.removeprefix("git@github.com:")
        if suffix.endswith(".git"):
            suffix = suffix[:-4]
        if "/" in suffix:
            return suffix

    if remote.startswith("https://github.com/"):
        suffix = remote.removeprefix("https://github.com/")
        if suffix.endswith(".git"):
            suffix = suffix[:-4]
        if "/" in suffix:
            return suffix

    fail(f"cannot parse owner/repo from origin remote: {remote}")


def unique_keep_order(items: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in items:
        if item and item not in seen:
            seen.add(item)
            out.append(item)
    return out


def extract_contexts(payload: dict[str, Any]) -> list[str]:
    checks = payload.get("checks")
    contexts: list[str] = []

    if isinstance(checks, list):
        for item in checks:
            if isinstance(item, dict):
                context = item.get("context")
                if isinstance(context, str) and context:
                    contexts.append(context)

    legacy = payload.get("contexts")
    if isinstance(legacy, list):
        for item in legacy:
            if isinstance(item, str) and item:
                contexts.append(item)

    return unique_keep_order(contexts)


def gh_api_json(path: str) -> dict[str, Any]:
    proc = run_cmd(["gh", "api", path])
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        fail(f"failed to parse JSON from gh api {path}: {exc}")


def gh_api_write(path: str, payload: dict[str, Any], method: str) -> None:
    with tempfile.NamedTemporaryFile("w", delete=False, encoding="utf-8") as fp:
        json.dump(payload, fp, ensure_ascii=True)
        fp.flush()
        payload_path = fp.name

    try:
        run_cmd(["gh", "api", "-X", method, path, "--input", payload_path])
    finally:
        Path(payload_path).unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default="", help="Target repository owner/name")
    parser.add_argument("--branch", default=DEFAULT_BRANCH, help="Target branch")
    parser.add_argument(
        "--check",
        action="append",
        default=[],
        help="Required status check context (repeatable)",
    )
    parser.add_argument(
        "--strict",
        default="true" if DEFAULT_STRICT else "false",
        help="Strict status check policy: true|false",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print payload only")
    args = parser.parse_args()

    require_cmd("gh")
    require_cmd("python3")

    strict = parse_bool(args.strict)
    repo = args.repo or resolve_repo_from_origin()

    requested = args.check[:] if args.check else [DEFAULT_REQUIRED_CHECK]
    requested = unique_keep_order(requested)
    if not requested:
        fail("requested checks cannot be empty")

    run_cmd(["gh", "auth", "status"])

    print(f"repo={repo}")
    print(f"branch={args.branch}")
    print(f"requested_checks={','.join(requested)}")

    api_root = f"repos/{repo}/branches/{args.branch}/protection"
    status_checks_api = f"{api_root}/required_status_checks"

    existing_required: dict[str, Any] | None = None
    probe = run_cmd(["gh", "api", status_checks_api], check=False)
    if probe.returncode == 0:
        try:
            existing_required = json.loads(probe.stdout)
        except json.JSONDecodeError as exc:
            fail(f"failed to parse required_status_checks JSON: {exc}")
        print("branch protection status: protected")
    else:
        combined_err = (probe.stderr or "") + (probe.stdout or "")
        if "Branch not protected" in combined_err:
            print("branch protection status: not protected")
        else:
            sys.stderr.write(probe.stderr)
            raise SystemExit(probe.returncode)

    if existing_required is None:
        create_payload: dict[str, Any] = {
            "required_status_checks": {
                "strict": strict,
                "checks": [{"context": c, "app_id": -1} for c in requested],
            },
            "enforce_admins": False,
            "required_pull_request_reviews": None,
            "restrictions": None,
            "required_linear_history": False,
            "allow_force_pushes": False,
            "allow_deletions": False,
            "block_creations": False,
            "required_conversation_resolution": False,
            "lock_branch": False,
            "allow_fork_syncing": False,
        }

        if args.dry_run:
            print("dry-run: would create branch protection payload:")
            print(json.dumps(create_payload, ensure_ascii=True, indent=2))
            print("dry-run complete")
            return

        gh_api_write(api_root, create_payload, method="PUT")
        print(f"created branch protection for {args.branch}")
    else:
        existing_contexts = extract_contexts(existing_required)
        merged = unique_keep_order(existing_contexts + requested)
        strict_existing = bool(existing_required.get("strict", False))

        patch_payload: dict[str, Any] = {
            "strict": strict_existing,
            "checks": [{"context": c, "app_id": -1} for c in merged],
        }

        if args.dry_run:
            print("dry-run: would patch required_status_checks payload:")
            print(json.dumps(patch_payload, ensure_ascii=True, indent=2))
            print("dry-run complete")
            return

        gh_api_write(status_checks_api, patch_payload, method="PATCH")
        print(f"patched required_status_checks for {args.branch}")

    verify = gh_api_json(status_checks_api)
    verify_contexts = extract_contexts(verify)
    missing = [c for c in requested if c not in verify_contexts]

    if missing:
        fail(f"required checks missing after apply: {','.join(missing)}")

    print(f"required checks verified: {','.join(requested)}")
    print(f"effective checks: {','.join(verify_contexts)}")


if __name__ == "__main__":
    main()
