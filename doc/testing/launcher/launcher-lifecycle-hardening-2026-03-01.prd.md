# Agent World: 启动器生命周期与就绪硬化（2026-03-01）

审计轮次: 1

- 对应项目管理文档: doc/testing/launcher/launcher-lifecycle-hardening-2026-03-01.prd.project.md

## 1. Executive Summary
- Problem Statement: `world_game_launcher` 在启动失败、外部停止和就绪探针场景存在生命周期缺陷，可能残留子进程或产生“已就绪”假阳性，影响测试门禁与人工验收可信度。
- Proposed Solution: 强化启动器生命周期管理（信号清理、失败回滚、就绪健康联动）并统一 IPv6 解析/URL 规则到 CLI 与 GUI 入口。
- Success Criteria:
  - SC-1: 启动失败（如静态 HTTP 绑定失败）时已拉起子进程可立即回滚终止。
  - SC-2: 就绪探针仅在子进程持续存活且响应合法时判定成功。
  - SC-3: IPv6 地址解析强制 bracket 规范，URL 输出自动兼容 `[ipv6]:port`。
  - SC-4: `world_game_launcher` 与 `agent_world_client_launcher` 规则一致并通过定向测试。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：需要启动链路结果稳定可复现。
  - 发布负责人：需要“ready”状态可作为真实放行依据。
  - 桌面启动器用户：需要 GUI/CLI 参数语义一致。
- User Scenarios & Frequency:
  - 日常本地启动：每次调试或回归都会触发。
  - 发布前验收：需要验证链路不会遗留脏进程。
  - 网络地址配置：IPv4/IPv6 混合环境下频繁使用。
- User Stories:
  - PRD-TESTING-LAUNCHER-HARDEN-001: As a 测试维护者, I want launcher shutdown and rollback to clean all child processes, so that repeated runs remain deterministic.
  - PRD-TESTING-LAUNCHER-HARDEN-002: As a 发布负责人, I want readiness checks to reject false positives, so that gate evidence reflects real runtime health.
  - PRD-TESTING-LAUNCHER-HARDEN-003: As a 启动器用户, I want IPv6 parsing and URL formatting to be consistent across CLI/GUI, so that connection setup is predictable.
- Critical User Flows:
  1. Flow-LIFECYCLE-001: `启动 launcher -> 拉起 viewer/chain -> 若启动阶段失败则回滚清理全部子进程`
  2. Flow-LIFECYCLE-002: `ready 等待期间 -> 持续检查子进程存活与 HTTP 响应合法性 -> 满足后才标记 ready`
  3. Flow-LIFECYCLE-003: `输入 host:port -> IPv6 bracket 解析校验 -> 生成 URL 并用于连接`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 信号清理 | shutdown 标记、子进程句柄 | 收到终止信号后触发统一清理路径 | `running -> shutting_down -> cleaned` | 清理顺序先子进程后主进程退出 | 运行时自动执行 |
| 启动失败回滚 | 失败原因、回滚目标进程 | 绑定失败或初始化失败时立即终止已启动进程 | `starting -> failed -> rolled_back` | 失败即回滚，不允许部分残留 | 启动器维护者定义策略 |
| 就绪探针增强 | TCP/HTTP ready、进程存活、响应前缀 | 轮询期间校验存活与响应格式 | `probing -> ready/failed` | HTTP 仅接受合法响应前缀 | 测试与发布流程可审阅日志 |
| IPv6 解析与 URL | `[::1]:port`、URL host 包裹规则 | 接受 bracket IPv6，拒绝裸 IPv6:port | `input -> parsed/rejected` | URL 输出时 IPv6 自动加 `[]` | CLI/GUI 统一遵循 |
| 代码清理降噪 | 未使用测试包装函数、测试模块路径 | 移除冗余并重组测试入口 | `noisy -> cleaned` | 保持测试语义不变 | 维护者可调整结构 |
- Acceptance Criteria:
  - AC-1: `world_game_launcher` 支持终止信号清理、启动失败回滚、就绪存活联动。
  - AC-2: HTTP ready 仅在合法响应前缀下判定成功。
  - AC-3: `parse_host_port` 支持 bracket IPv6，拒绝未 bracket 写法。
  - AC-4: `agent_world_client_launcher` 与 `world_game_launcher` 地址规则一致。
  - AC-5: 定向单测与文档收口完成，`--all-targets` 构建噪音下降。
