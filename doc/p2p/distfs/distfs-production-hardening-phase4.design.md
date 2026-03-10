# DistFS 生产化增强 Phase 4 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase4.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase4.project.md`

## 1. 设计定位
定义 DistFS challenge 探测有状态调度方案：通过 cursor 和累计统计避免每轮重复命中同一批 blob，提升覆盖率与公平性。

## 2. 设计结构
- 调度状态层：`StorageChallengeProbeCursorState` 维护 cursor、轮次和累计统计。
- 有状态探测层：`probe_storage_challenges_with_cursor(...)` 按 cursor 轮转选择 blob。
- 持久化接线层：reward runtime 启动加载、每轮更新并原子写回调度状态。
- 容错恢复层：状态文件损坏时回退默认状态但不阻断服务。

## 3. 关键接口 / 入口
- `StorageChallengeProbeCursorState`
- `probe_storage_challenges_with_cursor(...)`
- `reward-runtime-distfs-probe-state.json`
- reward runtime 生命周期接线

## 4. 约束与边界
- 不做跨节点统一 challenge coordinator。
- 不进入 ZK/PoRep/PoSt 协议升级。
- 每轮一写的 I/O 开销当前可接受，后续再谈节流优化。
- 状态化调度不能影响 reward settlement 主链路可用性。

## 5. 设计演进计划
- 先定义调度状态模型。
- 再落有状态探测接口。
- 最后接 reward runtime 持久化并通过回归收口。
