# Shared Network Rollback Target Template

审计轮次: 1

## Meta
- `window_id`:
  - `<shared-devnet-window-id>`
- `track`:
  - `shared_devnet`
- `candidate_id`:
  - `<current-candidate-id>`
- `owner`:
  - `liveops_community`

## Current Candidate
- `candidate_bundle_ref`:
  - `<output/release-candidates/current.json>`
- `candidate_gate_ref`:
  - `<output/shared-network/.../gate/.../summary.md>`

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
  - `<runbook | command log | operator checklist>`
- `validated_by`:
  - `<liveops owner / runtime owner>`
- `validated_at`:
  - `<YYYY-MM-DD HH:MM:SS TZ>`
- `restoration_scope`:
  - `<runtime build | world snapshot | governance manifest>`

## Verdict
- `lane_result`:
  - `pass | partial | block`
- `reason`:
  - `<why this is pass/partial/block>`

## Notes
- `pass` only if fallback candidate is a formal previous shared-devnet `pass` candidate.
- `partial` if there is only a local/provisional fallback but no formal shared-devnet `pass` history.
- `block` if fallback truth is missing, inconsistent, or not restorable.
