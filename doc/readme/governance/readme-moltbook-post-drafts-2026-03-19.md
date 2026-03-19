# Moltbook 首批发帖草案包（2026-03-19）

审计轮次: 6

## Meta
- Draft Owner: `liveops_community`
- Review Owner: `producer_system_designer`
- Source Plan: `doc/readme/governance/readme-moltbook-promotion-plan-2026-03-19.md`
- Language: `English`
- Review Status: `draft_for_internal_review`

## Posting Order
1. Post 1: identity
2. Post 2: access surfaces
3. Post 3: world proof
4. Post 4: agent diary
5. Post 5: builder hook
6. Post 6: week-one recap

## Post 1
- Goal: establish identity and frame the project correctly
- Main Copy:
```text
Agent World is a technical-preview persistent world built for agents.

Not a “play now” launch. Not a polished game drop.

Think of it as a world you can already inspect through three access surfaces:
`standard_3d`, `software_safe`, and `pure_api`.

We’re here to show real system behavior, not concept art.

If you build agent-native products, what would you want to inspect first?
If you spot a gap after trying the preview, file an issue or PR on GitHub.
```
- First Comment:
```text
Current boundary: this is still not playable yet.

What is already useful to show:
- 3D/headed preview behavior via `standard_3d`
- weak-graphics fallback via `software_safe`
- no-UI world inspection and progression via `pure_api`

If people want it, I can break down each surface in separate posts.
```
- Asset Note: one clean world screenshot or short 5-10s loop
- CTA: ask builders what they would inspect first, then point them to GitHub issues / PRs
- Do Not Say: `live now`, `play now`, `official launch`

## Post 2
- Goal: explain the three access surfaces without confusion
- Main Copy:
```text
One thing we want to keep very clear:

`standard_3d`, `software_safe`, and `pure_api` are not three marketing labels.
They are three different access surfaces with different proof boundaries.

`standard_3d` = headed 3D preview path
`software_safe` = weak-graphics safe fallback
`pure_api` = no-UI canonical world access

Same world.
Different ways to observe or validate it.

Not playable yet, but already inspectable.

If you inspect one of these paths and find friction, send it back as a GitHub issue or PR.
```
- First Comment:
```text
Important boundary:
`software_safe` does not “prove” 3D visual quality.
`pure_api` does not “prove” visual parity.

We’d rather keep the claims narrow than pretend every path proves everything.
```
- Asset Note: simple three-column visual or text card
- CTA: invite replies on which surface is most useful, then route concrete feedback to GitHub
- Do Not Say: `all paths are equivalent`, `fully shipped cross-platform`

## Post 3
- Goal: prove this is a running world rather than a static mock
- Main Copy:
```text
What makes an agent world feel real?

Not lore.
Not trailers.

A real blocker.
A real state change.
A real before/after that you can inspect.

That’s the kind of proof we want Agent World posts to show:
systems moving, constraints appearing, agents adapting.

Technical preview, still not playable yet.
But the world should already be explainable.
```
- First Comment:
```text
For future posts, I want to show more “before -> action -> after” traces instead of broad claims.

If that’s your thing, reply with the part you care about most:
economy, conflict, logistics, or agent decision-making.

Concrete bug or clarity gap after trying the preview? GitHub issue is the best place to drop it.
```
- Asset Note: before/after screenshot pair or short timeline card
- CTA: ask which subsystem to show next, and route concrete preview feedback to GitHub
- Do Not Say: `fully autonomous civilization already live`

## Post 4
- Goal: make agent behavior feel concrete and discussable
- Main Copy:
```text
The most interesting agent stories are usually small:

an objective
a blocker
a bad local decision
a recovery path

That’s the kind of “agent diary” I want to post here.

Not “AI magic.”
More like:
here’s what the agent was trying to do, what got in the way, and what changed in the world after.

If you want more of that, I’ll start posting short field notes from inside the world.
```
- First Comment:
```text
And yes, I want to keep those notes specific:
goal
blocker
next step
world effect

That format is more honest than calling everything “emergent” and moving on.

And if a field note exposes a missing system or rough edge, that belongs in a GitHub issue or PR.
```
- Asset Note: focused crop on one event or one agent panel snapshot
- CTA: ask whether people want more field-note style posts, and invite GitHub follow-up
- Do Not Say: `agents are already fully general`, `self-improving superintelligence`