- Non-Goals:
  - 不改动 `world_chain_runtime` 业务协议。
  - 不引入新的分布式编排框架。
  - 不放宽 IPv6 输入规范到歧义格式。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为启动器生命周期与网络地址规则硬化，不涉及 AI 模型策略变更）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `world_game_launcher` 为核心执行器，围绕“启动-就绪-停止”阶段增加失败原子性和探针真实性保障，并由客户端启动器共享同一地址解析语义。
- Integration Points:
  - `crates/agent_world/src/bin/world_game_launcher.rs`
  - `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world/src/bin/world_viewer_live/distfs_probe_runtime.rs`
  - `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part2.rs`
- Edge Cases & Error Handling:
  - 静态 HTTP 端口占用：视为启动失败并触发子进程回滚。
  - 子进程短暂启动后立即退出：ready 轮询中立即失败，避免误报成功。
  - 非 HTTP 端口返回垃圾字节：拒绝 ready 判定并记录错误。
  - IPv6 裸地址 `::1:port`：直接报错并提示 bracket 格式。
  - 测试环境信号处理干扰：采用一次性安装并避免测试主动发送终止信号。
- Non-Functional Requirements:
  - NFR-LIFECYCLE-1: 启动失败后无残留 `world_viewer_live`/`world_chain_runtime` 进程。
  - NFR-LIFECYCLE-2: 就绪判定误报窗口最小化，ready 状态可审计。
  - NFR-LIFECYCLE-3: CLI 与 GUI 地址规则一致，错误提示可操作。
  - NFR-LIFECYCLE-4: 测试组织优化后 `--all-targets` 构建噪音降低。
- Security & Privacy: 启动器仅处理本地进程与地址参数，不新增敏感信息收集。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (LCH-1): 设计/项目文档建档。
  - v1.1 (LCH-2): `world_game_launcher` 生命周期与就绪逻辑硬化。
  - v2.0 (LCH-3): `agent_world_client_launcher` IPv6/URL 规则对齐。
  - v2.1 (LCH-4): 回归测试、文档收口与代码噪音清理。
- Technical Risks:
  - 风险-1: 全局信号处理可能影响并行测试行为。
  - 风险-2: IPv6 解析收紧导致历史非标准写法失败。
  - 风险-3: 过于严格的 ready 规则可能暴露旧脚本隐藏问题，需要配套错误信息。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LAUNCHER-HARDEN-001 | LCH-1/2 | `test_tier_required` | 信号清理与失败回滚定向测试 | 启动器生命周期稳定性 |
| PRD-TESTING-LAUNCHER-HARDEN-002 | LCH-2/3 | `test_tier_required` | ready 探针合法响应与进程存活联动校验 | 发布门禁 ready 可信度 |
| PRD-TESTING-LAUNCHER-HARDEN-003 | LCH-3/4 | `test_tier_required` | IPv6 解析/URL 规则单测与文档一致性检查 | CLI/GUI 联调与地址兼容 |

### 最小验收命令
- `test -f crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_game_launcher`
- `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher`
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-LCH-001 | 启动失败立即回滚所有子进程 | 保留部分进程供排查 | 优先保证重复执行可预测，排查通过日志完成。 |
| DEC-LCH-002 | ready 阶段强制健康联动与响应前缀校验 | 仅检测端口可连接 | 单纯连通性不足以证明服务真实可用。 |
| DEC-LCH-003 | IPv6 强制 bracket 规范 | 接受多种模糊写法 | 规范化可减少跨端解析歧义。 |

## 原文约束点映射（内容保真）
- 原“目标（清理缺陷、ready 假阳性、IPv6 兼容）” -> 第 1 章 Problem/Solution/SC。
- 原“范围（两个 launcher + 定向测试）” -> 第 2 章 AC 与第 4 章 Integration。
- 原“接口/数据（信号、回滚、探针、解析规则）” -> 第 2 章规格矩阵 + 第 4 章 Edge Cases。
- 原“里程碑 M1~M3” -> 第 5 章 Phased Rollout（LCH-1~4）。
- 原“风险（信号并测、IPv6 收紧）” -> 第 5 章 Technical Risks。
