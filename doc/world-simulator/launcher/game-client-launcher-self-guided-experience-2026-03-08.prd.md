# 客户端启动器自引导体验闭环（2026-03-08）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-self-guided-experience-2026-03-08.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-self-guided-experience-2026-03-08.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 启动器已具备功能完备性，但新用户首次进入时仍需自行理解术语、按钮依赖与推荐操作路径，容易在“能做什么/下一步做什么”上卡住。
- Proposed Solution: 构建“默认自引导、可切专家模式”的体验闭环，补齐首次三步引导、任务流卡片、禁用态下一步指引、术语内联解释、演示模式、成功配置画像与引导埋点。
- Success Criteria:
  - SC-1: 首次进入启动器时，3 步引导流程可见并可在 60 秒内完成，完成后不再重复弹出（除非用户重置）。
  - SC-2: 主界面默认展示任务流卡片与“下一步建议”，新用户无需打开高级配置即可完成“启动链 -> 启动游戏 -> 打开页面”。
  - SC-3: 所有关键禁用按钮（反馈/转账/浏览器/启动）均提供就地可执行 CTA（修复配置、启动链、开启演示等），不再只有被动灰态。
  - SC-4: 转账与浏览器面板具备快速入口（金额预设、搜索快捷跳转）与术语解释，降低区块链概念理解门槛。
  - SC-5: 启动成功配置可自动落盘并可一键恢复；演示模式可在默认参数下拉起最小闭环。
  - SC-6: 启动器记录引导关键事件（打开/完成/跳过/失败）与漏斗计数，支持后续可用性复盘。

## 2. User Experience & Functionality
- User Personas:
  - 新用户（首次体验者）：希望“打开即用”，不需要先理解全部配置和链术语。
  - 运营/演示人员：希望快速进入稳定演示路径，减少手工填参。
  - 启动器维护者：希望通过埋点定位用户卡点并迭代引导。
- User Scenarios & Frequency:
  - 首次安装后的第一次使用：高优先级、每用户至少 1 次。
  - 演示/培训场景：每周多次，需快速重置到可讲解状态。
  - 日常运维：高频，需保留专家模式与高级配置入口。
- User Stories:
  - PRD-WORLD_SIMULATOR-030: As a 新用户, I want launcher to guide me with actionable next steps and safe defaults, so that I can complete first launch without reading extra docs.
- Critical User Flows:
  1. Flow-LAUNCHER-SG-001（首次 3 步引导）:
     `首次打开 -> 展示步骤 1（理解页面目标） -> 步骤 2（检查并修复配置） -> 步骤 3（启动链/游戏并打开页面） -> 标记完成`
  2. Flow-LAUNCHER-SG-002（任务卡片驱动）:
     `主界面展示任务卡片 -> 根据当前状态高亮下一步 -> 用户点击卡片动作 -> 状态更新后自动切到下一张卡`
  3. Flow-LAUNCHER-SG-003（禁用态就地修复）:
     `关键按钮禁用 -> 同行展示原因 + CTA -> 用户点击 CTA 进入配置引导或启动链 -> 按钮恢复可用`
  4. Flow-LAUNCHER-SG-004（转账轻量引导）:
     `打开转账 -> 选择账户 -> 点击金额预设 -> 自动 nonce -> 提交 -> 时间线显示 accepted/pending/final`
  5. Flow-LAUNCHER-SG-005（浏览器快捷跳转）:
     `打开浏览器 -> 使用快捷按钮跳到最新区块/最近交易/我的账户 -> 一键带入查询条件`
  6. Flow-LAUNCHER-SG-006（配置画像恢复）:
     `启动成功 -> 自动保存成功配置画像 -> 下次点击“恢复最近成功配置” -> 关键字段回填`
  7. Flow-LAUNCHER-SG-007（演示模式）:
     `点击演示模式 -> 应用安全默认配置 -> 自动执行链启动与游戏启动 -> 弹出下一步说明`
  8. Flow-LAUNCHER-SG-008（引导漏斗埋点）:
     `用户打开/跳过/完成引导 -> 更新本地计数器 -> 在引导洞察面板可查看`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 首次引导向导 | `onboarding_step`、`onboarding_completed` | 支持下一步/上一步/跳过/完成；可一键打开配置引导 | `hidden -> step1 -> step2 -> step3 -> completed` | 首次未完成自动打开；完成后默认不再自动弹出 | 当前会话可写 |
