# Agent World Runtime：节点密钥 config.toml 自举

审计轮次: 4
## 1. Executive Summary
- Problem Statement: 在节点启动时确保根目录 `config.toml` 存在可用的节点密钥字段（公钥/私钥）。
- Proposed Solution: 当字段缺失时自动生成一对密钥并写回 `config.toml`，形成“启动即自举”的最小闭环。
- Success Criteria:
  - SC-1: 不改变现有节点主循环行为，仅补配置层能力。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：节点密钥 config.toml 自举 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `world_viewer_live` 节点启动路径增加：
  - AC-2: 读取 `config.toml` 的 `[node]` 区块。
  - AC-3: 若 `private_key/public_key` 缺失，自动生成 ed25519 密钥对。
  - AC-4: 将密钥字段写回 `config.toml`。
  - AC-5: 保持已有 CLI 参数和节点启动流程兼容。
  - AC-6: 增加单元测试覆盖：
- Non-Goals:
  - 生产级密钥托管（HSM/KMS）。
  - 密钥轮换、吊销、分层权限策略。
  - 跨节点密钥注册与身份治理。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-keypair-config-bootstrap.prd.md`
  - `doc/p2p/node/node-keypair-config-bootstrap.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### config.toml 结构（草案）
```toml
[node]
private_key = "<hex>"
public_key = "<hex>"
```

### 启动流程（草案）
- `start_live_node` 调用 `ensure_node_keypair_in_config(config_path)`。
- 若 `node_enabled=false`，不触发密钥自举。

## 5. Risks & Roadmap
- Phased Rollout:
  - NKEY-1：设计文档与项目管理文档落地。
  - NKEY-2：实现读取/生成/写回逻辑并接入启动流程。
  - NKEY-3：补齐单元测试并完成回归。
  - NKEY-4：文档状态与 devlog 收口。
- Technical Risks:
  - 回写 `config.toml` 会重新序列化 TOML，可能丢失注释格式。
  - 若运行目录不可写，节点启动将报错，需要明确错误提示。
  - 已存在非法密钥格式时需要明确失败语义，避免静默覆盖。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-095-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-095-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
