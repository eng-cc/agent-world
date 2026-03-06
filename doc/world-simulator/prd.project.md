# world-simulator PRD Project

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_SIMULATOR-001 (PRD-WORLD_SIMULATOR-001) [test_tier_required]: 完成 world-simulator PRD 改写，建立模拟层设计主入口。
- [x] TASK-WORLD_SIMULATOR-002 (PRD-WORLD_SIMULATOR-001/002) [test_tier_required]: 对齐场景系统、Viewer、启动器的统一验收清单。
- [x] TASK-WORLD_SIMULATOR-003 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: 固化 Web-first 闭环与 LLM 链路的测试证据模板。
- [x] TASK-WORLD_SIMULATOR-004 (PRD-WORLD_SIMULATOR-003) [test_tier_required]: 建立 simulator 体验质量趋势跟踪。
- [x] TASK-WORLD_SIMULATOR-005 (PRD-WORLD_SIMULATOR-004/005) [test_tier_required]: 完成“启动器链上转账”PRD 条款建模与验收标准冻结（接口、安全、测试口径）。
- [x] TASK-WORLD_SIMULATOR-006 (PRD-WORLD_SIMULATOR-004) [test_tier_required]: `world_chain_runtime` 新增转账提交接口（含请求校验、结构化响应、单元测试）。
- [x] TASK-WORLD_SIMULATOR-007 (PRD-WORLD_SIMULATOR-005) [test_tier_required]: runtime 新增主 token 账户转账动作/事件/状态更新（含 nonce anti-replay、余额约束、回归测试）。
- [x] TASK-WORLD_SIMULATOR-008 (PRD-WORLD_SIMULATOR-004) [test_tier_required]: `agent_world_client_launcher` 新增转账 UI 与提交流程（含输入校验、状态提示、错误展示）。
- [x] TASK-WORLD_SIMULATOR-009 (PRD-WORLD_SIMULATOR-004/005) [test_tier_required]: 完成启动器-链运行时转账闭环测试（`test_tier_required`）与测试证据沉淀。
- [x] TASK-WORLD_SIMULATOR-010 (PRD-WORLD_SIMULATOR-001/002/003) [test_tier_required]: 建立模块级专题任务映射索引（2026-03-02 批次）。
- [x] TASK-WORLD_SIMULATOR-011 (PRD-WORLD_SIMULATOR-001/002) [test_tier_required]: 将 Viewer 使用手册迁入 `viewer/` 主题目录并保留根目录兼容跳转。
- [x] TASK-WORLD_SIMULATOR-012 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: 为启动器反馈“分布式提交失败回落本地”补充 `Connection refused` 回归约束测试，锁定错误签名可诊断性。
- [x] TASK-WORLD_SIMULATOR-013 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: 在 `agent_world_client_launcher` 顶部新增区块链启动状态可视化与探针回归测试（禁用/未启动/启动中/已就绪/不可达）。
- [x] TASK-WORLD_SIMULATOR-014 (PRD-WORLD_SIMULATOR-006) [test_tier_required]: 完成“启动器链/游戏独立启动 + 反馈链就绪门控”需求建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-015 (PRD-WORLD_SIMULATOR-006) [test_tier_required]: 在 `agent_world_client_launcher` 落地链/游戏独立启动按钮、启动器打开默认拉起链、反馈入口链就绪门控，并补齐回归测试。
- [x] TASK-WORLD_SIMULATOR-016 (PRD-WORLD_SIMULATOR-007) [test_tier_required]: 完成“启动器完整设置中心”需求建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-017 (PRD-WORLD_SIMULATOR-007) [test_tier_required]: 在 `agent_world_client_launcher` 落地完整设置中心（游戏/区块链/LLM 一体化配置入口）并补齐回归测试。
- [x] TASK-WORLD_SIMULATOR-018 (PRD-WORLD_SIMULATOR-001/002/003/004/005) [test_tier_required]: 对齐 strict PRD schema，补齐 Critical User Flows、Functional Specification Matrix、Edge Cases、NFR、Validation & Decision Record。
- [x] TASK-WORLD_SIMULATOR-019 (PRD-WORLD_SIMULATOR-008) [test_tier_required]: 完成“viewer native 粉紫屏回归”PRD 建模与任务拆解（含 Web+native 验收口径）。
- [x] TASK-WORLD_SIMULATOR-020 (PRD-WORLD_SIMULATOR-008) [test_tier_required]: 修复 `agent_world_viewer` 默认 tonemapping 在 native 链路的 feature 缺失回归，并补齐回归测试与抓帧验证。
- [x] TASK-WORLD_SIMULATOR-021 (PRD-WORLD_SIMULATOR-009) [test_tier_required]: 完成“launcher bundle 运行中二进制覆写失败（Text file busy）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-022 (PRD-WORLD_SIMULATOR-009) [test_tier_required]: 修复 `build-game-launcher-bundle.sh` 二进制复制阶段的占用冲突，验证重复打包不再出现 `Text file busy`。
- [x] TASK-WORLD_SIMULATOR-023 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 完成“启动器 Web 控制台（无 GUI 服务器场景）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-024 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 落地 `world_web_launcher`（远程 Web 控制台/API）、打包入口与回归测试。
- [x] TASK-WORLD_SIMULATOR-025 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: 完成“启动器 UI schema 共享（native/web）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-026 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: 落地共享 launcher UI schema crate，接入 native/web 渲染并补齐回归测试。
- [x] TASK-WORLD_SIMULATOR-027 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: 完成“启动器 egui Web 同层复用与静态资源服务”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-028 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: 落地 launcher egui wasm 复用、`world_web_launcher` 静态托管与 bundle `web-launcher/` 打包链路。
- [x] TASK-WORLD_SIMULATOR-029 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 完成“启动器 wasm 时间兼容与 Web 闭环修复”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-030 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: 修复 launcher wasm `time not implemented` 崩溃并完成 Playwright headed 闭环采证。
- [x] TASK-WORLD_SIMULATOR-031 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: 完成“启动器 Web 必填校验分流修复”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-032 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: 修复 Web 端 native-only 必填项误报（launcher/chain runtime bin）并完成 Playwright 回归。
- [x] TASK-WORLD_SIMULATOR-033 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: 完成“启动器 native/web 同控制面 + native 客户端服务端分离”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-034 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: 升级 `world_web_launcher` 为游戏/区块链独立编排控制面，新增链独立启停 API 与状态快照。
- [x] TASK-WORLD_SIMULATOR-035 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: `agent_world_client_launcher` native 改为客户端-服务端分离并复用同一 API 控制链路，恢复 web 端链启停与状态对齐并完成 Playwright 回归。
- [x] TASK-WORLD_SIMULATOR-036 (PRD-WORLD_SIMULATOR-016) [test_tier_required]: 完成“viewer live runtime/world 接管 Phase 1”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-037 (PRD-WORLD_SIMULATOR-016) [test_tier_required]: 落地 `world_viewer_live --runtime-world`、runtime live 兼容适配与 required 回归收口。
- [x] TASK-WORLD_SIMULATOR-038 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: 完成“viewer live runtime/world 接管 Phase 2（LLM/chat/prompt）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-039 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: 落地 runtime live `LLM/chat/prompt` 控制打通、CLI 接线与 required 回归收口。
- [x] TASK-WORLD_SIMULATOR-040 (PRD-WORLD_SIMULATOR-018) [test_tier_required]: 完成“viewer live runtime/world 接管 Phase 3（action 映射覆盖 + 旧分支移除）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-041 (PRD-WORLD_SIMULATOR-018) [test_tier_required]: 落地 runtime live action 映射覆盖扩展、等价回归测试与 `world_viewer_live` runtime-only 分支收敛。
- [x] TASK-WORLD_SIMULATOR-042 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 完成“viewer live runtime/world 真 LLM 全量接管（LLM 决策 + 100% 事件/快照 + hard-fail）”PRD 建模与任务拆解。
- [x] TASK-WORLD_SIMULATOR-043 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 移除启发式 sidecar，落地真实 LLM driver + shadow WorldKernel，并接入硬失败语义。
- [x] TASK-WORLD_SIMULATOR-044 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 补齐 runtime 事件/快照 100% 映射、扩展 viewer 协议并输出 DecisionTrace。
- [x] TASK-WORLD_SIMULATOR-045 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 执行 required 回归、更新 viewer 手册与模块项目状态收口。

