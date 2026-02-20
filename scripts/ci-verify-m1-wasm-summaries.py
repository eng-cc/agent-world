#!/usr/bin/env python3
import argparse
import json
import pathlib
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify m1 builtin wasm summaries collected from multi-runner CI jobs."
    )
    parser.add_argument(
        "--summary-dir",
        required=True,
        help="Directory containing per-runner summary JSON files.",
    )
    parser.add_argument(
        "--expected-runners",
        default="linux-x86_64,darwin-arm64",
        help="Comma-separated runner labels expected to be present.",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_summary(path: pathlib.Path) -> dict:
    try:
        payload = json.loads(path.read_text())
    except Exception as exc:  # noqa: BLE001
        fail(f"failed to parse summary {path}: {exc}")

    required_keys = {
        "schema_version",
        "runner",
        "current_platform",
        "module_count",
        "module_hashes",
        "manifest_platform_hashes",
        "identity_hashes",
    }
    missing = sorted(required_keys - set(payload.keys()))
    if missing:
        fail(f"summary {path} missing required keys: {missing}")
    return payload


def verify_summary_shape(path: pathlib.Path, payload: dict) -> None:
    if payload["schema_version"] != 1:
        fail(f"summary {path} schema_version must be 1")

    for key in ("module_hashes", "manifest_platform_hashes", "identity_hashes"):
        if not isinstance(payload[key], dict):
            fail(f"summary {path} field {key} must be an object")

    module_hashes = payload["module_hashes"]
    manifest_hashes = payload["manifest_platform_hashes"]
    identity_hashes = payload["identity_hashes"]

    if len(module_hashes) != payload["module_count"]:
        fail(
            f"summary {path} module_count mismatch: declared={payload['module_count']} actual={len(module_hashes)}"
        )

    if set(module_hashes.keys()) != set(manifest_hashes.keys()):
        fail(f"summary {path} module set mismatch between module_hashes and manifest_platform_hashes")

    if set(module_hashes.keys()) != set(identity_hashes.keys()):
        fail(f"summary {path} module set mismatch between module_hashes and identity_hashes")

    for module_id, module_hash in module_hashes.items():
        expected_hash = manifest_hashes[module_id]
        if module_hash != expected_hash:
            fail(
                "summary {} module {} hash mismatch built={} manifest={}".format(
                    path, module_id, module_hash, expected_hash
                )
            )


def main() -> None:
    args = parse_args()
    summary_dir = pathlib.Path(args.summary_dir)
    if not summary_dir.exists():
        fail(f"summary dir does not exist: {summary_dir}")

    expected_runners = {
        value.strip() for value in args.expected_runners.split(",") if value.strip()
    }
    if not expected_runners:
        fail("--expected-runners has no valid entries")

    summary_paths = sorted(summary_dir.glob("*.json"))
    if not summary_paths:
        fail(f"no summary json files found in {summary_dir}")

    summaries_by_runner = {}
    for path in summary_paths:
        payload = load_summary(path)
        verify_summary_shape(path, payload)
        runner = payload["runner"]
        if runner in summaries_by_runner:
            fail(f"duplicate runner summary detected for {runner}")
        summaries_by_runner[runner] = payload

    found_runners = set(summaries_by_runner.keys())
    missing_runners = sorted(expected_runners - found_runners)
    extra_runners = sorted(found_runners - expected_runners)
    if missing_runners:
        fail(f"missing expected runner summaries: {missing_runners}")
    if extra_runners:
        fail(f"found unexpected runner summaries: {extra_runners}")

    baseline_runner = sorted(found_runners)[0]
    baseline = summaries_by_runner[baseline_runner]
    baseline_module_keys = set(baseline["module_hashes"].keys())
    baseline_identity_hashes = baseline["identity_hashes"]

    for runner in sorted(found_runners):
        payload = summaries_by_runner[runner]
        module_keys = set(payload["module_hashes"].keys())
        if module_keys != baseline_module_keys:
            fail(
                f"module key mismatch between runners baseline={baseline_runner} runner={runner}"
            )

        if payload["identity_hashes"] != baseline_identity_hashes:
            fail(
                f"identity hash mismatch between runners baseline={baseline_runner} runner={runner}"
            )

    print(
        "m1 multi-runner summary verify ok: runners={} module_count={}".format(
            ",".join(sorted(found_runners)), len(baseline_module_keys)
        )
    )


if __name__ == "__main__":
    main()
