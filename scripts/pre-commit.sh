#!/usr/bin/env bash
set -euo pipefail

# Run the same integration tests that CI executes for viewer servers.
env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration
env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_offline_integration