## 专题任务映射（2026-03-02 批次）
- [x] SUBTASK-WORLD_SIMULATOR-20260302-001 (PRD-WORLD_SIMULATOR-001/002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-002 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-003 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-004 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-005 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-006 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-007 (PRD-WORLD_SIMULATOR-001/002/003) [test_tier_required]: `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-008 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-009 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.prd.project.md`

## 专题任务映射（2026-03-03 批次）
- [x] SUBTASK-WORLD_SIMULATOR-20260303-001 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260303-002 (PRD-WORLD_SIMULATOR-002/003) [test_tier_required]: legacy launcher desktop 方案文档清理（旧文档已删除）
- [x] SUBTASK-WORLD_SIMULATOR-20260303-003 (PRD-WORLD_SIMULATOR-006) [test_tier_required]: legacy launcher unified 方案文档清理（旧文档已删除）
- [x] SUBTASK-WORLD_SIMULATOR-20260303-004 (PRD-WORLD_SIMULATOR-007) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.prd.project.md`

## 专题任务映射（2026-03-04 批次）
- [x] SUBTASK-WORLD_SIMULATOR-20260304-001 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-002 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-003 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-004 (PRD-WORLD_SIMULATOR-013) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-005 (PRD-WORLD_SIMULATOR-014) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-006 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260304-007 (PRD-WORLD_SIMULATOR-016) [test_tier_required]: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.project.md`

## 专题任务映射（2026-03-05 批次）
- [x] SUBTASK-WORLD_SIMULATOR-20260305-001 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260305-002 (PRD-WORLD_SIMULATOR-018) [test_tier_required]: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260305-003 (PRD-WORLD_SIMULATOR-018) [test_tier_required]: `crates/agent_world/src/viewer/runtime_live/control_plane.rs` + `crates/agent_world/src/bin/world_viewer_live.rs`
- [x] SUBTASK-WORLD_SIMULATOR-20260305-004 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.project.md`

## 依赖
- doc/world-simulator/prd.index.md
- `doc/world-simulator/scenario/scenario-files.prd.md`
- `doc/world-simulator/viewer/viewer-web-closure-testing-policy.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-chain-runtime-decouple-2026-02-28.prd.md`
- `doc/world-simulator/prd/acceptance/unified-checklist.md`
- `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`
- `doc/world-simulator/prd/quality/experience-trend-tracking.md`
- `doc/world-simulator/prd/launcher/blockchain-transfer.md`
- `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.md`
- `.agents/skills/prd/check.md`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world_launcher_ui/src/lib.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world/src/runtime/world/event_processing/action_to_event_core.rs`
- `crates/agent_world_viewer/Cargo.toml`
- `crates/agent_world_viewer/src/main.rs`
- `scripts/build-game-launcher-bundle.sh`
- `scripts/capture-viewer-frame.sh`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-05
- 当前状态: completed
- 当前优先任务: none
- 并行待办: 无
- 专题映射状态: 2026-03-02 批次 9/9、2026-03-03 批次 4/4、2026-03-04 批次 7/7、2026-03-05 批次 4/4 已纳入模块项目管理文档。
- 手册入口状态: `doc/world-simulator/viewer/viewer-manual.md` 为唯一活跃手册入口。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: `C2-001/C2-002` 已完成物理合并（`experience-overhaul` 与 `live-event-driven-phase10` 为主入口，phase 文档归档）。
- 说明: 本文档仅维护 world-simulator 模块设计执行状态；过程记录在 `doc/devlog/2026-03-05.md`。
