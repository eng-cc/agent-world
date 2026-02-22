# Agent World Runtime：共识数值语义与原子状态转移硬化（15 点清单第二阶段）

## 目标
- 延续第一阶段“正确性优先”原则，继续收敛区块链/P2P 主链路中的数值语义，消除静默饱和带来的隐藏错误。
- 在长期运行场景下，确保关键计数与票权累加在越界时显式失败，不进入“看似成功但状态失真”的路径。
- 对齐既定 15 点清单的第二阶段优先项（1、4、8、11），形成可验证、可回归的工程闭环。

## 范围

### In Scope（第二阶段）
- Node Points 账本结算路径：
  - `NodePointsLedger::settle_epoch` 关键累加改为显式溢出错误（不再静默饱和）。
  - 覆盖 `awarded_points`、`cumulative_points`、`total_distributed_points`、`epoch_index` 等长期运行敏感字段。
  - 保证结算失败时不提交部分账本更新（原子性）。
- Node PoS 投票路径：
  - `node_pos::insert_attestation` 的 `approved_stake/rejected_stake` 累加改为显式溢出错误。
  - `node_pos::propose_next_head` 的 `next_slot` 递进改为显式溢出错误，禁止 `u64::MAX` 饱和停滞。
  - 保持提案/投票状态在失败路径不被污染。
- 测试：
  - 增加溢出拒绝与原子性测试。
  - 补齐 Node/Consensus 调用链编译与定向回归。

### Out of Scope（后续阶段）
- 全仓库统一引入大整数类型（BigInt/U256）替换所有 `u64/i64`。
- 全量共识消息 canonical 编码与签名域版本演进治理。
- 全链路形式化验证与跨节点确定性证明平台化。

## 接口/数据
- `NodePointsLedger::settle_epoch`：
  - 由直接返回 `EpochSettlementReport` 调整为 `Result<EpochSettlementReport, NodePointsError>`。
  - 新增溢出错误类型，明确失败原因（awarded/cumulative/epoch 等）。
- `NodePointsRuntimeCollector`：
  - 结算接口透传 `NodePointsError`，避免静默吞错。
- `node_pos`：
  - `insert_attestation` / `propose_next_head` 在关键 `checked_add` 失败时返回 `NodePosError`。
  - 保持现有调用方错误映射语义（`WorldError` / `NodeError`）不变。

## 里程碑
- M0：Phase2 文档建档，冻结边界。
- M1：Node Points 账本结算数值语义改造完成。
- M2：Node PoS 票权与 slot 递进溢出语义改造完成。
- M3：回归测试通过，文档与 devlog 收口。

## 风险
- 部分历史逻辑依赖“饱和后继续执行”，改造后会转为显式拒绝，需同步更新测试预期。
- `NodePointsLedger` 接口签名变化会影响 runtime collector 与调用链，需一次性改齐。
- 长期运行边界测试场景较极端，若用例不充分，仍可能遗漏非主路径越界点。

## 15 点清单映射（阶段视角）
- 本阶段优先覆盖：1、4、8、11。
- 与第一阶段形成连续闭环：第一阶段已覆盖 2、3、5、6、14（局部）。
- 后续阶段继续覆盖：7、9、10、12、13、15。

## 当前状态
- 截至 2026-02-22：M0、M1、M2、M3 已完成。
