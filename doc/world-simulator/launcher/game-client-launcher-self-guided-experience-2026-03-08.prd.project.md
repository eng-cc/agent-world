# 客户端启动器自引导体验闭环（2026-03-08）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-self-guided-experience-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地首次 3 步引导向导（打开/跳过/完成/重置）与任务状态联动。
- [x] T2 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地主界面任务流卡片（链/游戏/页面）与“下一步建议”渲染。
- [x] T3 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 完成专家模式切换（默认简化视图 + 高级配置入口保持可达）。
- [x] T4 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 为关键禁用态按钮补齐就地 CTA（修复配置/启动链/重试）。
- [x] T5 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 增强配置引导联动（从任务流与 CTA 直达引导，支持引导重置）。
- [x] T6 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 扩展转账轻量模式（金额预设、账户推荐、提交前引导文案）。
- [x] T7 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 新增转账状态时间线展示（accepted/pending/final）。
- [x] T8 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 增加浏览器快捷入口（最新区块/最近交易/我的账户）并接入现有查询。
- [x] T9 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 为 nonce/slot/mempool/action_id 补齐术语内联解释。
- [x] T10 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 新增“最近成功配置画像”保存/恢复/清空闭环。
- [x] T11 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地演示模式一键闭环（安全默认配置 + 串行动作 + 日志提示）。
- [x] T12 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地本地引导埋点计数与洞察面板（打开/跳过/完成/演示）。
- [x] T13 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地“错误卡片 + 一键修复/自动补默认值/重试”并补齐回归测试。
- [ ] T14 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 强化阻塞态下一步动作（启动链、重试探测、修复配置）并补齐回归测试。
- [ ] T15 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 落地启动前体检清单（preflight）与逐项修复入口，并补齐回归测试。
- [ ] T16 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 将 onboarding 改为“跳过后持续轻提示”并补齐回归测试。
- [ ] T17 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 修复专题文档冲突并补充启动器文件拆分任务建模。
- [ ] T18 (PRD-WORLD_SIMULATOR-030) [test_tier_required]: 执行启动器超长文件拆分改造并完成回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/config_ui.rs`
- `crates/agent_world_client_launcher/src/transfer_window.rs`
- `crates/agent_world_client_launcher/src/explorer_window.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/main_tests.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: in_progress（round-2）
- 当前任务: T14（阻塞态可执行下一步）
- 备注: T0~T13 已完成；T14~T18 待执行并需逐任务提交与回归。
