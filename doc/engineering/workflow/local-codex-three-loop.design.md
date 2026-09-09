# Local Codex Three-Loop Manual Entry — Design

Version: **v1.2.0**
Last Updated: **2026-09-09**
Status: **non-normative design companion; implementation available, activation pending**

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

## 3. Data and helper surfaces

The policy/contracts modules, schemas, manual facade, PM/bootstrap integration and three thin skills are implemented. Their mechanical tests do not activate the entry or establish native client compatibility. Reuse existing implementations rather than create parallel state stores.

The nested `loop_binding` uses `oasis7.loop-task/v1` and contains `task_uid`, `change_id`, `loop`, `owner_role`, `bootstrap_epoch`, `manual_request_ref`, `request_key`, `write_scope`, `out_of_scope`, `input_contracts`, `acceptance_refs`, `dependencies`, `target_delivery`, `policy_digest`, and immutable `policy_commit`. Dependencies must be acyclic; full closure validation requires authoritative selected dependency bindings. Optional `delivery_obligations` binds obligation ID, task UID and issue number. Request identity is persisted with the first creation intent and reused on retry; hashing only loop plus request text incorrectly conflates later identical requests. `manual_request_ref` is auditable context, not a cryptographic user signature or authority conferred by a caller's `trigger=manual` string.

Manual bootstrap fixes the fetched default-branch OID in its request journal and preserves it on retry; legacy bootstrap retains its existing defaults. The explicit `--pm-task-uid` path resumes the selected existing worktree. Nested metadata roundtrips through Issue, Project navigation, cache, packets, review and archive. New tasks and changed bindings check `new_tasks` contract eligibility; continuation checks `in_flight`. Selected dependency closure is validated without scanning unrelated tasks. Delivery readiness reads the closed/completed Issue, its exact canonical Project item with `Status=Done`, `PM Status=done`, `Workflow Phase=done`, and the unique finalizer `post_merge_done` evidence comment. The Issue body may still say `task_done`; it is not the finalizer's terminal projection. Pending, cancelled, deferred or non-merge closure does not prove merged delivery. New requests require a new persisted request key, even when their text matches an earlier request.

The short-lived `scripts/pm/loop.py` operations are `doctor`, `bind`, `status`, `resume-check`, `recover`, `validate-scope`, `validate-contracts`, and `publish-contract`. Every operation requires a task UID. Binding, continuation, recovery and publication require the current manual request reference. `recover` reconciles only that task's recorded action. `workflow-next.py` remains read-only; the facade reports its result without executing `next_command`. No `tick`, background `watch`, frequency, event cursor, or next-task operation is added.

Three thin skills, `run-product-loop`, `run-system-loop`, and `run-code-loop`, delegate to existing bootstrap/router/packet/slice paths. They do not recursively invoke another loop or construct a task from description matching. Optional local CLI slice adapters are not first-release prerequisites and do not prove role activation or independent review.

### Manual command examples

These examples apply after an approved effective implementation is available and enablement is authorized. Set `LOOP_TOOL_ROOT` to a clean checkout at the binding's immutable `policy_commit`, `LOOP_TASK_ROOT` to the canonical target, `LOOP_TASK_UID` to its assigned UID, `LOOP_BINDING` to the full binding JSON, and `LOOP_REQUEST_REF` to this request's audit reference. Invoke the script from the effective checkout, including when the current directory is a candidate. A missing `gh`, missing effective revision, invalid digest, helper changes or missing Project field blocks admission; copying files into a directory is insufficient.

For a new bounded code task, the binding already contains its assigned UID, owner, scope, immutable inputs and a stable request key. Set `LOOP_REQUEST_KEY` from that binding; the request reference and key must match the JSON. The helper fetches and pins the base; it does not use arbitrary caller HEAD:

