# Agent World Runtime：PoS 超多数比率边界数值语义硬化（15 点清单第七阶段）

## 目标
- 收口 PoS 配置校验中 `supermajority_numerator * 2` 的饱和乘法语义，避免极端大整数比率下出现“本应合法却被误拒绝”的隐性数值失真。
- 统一 `proto/consensus/node` 三处超多数比率判定逻辑，保证同一输入在各层行为一致。
- 补齐大整数边界测试，确保长期运行系统在极端参数下仍具确定性语义。

## 范围

### In Scope（第七阶段）
- `agent_world_proto::distributed_pos::required_supermajority_stake`：
  - 将 `numerator.saturating_mul(2) <= denominator` 改为无乘法溢出的判定方式（等价于 `numerator <= denominator / 2`）。
- `agent_world_consensus::PosConsensus::new`：
  - 同步采用无溢出超多数比率判定。
- `agent_world_node::pos_validation::validated_pos_state`：
  - 同步采用无溢出超多数比率判定。
- 测试：
  - 新增/更新大整数边界测试，覆盖 `denominator = u64::MAX`、`numerator = denominator / 2 + 1` 等场景。

### Out of Scope（后续阶段）
- PoS 其他统计性 `saturating_*`（如 epoch 回溯辅助语义）的大范围语义重构。
- 跨模块 stake 类型升级（如 newtype/U256）与链上编码兼容变更。
- 非 PoS 主路径的通用计数器饱和语义清理。

## 接口/数据
- 不改变公开结构体与函数签名：
  - `required_supermajority_stake(total_stake, numerator, denominator) -> Result<u64, String>`
  - `PosConsensus::new(config) -> Result<PosConsensus, WorldError>`
  - `validated_pos_state(...) -> Result<...>`
- 仅调整“> 1/2”判断实现方式，保证在超大整数输入下语义与数学定义一致。

## 里程碑
- M0：Phase7 建档并冻结范围。
- M1：proto/consensus/node 三处比率判定逻辑完成一致化改造。
- M2：边界测试通过并完成回归。
- M3：文档/devlog 收口，阶段完成。

## 风险
- 比率判定方式切换可能影响既有极端参数用例预期，需要同步更新测试断言。
- 若上层配置系统依赖旧的“误拒绝”行为，切换后会改变可接受配置集合。
- 需避免仅改单层导致跨层语义分裂（proto 与 node/consensus 不一致）。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 全部完成。
