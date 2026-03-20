# oasis7 Runtime：`oasis7_net` runtime_bridge 可编译闭环设计

- 对应需求文档: `doc/p2p/network/net-runtime-bridge-closure.prd.md`
- 对应项目管理文档: `doc/p2p/network/net-runtime-bridge-closure.project.md`

## 1. 设计定位
定义 `oasis7_net` 在 `runtime_bridge` feature 下的稳定编译闭环设计，确保 runtime bridge 不再依赖已失效的内部路径。

## 2. 设计结构
- 依赖收敛层：把 blob store、分片组装和协议类型统一切到稳定 crate 依赖。
- 桥接接口层：保留 `distributed_bootstrap`、`distributed_head_follow`、`distributed_observer_replay` 等既有公开语义。
- 编译闭环层：在 feature 打开时保证 bootstrap / observer / validation / execution storage 一起通过编译。
- 回归校验层：用定向编译与文档回写确保 runtime_bridge 路径持续可审计。

## 3. 关键接口 / 入口
- `distributed_bootstrap`
- `distributed_head_follow`
- `distributed_observer_replay`
- `distributed_storage::store_execution_result`
- `distributed_validation::{validate_head_update, assemble_snapshot, assemble_journal}`

## 4. 约束与边界
- 不在本专题新增协议语义、签名体系或 distfs 能力边界。
- 保持既有对外 API 语义不变，只修正内部依赖来源。
- 跨 crate 耦合先以可编译闭环为目标，后续再继续下沉抽象。

## 5. 设计演进计划
- 先收敛 `runtime_bridge` 导入路径到稳定 crate。
- 再修复 validation/result 等导出路径并完成 feature 编译。
- 最后把该路径纳入常规回归，降低后续漂移风险。
