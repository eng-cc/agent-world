# Moltbook 运营 Runbook（2026-03-21）

审计轮次: 6

## Meta
- Owner Role: `liveops_community`
- Review Role: `producer_system_designer`
- Channel: `Moltbook`
- Scope: `发帖前复核 + 发帖后巡检 + 评论分级 + GitHub 回流 + devlog 回写`
- Source Docs:
  - `doc/readme/governance/readme-moltbook-promotion-plan-2026-03-19.md`
  - `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.md`

## 1. 适用范围
- 这份文档用于 `oasis7` 在 Moltbook 进入“持续发帖与持续看反馈”阶段后的日常运营。
- 它不替代推广方案，也不替代首批帖文草案；它只定义怎么执行、怎么回复、怎么升级、怎么回写。
- 任何真实外发内容仍必须遵守 `technical preview / not playable yet / no formal Moltbook integration announced` 的边界。

## 2. 发帖前检查
每次发帖前，按下面顺序做一次 2-5 分钟复核：

1. 确认主贴使用已批准的文案或其安全变体。
2. 确认首评已准备好，且承担更长解释、链接或补充说明。
3. 确认主贴没有以下表述：
   - `live now`
   - `play now`
   - `public launch`
   - `official Moltbook integration`
4. 确认外链只指向当前稳定公开入口。
5. 确认素材与主张一致，不拿 `software_safe` 或 `pure_api` 去替代 3D 视觉 claim。
6. 记录发帖时间、帖子标题和 post id，便于后续巡检和 `devlog` 回写。

## 3. 发帖后 24 小时巡检
前 24 小时是高价值窗口。建议最少按以下节奏检查：

- `T+15m`：确认帖子已正常可见，主贴/首评没有格式问题。
- `T+1h`：第一次看通知和评论，处理明显误解。
- `T+4h`：第二次看互动，筛合作线索和真实问题。
- `T+24h`：做一次小结，决定是否继续跟评、追问或回流 owner。

每次检查都按固定顺序：

1. 先看 `GET /api/v1/home`
   - 目标：快速判断有没有未读通知、哪些帖子有新活动。
2. 再看 `GET /api/v1/notifications`
   - 目标：区分“新关注 / 点赞 / 评论 / 回复”。
3. 如果指向某条帖子，再看：
   - `GET /api/v1/posts/:id`
   - `GET /api/v1/posts/:id/comments?sort=new&limit=35`
4. 如需人工确认公开呈现，再查看 profile 或 post page。

注意：
- `home` 有未读不等于有评论。
- 如果 `activity_on_your_posts` 为空但未读数不为 0，先怀疑是新关注或其他通知，不要误判成“有人留言”。

## 4. 常规日巡检
没有新帖的常规日，也建议至少 1 次轻巡检：

1. 看 `/home` 是否有新的帖子活动或未读通知。
2. 看最新 1-3 条自家帖子是否出现迟到评论。
3. 检查是否有值得补一句 follow-up comment 的高质量讨论。
4. 把需要跨角色处理的内容记录到当天 `devlog`。

如果当天有新帖或外部讨论升温，可加到 2-3 次。

## 5. 评论与通知分级
统一分成 4 桶：

### P1: 口径风险 / 状态误解
- 典型问题：
  - “已经上线了吗？”
  - “现在能玩吗？”
  - “是不是已经和 Moltbook 集成了？”
- 动作：
  - 直接回复。
  - 第一优先是纠偏，不是扩写愿景。
- 安全方向：
  - 重申 `technical preview`
  - 重申 `not playable yet`
  - 重申“这里只是平台原生推广，不代表正式集成已宣布”

### P1: 合作 / 集成 / 路线图追问
- 典型问题：
  - “什么时候做 identity / onchain / OpenClaw？”
  - “能不能一起做某个合作？”
- 动作：
  - 可以礼貌确认“这是有价值信号”。
  - 不公开承诺时间、范围或 owner。
  - 同步升级给 `producer_system_designer`。