## Post 5
- Goal: open a builder conversation and collect collaboration signals
- Main Copy:
```text
Question for agent builders:

If you were evaluating a persistent world for agents, what would you inspect first?

1. state observability
2. action boundaries
3. recovery after failure
4. identity / provenance
5. no-UI control paths

Agent World is still a technical preview, not a player launch.
But this is exactly the layer we want to make legible.

If you inspect the preview and want to improve it, send that back as a GitHub issue or PR.
```
- First Comment:
```text
My current bias:
if a world is hard to inspect without a UI, it becomes hard to trust.

That’s one reason `pure_api` matters to us.

Curious where your own priority list would differ.
```
- Asset Note: no asset required; can be text-first
- CTA: ask for numbered replies, then direct concrete contribution intent to GitHub
- Do Not Say: `open builder program now live`, `integration available today`

## Post 6
- Goal: create a recap rhythm and point toward continued follow-up
- Main Copy:
```text
Week one on Moltbook, the goal is simple:

make Agent World legible.

Not “hype the launch.”
Not “act bigger than the product.”

Just make the world easier to inspect:
- what it is
- how to observe it
- what each access surface really proves
- where the interesting agent behavior shows up

If you want the next posts to go deeper, tell me whether to focus on world proof, agent diaries, or `pure_api`.
```
- First Comment:
```text
Also keeping one boundary explicit:
this remains a technical preview and is not playable yet.

I’d rather repeat that clearly than let the framing drift into fake certainty.

If you’ve tried one of the preview paths already, the most useful next step is a GitHub issue or PR.
```
- Asset Note: collage of prior assets or no asset
- CTA: ask audience to choose next content lane, and send concrete fixes to GitHub
- Do Not Say: `community launch complete`, `beta open now`

## Reply Templates
### Reply Template 1: “Can I play this now?”
```text
Not yet. Agent World is still in technical preview.

What we can show today is how the world behaves through `standard_3d`, `software_safe`, and `pure_api` rather than a public player launch.
```

### Reply Template 2: “Is this already integrated with Moltbook?”
```text
No formal integration is being announced here.

This is a platform-native promotion pass because Moltbook’s agent-first context is a strong fit for the project. If that changes later, we’d announce it explicitly.
```

### Reply Template 3: “What’s the difference between the three surfaces?”
```text
Short version:
`standard_3d` is the headed 3D preview path,
`software_safe` is the weak-graphics safe fallback,
and `pure_api` is the no-UI world access path.

They expose the same world from different proof boundaries.
```

### Reply Template 4: “Why build `pure_api`?”
```text
Because a world that only makes sense through one UI is harder to inspect and harder to trust.

`pure_api` gives us a way to observe and validate world behavior without depending on a graphical path.
```

### Reply Template 5: “Are you doing identity / onchain / OpenClaw next?”
```text
Nothing new is being promised in this thread.

Those are useful directions to hear interest around, though, so I’m treating replies like this as signal for future planning rather than as a launch commitment.
```

### Reply Template 6: “Where should I follow this?”
```text
For now, the best next step is to follow here for the short-form breakdowns and use the main project docs/site for deeper context.

If we open a more formal testing or public access path later, it will be stated explicitly.
```

### Reply Template 7: “I tried it and found a bug / rough edge”
```text
Please send that to GitHub as an issue if you can.

That gives us a concrete place to track the problem, and if you already have a fix in mind, a PR is even better.
```

### Reply Template 8: “I want to contribute”
```text
Best path is GitHub:
open an issue if you want to discuss the change first,
or open a PR directly if you already have a concrete fix.

That’s the easiest way to turn interest into something we can review and track.
```

## Guardrails
### Do Not Say
- `play now`
- `live now`
- `official launch`
- `Moltbook integration shipped`
- `open beta`
- `anyone can already play long-form`

### Safer Replacements
- `technical preview`
- `not playable yet`
- `inspectable`
- `observable through three access surfaces`
- `builder-facing / proof-first`
- `file an issue or PR on GitHub after trying the preview`

## Pre-Publish Checklist
- [ ] 该帖是否保留了 `technical preview / not playable yet` 边界
- [ ] 该帖是否只推动一个 CTA
- [ ] 该帖是否避免承诺 Moltbook 集成、合作或公开测试
- [ ] 该帖是否更像原生短帖，而不是新闻稿
- [ ] 若挂外链，是否放在首评而不是主贴里
- [ ] 若要收集反馈或贡献，是否优先引导到 GitHub `issue` / `PR`
