# Shared Network Rollback Target

审计轮次: 1

## Meta
- `window_id`:
  - `shared-devnet-20260324-06`
- `track`:
  - `shared_devnet`
- `candidate_id`:
  - `shared-devnet-20260324-05`
- `owner`:
  - `liveops_community`

## Current Candidate
- `candidate_bundle_ref`:
  - `output/release-candidates/shared-devnet-20260324-05.json`
- `candidate_gate_ref`:
  - `output/shared-network/shared-devnet-20260324-06/gate/shared_devnet-20260324-175501/summary.md`

## Fallback Candidate
- `fallback_candidate_id`:
  - `<previous-pass-candidate-id>`
- `fallback_candidate_bundle_ref`:
  - `<output/release-candidates/fallback.json>`
- `fallback_gate_ref`:
  - `<output/shared-network/.../gate/.../summary.md>`
- `fallback_track_result`:
  - `pass`
- `fallback_owner_ref`:
  - `<promotion record | incident review | approval record>`

## Rollback Readiness
- `restore_steps_ref`:
  - `<pending>`
- `validated_by`:
  - `<liveops owner / runtime owner>`
- `validated_at`:
  - `<YYYY-MM-DD HH:MM:SS TZ>`
- `restoration_scope`:
  - `<runtime build | world snapshot | governance manifest>`

## Verdict
- `lane_result`:
  - `partial`
- `reason`:
  - formal previous shared-devnet pass fallback is not pinned yet

## Notes
- `pass` only if fallback candidate is a formal previous shared-devnet `pass` candidate.
- `partial` if there is only a local/provisional fallback but no formal shared-devnet `pass` history.
- `block` if fallback truth is missing, inconsistent, or not restorable.