```bash
bash "$LOOP_TOOL_ROOT/scripts/new-task-worktree.sh" engineering manual-change \
  --path "$LOOP_TASK_ROOT" --branch codex/manual-change \
  --pm-owner-role repository_health_engineer --pm-title "Bounded manual change" \
  --pm-source-ref "$LOOP_BINDING" --pm-acceptance "Declared contract acceptance" \
  --pm-loop code --pm-loop-binding "$LOOP_BINDING" \
  --pm-request-key "$LOOP_REQUEST_KEY" --pm-manual-request-ref "$LOOP_REQUEST_REF" --json
```

Use the corresponding `product` or `system` binding and owner for those loops. Reusing the original bootstrap request does not create another Issue. To bind or explicitly migrate an already bootstrapped task, use the facade; a changed/legacy binding requires `--migrate-epoch` with exactly the next epoch and matching JSON. An identical binding is an idempotent operation:

```bash
python3 "$LOOP_TOOL_ROOT/scripts/pm/loop.py" bind \
  --repo-root "$LOOP_TASK_ROOT" --tool-root "$LOOP_TOOL_ROOT" \
  --task-uid "$LOOP_TASK_UID" --loop-binding "$LOOP_BINDING" \
  --manual-request-ref "$LOOP_REQUEST_REF" --json
python3 "$LOOP_TOOL_ROOT/scripts/pm/loop.py" doctor \
  --repo-root "$LOOP_TASK_ROOT" --tool-root "$LOOP_TOOL_ROOT" --task-uid "$LOOP_TASK_UID" --json
python3 "$LOOP_TOOL_ROOT/scripts/pm/loop.py" resume-check \
  --repo-root "$LOOP_TASK_ROOT" --tool-root "$LOOP_TOOL_ROOT" \
  --task-uid "$LOOP_TASK_UID" --manual-request-ref "$LOOP_REQUEST_REF" --json
```

`doctor` validates the selected task's available authority; `legacy` is compatibility status, not activated-loop success. `resume-check` reads current task and recovery facts and does not start a model. Use `recover` with the same arguments when an unfinished action needs readback. The bootstrap entry also accepts `--pm-task-uid "$LOOP_TASK_UID" --pm-manual-request-ref "$LOOP_REQUEST_REF"` to locate and validate an existing canonical worktree without creating one. Zero exit status reports a successful check or compatible legacy result, not terminal task completion.

## 4. Contracts and effective policy

A contract identifies revision, owner loop, applicable scope, immutable content references, approval reference and upstream contracts. The implemented `loop_contracts.py` reader fetches the canonical GitHub task issue, exact structured publication comment, server author and current repository admin permission, plus the merged PR's source and merge commits. Content references bind path, SHA-256 and clauses, and both commits must contain matching regular-file bytes; squash equivalence does not require identical commit SHAs. Each input freezes `contract_id`, `revision`, `contract_digest`, `publication_ref={issue_number,comment_id}` and `consumed_clauses`. The digest covers canonical JSON of every contract field except live `eligibility`; substituted clauses still fail admission. Self-authored front matter and ordinary approval prose are insufficient. Admin publication provides human-operated authorization/provenance and relies on the existing merged-PR review gates; it is not independent runtime attestation.

`publish_contract` verifies content and upstream eligibility before writing, then validates server readback. It reconciles an exact task/contract revision/digest comment before retrying a lost response, rejecting conflicting or duplicate records. The facade must retain the task writer lock across this bounded publication transaction. Publication/eligibility evidence remains in the task issue rather than a new registry; withdrawal is an admin-authorized eligibility update to that structured publication and is checked on the next admission. Live publication has not been exercised by offline tests.

Consumption eligibility distinguishes new tasks, completion of in-flight tasks and target release. An approved old revision remains valid when a new draft appears; unrelated documentation changes do not invalidate it. Withdrawal, authority change or explicit input revision blocks affected actions or starts a new evidence epoch. Check eligibility at start/resume, formal dispatch, commit/push, promotion, merge and release. Without a background listener there is no claim of instantaneous revocation during local execution.