| 主界面任务流卡片 | `task_id`、`ready`、`blocked_reason`、`cta` | 每张卡提供主动作与下一步建议，点击后触发现有流程 | `todo -> doing -> done/blocked` | 按依赖顺序：链 -> 游戏 -> 页面 -> 资产功能 | 查询只读，动作可写 |
| 禁用态就地 CTA | `disabled_reason`、`cta_type` | 灰态按钮旁显示“为什么不可用 + 立刻修复” | `disabled -> fixing -> enabled` | CTA 优先级：配置修复 > 启动链 > 重试请求 | 当前会话可写 |
| 术语内联解释 | `term_key`（nonce/slot/mempool/action_id） | Hover/点击显示双语说明，不跳离当前页面 | `plain -> tooltip_open` | 术语文案按 key 固定映射 | 只读 |
| 成功配置画像 | `last_successful_config`、`saved_at` | 启动成功自动保存；支持一键恢复与清空 | `none -> saved -> restored` | 仅覆盖最近一次成功快照 | 本地会话可写 |
| 演示模式 | `demo_mode_enabled`、`demo_script_step` | 一键应用最小可演示配置并串行触发动作 | `idle -> preparing -> running -> done/failed` | 使用安全默认值，不覆盖用户手动高级字段 | 当前会话可写 |
| 引导埋点计数 | `onboarding_opened/skipped/completed`、`demo_runs`、`quick_action_clicks` | 行为发生时计数自增并可视化展示 | `counter += 1` | 按事件类型单调递增 | 本地会话可写 |
- Acceptance Criteria:
  - AC-1: 首次进入 launcher 自动打开 3 步引导，且在完成或跳过后不重复弹出（除非用户手动重置引导状态）。
  - AC-2: 主界面默认展示任务流卡片，至少覆盖“启动区块链、启动游戏、打开游戏页”三条主线动作。
  - AC-3: 当 `is_feedback_available=false` 时，反馈/转账/浏览器入口必须展示可见的下一步 CTA（如“先启动区块链”）。
  - AC-4: 启动按钮在配置不合法时必须提供可见“修复配置”动作，并直达现有配置引导窗口。
  - AC-5: 转账窗口提供金额预设按钮（至少 1/10/100）且点击后即时回填金额输入框。
  - AC-6: 转账状态区域提供时间线式阶段展示（Accepted/Pending/Final），并显示当前 action_id。
  - AC-7: 浏览器窗口提供快捷操作（最新区块、最近交易、我的账户）并触发对应查询。
  - AC-8: 启动器界面中 nonce/slot/mempool/action_id 至少 4 个术语具备内联解释。
  - AC-9: 启动成功后自动保存最近成功配置；点击“恢复最近成功配置”后关键字段回填并可再次启动。
  - AC-10: 演示模式按钮可在默认配置下触发“启动链 + 启动游戏”的串行动作，并输出可诊断日志。
  - AC-11: 引导与演示关键行为计数可见（打开/跳过/完成/演示启动），重启后仍保留。
  - AC-12: native/web 两端均通过 `test_tier_required` 回归，核心引导行为一致。
- Non-Goals:
  - 不新增链运行时协议或 Explorer 后端 API。
  - 不在本专题引入远程遥测上报服务（仅本地计数与可视化）。
  - 不替换现有设置中心与高级配置窗口，仅增强可达性与引导层。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不新增 AI 模块；仅复用现有 launcher UI 渲染、控制面 API 与 agent-browser 测试链路。
- Evaluation Strategy: 不适用（本专题以 UX 引导可达性、任务完成率与本地计数验证为主）。

