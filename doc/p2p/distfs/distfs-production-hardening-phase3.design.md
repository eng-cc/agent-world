# DistFS 生产化增强 Phase 3 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase3.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase3.project.md`

## 1. 设计定位
定义 DistFS 生产化第三阶段方案：把 challenge 探测等能力进一步接入活跃链路，并为后续有状态调度打基础。

## 2. 设计结构
- challenge 能力层：扩展 DistFS challenge 探测和相关产物。
- runtime 接线层：把 phase3 增量能力接到活跃 runtime 路径。
- 测试回归层：补齐 challenge 与 runtime 路径测试。
- 增量从属层：继续作为 phase1 的 slave 文档。

## 3. 关键接口 / 入口
- `challenge.rs`
- runtime 接线入口
- phase3 回归测试
- phase1 主入口

## 4. 约束与边界
- 不在本阶段引入完整网络级 challenge 协调器。
- 重点是把增量能力接上生产链路，而不是重写 challenge 模型。
- phase3 文档仍只维护本阶段增量。
- runtime 接线要保持与既有 reward 路径兼容。

## 5. 设计演进计划
- 先补 challenge 增量能力。
- 再接 runtime/活跃链路。
- 最后通过回归收口 phase3。
