# Local Codex Three-Loop Manual Entry — Design

Version: **v1.1.0**
Last Updated: **2026-09-09**
Status: **non-normative design companion; implementation and activation pending**

The [workflow source](./source-of-truth.md#manual-three-loop-transition) is the sole normative authority. This design reconciles the supplied manual-trigger proposal with the existing harness. It is not a task ledger, installed interface, authorization receipt, or claim of live verification. Implementation progress and evidence belong in GitHub task issues.

## 1. Manual execution boundary

The user starts one bounded task or explicitly resumes an identified task in the local Codex client. Three loops are professional delivery categories, not services or resident processes. TPM coordinates existing professional slices and the existing lifecycle inside that request. There is no new runner prerequisite, service, scheduler, webhook, queue consumer, model proxy, or unattended supervisor profile.

A request can authorize bootstrap, implementation, verification, frozen draft candidate, CI, role review, promotion, authorized merge, and safe cleanup without repeated clicks. Existing holds and authority checks remain effective. Stable CI waiting, unmet dependencies, permission/quota blockers, pause, or the current execution budget ending return control with resumable facts. No timer or background continuation is registered. Short one-shot checks within the active request remain possible; unchanged polling is not continuation authority.

Completion stops execution. Comments, contract publication, upstream merge, CI success, reopening a client, and a model's suggested improvement do not start another task. A loop-only request lacking an objective requests the missing target or presents bounded candidates without editing. Explicit permission to select one ready task selects at most one. Cross-loop feedback stays in the current evidence sink; subsequent task creation requires user authorization.

## 2. Ownership and identity

| Loop | Delivery ownership | Excluded edits |
| --- | --- | --- |
| product | Player promises, product value, gameplay rules, experience acceptance, player explanations | Technical contracts and implementation |
| system | Human-readable technical requirements, architecture, interfaces, state machines, recovery and manuals | Product promises and implementation |
| code | Source, tests, protocol sources, runtime configuration, scripts, executable skills, role cards and adapters | Formal product and system documents |

Roles remain distinct from loops. TPM integrates; specialists decide within their professions; QA, repository health and LiveOps participate through the existing role matrix. Source comments travel with code. Document tasks may read code and run relevant verification. Independent tasks, including two in one loop, may run concurrently when write scopes and resources do not conflict; no global loop lock is introduced.

Each leaf binds one owner, loop, Task UID, canonical worktree/branch and PR chain. Client directory and thread are observations, not alternate task truth. GitHub Issue/Project remains authoritative, with `Loop` and optional navigational `Change ID` projected through the existing adapter. Legacy tasks lacking a loop remain legacy; refresh must not infer ownership from paths. Explicit loop/input/scope/acceptance migration creates a new evidence epoch.

## 3. Proposed data and helper surfaces

All interfaces in this section are pending. Reuse existing implementations where possible rather than create parallel state stores.

The proposed `oasis7.loop-task/v1` binding contains `task_uid`, `change_id`, `loop`, `owner_role`, `bootstrap_epoch`, `manual_request_ref`, `request_key`, `write_scope`, `out_of_scope`, `input_contracts`, `acceptance_refs`, `dependencies`, `target_delivery`, and `policy_digest`. Dependencies must be acyclic. Request identity is persisted with the first creation intent and reused on retry; hashing only loop plus request text incorrectly conflates later identical requests. `manual_request_ref` is auditable context, not a cryptographic user signature or authority conferred by a caller's `trigger=manual` string.

Extend existing bootstrap with fixed fetched default-branch OID, existing-UID binding, loop and request key, and frozen inputs. Do not consume the launch directory's arbitrary HEAD. Existing snapshots and transaction journals handle retry; resume must not duplicate Issue, branch or worktree. Fields must roundtrip through Issue, Project, mapping/cache, packet, review and archive without silently dropping data.

Proposed short-lived `scripts/pm/loop.py` operations are `doctor`, `bind`, `status`, `resume-check`, `recover`, `validate-scope`, `validate-contracts`, and `publish-contract`. Mutating operations require the selected task and current authorized request; `recover` reconciles only that task's recorded action. `workflow-next.py` stays read-only and may emit structured allowlisted helper identifiers, arguments and preconditions; never evaluate shell text from model output or Issue comments. No `tick`, background `watch`, frequency, event cursor, or next-task operation is added.

Three thin skills, `run-product-loop`, `run-system-loop`, and `run-code-loop`, delegate to existing bootstrap/router/packet/slice paths. They do not recursively invoke another loop or construct a task from description matching. Optional local CLI slice adapters are not first-release prerequisites and do not prove role activation or independent review.

## 4. Contracts and effective policy

A contract identifies revision, owner loop, applicable scope, immutable content references, approval reference and upstream contracts. Content references bind commit, path, blob/digest and consumed clauses. Publication reads back independently reviewed merged content through an authorized helper; squash equivalence compares approved versus published content instead of requiring identical source and merge SHAs. Self-authored front matter and ordinary approval prose are insufficient.

Consumption eligibility distinguishes new tasks, completion of in-flight tasks and target release. An approved old revision remains valid when a new draft appears; unrelated documentation changes do not invalidate it. Withdrawal, authority change or explicit input revision blocks affected actions or starts a new evidence epoch. Check eligibility at start/resume, formal dispatch, commit/push, promotion, merge and release. Without a background listener there is no claim of instantaneous revocation during local execution.

Policy is deterministic, versioned and loaded from an already effective trusted revision. Separate `tool_root` (the trusted effective helper/policy revision) from `target_repo_root` (the candidate worktree being checked), and verify their identities before execution; do not load executable authority from the candidate merely because it is the current directory. Proposed `loop_policy.py`, `loop_contracts.py`, policy JSON and schemas should integrate existing packet, PR preparation and CI planning. A candidate modifying policy, helper or skill cannot use its own new rule to authorize itself. Scope enforcement checks both rename endpoints, deletion, new paths, file modes, symlinks and real asset type; unknown or ambiguous ownership fails closed. CI capability union and conservative unknown-path escalation remain intact; loop classification does not replace required gates.

Declared scope, execution pre/post diff checks, frozen PR checks and independent CI are minimum evidence. Native execution isolation must be tested for every actual editing surface, including other worktrees, Git metadata, host configuration, credentials and journals. If unproven, report `execution_scope_unverified`; a conforming final diff does not establish continuous filesystem confinement.

## 5. Reconciliation and delivery

Reuse selected-task queries, pagination/budget rules, portable locks, creation journals and terminal receipts. Mutations whose response is lost require live reconciliation before retry. On one host, task writer coordination spans worktrees through their verified Git common directory, using the OS lock plus actual process identity and surviving-child information. It does not claim cross-host exclusion. A second writer must not enter while the prior process or child still owns the task; TTL expiry never authorizes takeover or proves termination. Never delete the lock file and recreate its inode to bypass ownership. Do not introduce a global same-loop lease or automatic process restart.

Leaf completion and overall change delivery are separate. `change_id` associates required document/code/combined-validation obligations without making a second queue. Required explanations remain outstanding after a code leaf cleans up and may still block release. Cross-loop design gaps are evidenced in the current task rather than silently patched across ownership boundaries.

Return the task/Issue, loop or legacy status, canonical worktree/branch/HEAD, completed steps, actual validation, PR/CI/review state, blockers and whether another explicit continuation is needed. Distinguish a terminal task from a stopped invocation. Pause preserves work; cancellation follows the existing classified non-merge terminal path.

## 6. Transition and compatibility

The D0 normative transition is delivered before executable projections. Current legacy rules did not already mandate single-loop PRs. The approved transition permits subsequent code-only projection PRs to consume an independently reviewed, merged normative revision with immutable commit/clause/scope binding, provided they change no normative semantics. A discovered semantic gap returns for an independently approved system revision. Existing gates must remain passing; this exemption is not permission to disable coupled document or adapter checks.

Planned delivery boundaries are D0 system transition; C1 schema/policy/PM roundtrip; C2 bootstrap/resume; P3 independently reviewed product pilot; S4 independently reviewed system pilot; C5 thin skills and packet routing; C6 scope/contracts/CI/review and delivery obligations; C7 only necessary writer/recovery gaps; S8 actual-use manual and compatibility evidence. These are design dependency labels, not created tasks or automatic dispatch authority. A concrete pilot must be selected by the user or an authorized task scope.

Activation follows approved design, compatible implementation and tests, reviewed code merge, then explicit user enablement through the existing upgrade process. Until those prerequisites pass, new interfaces and ownership enforcement are pending; legacy behavior remains effective. Preserve stable policy and recovery evidence for rollback. In-flight work must not hot-swap inputs, role configuration or authority. The production supervisor remains blocked, and no scheduler staging prerequisite is imposed on this manual path.

## 7. Acceptance evidence

| IDs | Required observations |
| --- | --- |
| M01, M09, M10 | Independent task/worktree execution, same-loop concurrency without global lock, duplicate active writer rejected |
| M02–M08 | No request means no execution; terminal tasks stop; events never dispatch; stable waits yield without schedule; explicit resume preserves identity; insufficient target does not edit; authorized selection chooses one |
| M11–M12 | Cross-loop writes and rename/delete/mode/symlink/new-path bypass attempts rejected; execution isolation reported separately |
| M13–M15 | Valid old inputs remain usable, withdrawal blocks the next relevant boundary, design gaps produce feedback without another task |
| M16–M19 | Merge holds preserved, lost responses reconcile, surviving children prevent writer reuse, blocked tasks do not retry in the background |
| M20–M22 | Effective authority cannot be self-replaced; HEAD/review authority drift invalidates evidence; absent/failed/cancelled/unexpectedly skipped required checks cannot advance |
| M23–M24 | Unfinished explanations remain delivery obligations; installation adds no services/timers and does not claim production-supervisor readiness |

Use offline contract tests and temporary Git/fake-GitHub fault tests for mechanical cases, then authorized local-client and isolated GitHub tasks for actual interoperability. Reports distinguish human actions, in-task Codex actions, GitHub CI actions and measured isolation. Fixtures are not live evidence. Existing frozen-head CI, professional review, holds, merge and ordered cleanup remain mandatory.