## 4. Technical Specifications
- Architecture Overview:
  - `agent_world_client_launcher` 新增自引导状态层（引导状态、任务卡状态、术语词典、演示模式与本地计数）。
  - 通过独立模块管理“成功配置画像 + 引导计数”持久化，native 落盘、web 端 localStorage。
  - 不改变控制面 API；所有引导动作复用既有 `/api/start`、`/api/chain/start`、`/api/state`、transfer/explorer 查询接口。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world_client_launcher/src/config_ui.rs`
  - `crates/agent_world_client_launcher/src/transfer_window.rs`
  - `crates/agent_world_client_launcher/src/explorer_window.rs`
  - `crates/agent_world_client_launcher/src/app_process.rs`
  - `crates/agent_world_client_launcher/src/app_process_web.rs`
  - `crates/agent_world_client_launcher/src/main_tests.rs`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 引导中点击“启动”但配置非法：保持在当前步骤并打开配置引导，不得静默失败。
  - 演示模式执行中已有请求在途：提示“请求处理中”，延后动作，不并发覆盖。
  - 成功画像损坏/反序列化失败：回退默认配置并记录日志，不阻断主界面。
  - web localStorage 不可用：降级为会话内状态并记录 warning。
  - 转账账户为空时金额预设点击：仅回填金额，不强制提交。
  - 快捷跳转目标缺失（如无 blocks/txs 数据）：显示空态提示，不触发 panic。
- Non-Functional Requirements:
  - NFR-1: 引导窗口打开/切步交互延迟 `p95 <= 50ms`（本地环境）。
  - NFR-2: 配置画像保存与恢复操作 `p95 <= 100ms`（本地环境）。
  - NFR-3: 任务流与 CTA 逻辑在 native/wasm 行为一致，不得出现平台分叉。
  - NFR-4: 自引导新增后 `main.rs`、`explorer_window.rs`、`transfer_window.rs` 单文件行数仍需 <=1200。
  - NFR-5: 演示模式失败时必须保留可诊断日志，不得吞错。
- Security & Privacy:
  - 配置画像与埋点仅本地持久化，不上传远端。
  - 写入内容不得包含 LLM API key 等敏感明文（仅保存 launcher 启动配置）。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (SG-1): 建立专题 PRD、任务拆解与模块文档树映射。
  - v1.1 (SG-2): 首次引导、任务卡片、禁用态 CTA、术语解释。
  - v1.2 (SG-3): 转账轻量引导、浏览器快捷入口、成功配置画像。
  - v1.3 (SG-4): 演示模式与本地引导计数闭环。
- Technical Risks:
  - 风险-1: 引导层和现有状态机并行更新，可能引入按钮状态不一致。
  - 风险-2: 过度引导可能干扰专家用户操作效率。
  - 风险-3: 本地持久化兼容性（native 文件权限、web localStorage 配额）存在环境差异。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_SIMULATOR-030 | TASK-WORLD_SIMULATOR-072/073/074/075/076/077/078/079/080/081/082/083/084 | `test_tier_required` | `./scripts/doc-governance-check.sh` + `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher -- --nocapture` + `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher --target wasm32-unknown-unknown` + agent-browser（桌面+390x844）验证首次引导、任务流、CTA、转账预设、快捷跳转、演示模式与计数展示 | 启动器自引导可用性、跨端一致性与新用户首日留存路径 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LAUNCHER-SG-001 | 采用“默认自引导 + 可跳过 + 可重置”的渐进披露方案 | 仅提供静态帮助文档链接 | 操作中引导可直接闭环问题修复，减少上下文切换。 |
| DEC-LAUNCHER-SG-002 | 成功配置画像按“最近一次成功”覆盖保存 | 维护多配置档案管理器 | 本轮目标是降低首次使用复杂度，先保证一键恢复最短路径。 |
| DEC-LAUNCHER-SG-003 | 演示模式复用现有启动动作，不新增后端专用 API | 新增 `demo/start` 控制面 API | 减少服务端改动面，优先在客户端编排现有稳定动作。 |
| DEC-LAUNCHER-SG-004 | 先做本地计数埋点，不做远端遥测 | 直接接入远端埋点平台 | 降低隐私与部署复杂度，先验证指标口径与产品价值。 |
