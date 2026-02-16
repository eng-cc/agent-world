# Agent World Runtime：`agent_world_net` runtime_bridge 可编译闭环（设计文档）

## 目标
- 让 `agent_world_net --features runtime_bridge` 在当前拆分架构下可独立编译通过。
- 清理 `runtime_bridge` 路径对已删除 runtime 内部模块路径的依赖，改为稳定 crate 依赖。
- 保持既有对外 API 语义不变（bootstrap / head_follow / observer / replay / validation / execution_storage）。

## 范围

### In Scope
- `agent_world_net` runtime_bridge 相关模块导入收敛：
  - `bootstrap.rs`
  - `execution_storage.rs`
  - `head_follow.rs`
  - `head_validation.rs`
  - `observer.rs`
  - `observer_replay.rs`
- `Cargo.toml` 增加 runtime_bridge 所需依赖并挂到 feature。
- 修复 `HeadValidationResult` 导出路径，消除循环导入。
- 回归验证：
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_net --features runtime_bridge`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_net --features "runtime_bridge,libp2p"`

### Out of Scope
- 新增分布式协议语义。
- 改造签名体系、共识机制或 distfs 能力边界。
- 新增节点编排可执行程序。

## 接口 / 数据
- 保持 `agent_world_net` 现有公开 API 不变：
  - `distributed_bootstrap`
  - `distributed_head_follow`
  - `distributed_observer_replay`
  - `distributed_storage::store_execution_result`
  - `distributed_validation::{validate_head_update, assemble_snapshot, assemble_journal}`
- 依赖来源调整：
  - BlobStore/分片组装：`agent_world_distfs`
  - World/Snapshot/Journal/事件类型：`agent_world::runtime`
  - 协议类型：`agent_world_proto`

## 里程碑
- RB1：完成文档与任务拆解。
- RB2：完成 runtime_bridge 导入和 feature 依赖收敛，编译通过。
- RB3：完成定向回归并更新项目状态与开发日志。

## 风险
- `agent_world_net -> agent_world` 依赖可能增加层级耦合，需要后续继续下沉抽象。
- runtime_bridge 代码路径与默认路径存在长期漂移风险，需要后续纳入常规 CI 覆盖。
