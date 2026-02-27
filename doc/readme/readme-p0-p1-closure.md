# README 对齐收口：P0/P1 设计文档

## 目标
- 对齐 `README.md` 的三条核心承诺：
  - 共识提交与执行状态可验证绑定。
  - 在线可视化主路径不再可无约束绕过共识进度。
  - Agent 可通过世界内动作完成 WASM 模块部署/安装闭环。
- 在不破坏当前主流程的前提下，提供最小可用且可测试的实现。

## 范围
- P0：`agent_world_node` 共识 commit/gossip/replication 链路增加执行哈希绑定字段并签名。
- P1-A：`world_viewer_live` 增加默认启用的“共识执行高度门控”运行模式。
- P1-B：`agent_world::runtime` 新增世界内模块部署/安装动作，打通提案-影子-批准-应用闭环。

不在本次范围：
- 完整“游戏内 Rust 编译器”实现；仍以链下构建后将 wasm 工件提交到世界为边界。
- 重写 viewer 在线协议；仅增加可选兼容字段和门控行为。

## 接口 / 数据
- `agent_world_node`:
  - `GossipCommitMessage` 新增：
    - `execution_block_hash: Option<String>`
    - `execution_state_root: Option<String>`
  - commit 签名载荷增加上述字段。
  - `ReplicatedCommitPayload` 新增上述字段并入 `consensus/commits/*.json`。
- `world_viewer_live`:
  - `ViewerLiveServerConfig` 增加可选高度门控输入（按共识/执行高度限制本地 step）。
  - `world_viewer_live` 启动参数新增是否强制门控相关开关（默认强制）。
- `runtime::Action`:
  - 新增 `DeployModuleArtifact`（注册 wasm 工件）。
  - 新增 `InstallModuleFromArtifact`（基于 artifact 构建 ModuleChangeSet 并执行治理闭环）。

## 里程碑
- M1（P0）：共识 commit/gossip/replication 带 execution 哈希并可验签。
- M2（P1-A）：viewer live 默认按节点共识执行高度门控。
- M3（P1-B）：runtime 动作支持“部署 + 安装”模块闭环。

## 风险
- 兼容性风险：commit 签名载荷扩展后，旧版本签名消息可能无法互验。
- 语义风险：viewer 门控默认开启可能改变历史“纯本地模拟”调试体验。
- 安全风险：模块安装动作若校验不足可能放大错误配置影响，需要复用现有 shadow/apply 校验。