### P2: 真实 bug / friction / 文档缺口
- 典型问题：
  - “我试了预览，这里报错”
  - “文档没看懂 / 缺步骤”
  - “某个 surface 很 rough”
- 动作：
  - 优先引导到 GitHub `issue`。
  - 如果对方已有修复方案，优先引导到 GitHub `PR`。
  - 内部再同步给 `qa_engineer` 与对应工程 owner。

### P2: 机制与设计讨论
- 典型问题：
  - “为什么要做 `pure_api`？”
  - “三种 access surface 的边界是什么？”
- 动作：
  - 可以直接回答。
  - 尽量用已批准主张和具体证据回答，不讲未落地 roadmap。

### P3: 纯情绪互动
- 典型问题：
  - “cool”
  - “interesting”
- 动作：
  - 可简短互动，也可不消耗太多精力。
  - 不需要升级。

## 6. 回复边界
公开回复时默认遵守以下顺序：

1. 先校正状态边界。
2. 再给最小必要解释。
3. 最后给明确下一步动作。

推荐句型：
- `Still a technical preview. Not playable yet.`
- `This thread is not announcing formal Moltbook integration.`
- `If you tried the preview and hit a rough edge, the best next step is a GitHub issue.`
- `If you already have a fix in mind, a PR is even better.`

不要做的事：
- 不在评论区承诺发布日期。
- 不在评论区承诺合作已确定。
- 不在评论区把探索性方向说成 roadmap。
- 不把渠道互动区当成长期 debug 线程。

## 7. GitHub 回流规则
- 外部反馈属于 `bug / friction / missing docs`
  - 引导到 GitHub `issue`
- 外部反馈属于 `I can fix this / I want to submit a change`
  - 引导到 GitHub `PR`
- 外部反馈属于 `feature idea / product direction`
  - 可先在评论里收一句上下文，再内部回流 `producer_system_designer`

口径要求：
- 用 `after you inspect or try the preview`
- 不用 `after you play the game`

## 8. 升级矩阵
| 场景 | 直接 owner | 升级动作 |
| --- | --- | --- |
| 对外承诺、合作、路线图追问 | `producer_system_designer` | 记录原话、帖子链接、你的拟回复边界 |
| 真实缺陷、兼容性、体验阻断 | `qa_engineer` + 对应工程 owner | 记录 surface、现象、是否已有 GitHub issue |
| 对玩法、世界机制的高价值兴趣 | `producer_system_designer` | 记录兴趣点和频次 |
| 创作者放大、联动意向 | `liveops_community` -> `producer_system_designer` | 先判定是否越界，再决定 follow-up |

## 9. 当日回写要求
当天做过 Moltbook 动作后，必须回写 `doc/devlog/YYYY-MM-DD.md`。

最少记录：
- 时间
- 角色：`liveops_community`
- 完成内容：
  - 发了什么
  - 查了什么
  - 有没有评论 / 关注 / 质量信号 / 合作信号
- 遗留事项：
  - 需要谁 follow-up
  - 哪条评论还没回
  - 哪个问题需要升级

推荐附带字段：
- `post_id`
- `signal_tags`
- `owner`
- `next_action`

## 10. 每周复盘
每周至少做一次轻复盘，回答 4 个问题：

1. 哪类帖子最容易引发高质量讨论？
2. 哪类表述最容易引发“是不是已经上线了”的误解？
3. 本周有多少条信号转成了 GitHub `issue` / `PR` / owner follow-up？
4. 下周继续推什么：
   - `world proof`
   - `agent diary`
   - `builder hook`
   - `pure_api`

## 11. 参考入口
- `https://www.moltbook.com/skill.md`
- `https://www.moltbook.com/api/v1/home`
- `https://www.moltbook.com/api/v1/notifications`
- `https://www.moltbook.com/api/v1/posts/:id`
- `https://www.moltbook.com/api/v1/posts/:id/comments`
- `doc/readme/governance/readme-moltbook-promotion-plan-2026-03-19.md`
- `doc/readme/governance/readme-moltbook-post-drafts-2026-03-19.md`
