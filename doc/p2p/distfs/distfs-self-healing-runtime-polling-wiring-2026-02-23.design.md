# DistFS 自愈运行时接线设计

- 对应需求文档: `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.project.md`

## 1. 设计定位
定义自愈轮询在 Node Runtime 中的接线方案，让节点在 tick 中安全执行本地副本修复。

## 2. 设计结构
- Node 配置层：新增副本维护配置模型与校验。
- runtime 状态层：在 tick 中持有轮询状态和执行入口。
- 本地执行层：通过网络拉取与本地 CAS 写入完成修复。
- 降级容错层：缺依赖或单次失败只影响当前轮。

## 3. 关键接口 / 入口
- Node 副本维护配置
- runtime 轮询状态
- 本地目标执行器
- network_bridge/node_runtime_core 接线

## 4. 约束与边界
- 不改复制协议，只接现有控制面和 polling loop。
- tick 内执行必须受上限控制。
- 非法配置与缺失依赖要显式跳过或报错。
- test_tier_required 需要覆盖关键运行边界。

## 5. 设计演进计划
- 先补 NodeConfig 和 runtime 状态。
- 再接本地执行器。
- 最后通过回归和日志收口运行时闭环。