Policy is deterministic, versioned and loaded from an already effective trusted revision. Separate `tool_root` (the trusted effective helper/policy revision) from `target_repo_root` (the candidate worktree being checked). `validate_tool_root` requires the pinned commit at the tool checkout, common-directory identity, an ancestor of refreshed `origin/main`, unchanged tracked helper bytes and no untracked helper shadows; callers retain live refresh responsibility. `validate_scope` reads policy bytes from that immutable commit and verifies their digest. Do not load executable authority from the candidate merely because it is the current directory. The modules integrate existing packet, PR preparation and CI planning; a candidate modifying policy, helper or skill cannot use its own new rule to authorize itself. Scope enforcement checks both rename endpoints, deletion, new paths, file modes, symlinks and document extensions; unknown ownership fails closed. Executable agent instructions are code even under document paths. CI capability union and conservative unknown-path escalation remain intact; loop classification does not replace required gates.

Declared scope, execution pre/post diff checks, frozen PR checks and independent CI are minimum evidence. Native execution isolation must be tested for every actual editing surface, including other worktrees, Git metadata, host configuration, credentials and journals. If unproven, report `execution_scope_unverified`; a conforming final diff does not establish continuous filesystem confinement.

## 5. Reconciliation and delivery

Reuse selected-task queries, pagination/budget rules, portable locks, creation journals and terminal receipts. Mutations whose response is lost require live reconciliation before retry. On one host, task writer coordination spans worktrees through their verified Git common directory, using the OS lock plus actual process identity and surviving-child information. It does not claim cross-host exclusion. A second writer must not enter while the prior process or child still owns the task; TTL expiry never authorizes takeover or proves termination. Never delete the lock file and recreate its inode to bypass ownership. Do not introduce a global same-loop lease or automatic process restart.

Leaf completion and overall change delivery are separate. `change_id` associates required document/code/combined-validation obligations without making a second queue. Required explanations remain outstanding after a code leaf cleans up and may still block release. Cross-loop design gaps are evidenced in the current task rather than silently patched across ownership boundaries.

Return the task/Issue, loop or legacy status, canonical worktree/branch/HEAD, completed steps, actual validation, PR/CI/review state, blockers and whether another explicit continuation is needed. Distinguish a terminal task from a stopped invocation. Pause preserves work; cancellation follows the existing classified non-merge terminal path.

## 6. Transition and compatibility

The explicitly user-authorized bootstrap migration delivers the normative transition first and executable projections afterward in one PR, under legacy gates; the original separate D0 rollout is superseded for this migration. Current legacy rules did not already mandate single-loop PRs. Future code-only projection PRs may consume an independently reviewed, merged normative revision with immutable commit/clause/scope binding, provided they change no normative semantics. A discovered semantic gap returns for an independently approved system revision. Existing gates must remain passing; this exemption is not permission to disable coupled document or adapter checks or to admit the bootstrap candidate using its own policy.

The original D0/C1/C2/C5/C6/C7 labels describe parts of this integrated implementation, not separate-PR prerequisites. P3/S4 product/system pilots and actual-use compatibility evidence require their own authorized examples; they are not automatically created or claimed by the harness implementation. A concrete pilot must be selected by the user or an authorized task scope.

Activation follows approved design, compatible implementation and tests, reviewed code merge, then explicit user enablement through the existing upgrade process. Reviewable implementation is not permission to merge: a user hold remains binding throughout draft, CI and promotion. Until activation prerequisites pass, legacy behavior remains effective and the new entry must not be presented as installed or enabled. Preserve stable policy and recovery evidence for rollback. In-flight work must not hot-swap inputs, role configuration or authority. The production supervisor remains blocked, and no scheduler staging prerequisite is imposed on this manual path.

Compatibility evidence includes temporary-Git/fake-GitHub bootstrap, request retries, existing-UID resume, epoch migration and the existing PM regressions. The helpers require Git, Python, Bash and authenticated `gh`; Project navigation uses `Loop` (product/system/code) and `Change ID`. Mechanical fixtures do not establish real publication, full client skill activation, native model dispatch or continuous filesystem isolation. The manual facade does not add a native CLI adapter or invoke a model itself.

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
