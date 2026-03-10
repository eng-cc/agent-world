# DistFS 生产化增强 Phase 5 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase5.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase5.project.md`

## 1. 设计定位
定义 DistFS challenge probe 参数治理化与模块化拆分方案：把硬编码参数提升为运行时可配置项，并增强 epoch 报告可观测性。

## 2. 设计结构
- 参数治理层：把 probe 强度、TTL、时钟偏移等暴露成 CLI/runtime 配置。
- 模块化拆分层：把 DistFS probe runtime 从主文件拆到独立子模块。
- 可观测层：在 epoch 报告中输出 probe config、cursor state 和 challenge report。
- 兼容默认层：保留合理默认值并对参数做强校验。

## 3. 关键接口 / 入口
- `DistfsProbeRuntimeConfig`
- probe runtime 子模块
- CLI `--reward-distfs-probe-*` 参数
- epoch 报告新增字段

## 4. 约束与边界
- 不做链上动态参数下发或集中式 coordinator。
- 模块拆分目标是控行数和可维护性，不改变 challenge 语义。
- 报告字段增加要控制在可接受 I/O 范围。
- 参数治理要有默认值和校验兜底，避免配置爆炸。

## 5. 设计演进计划
- 先把 probe 参数治理化。
- 再拆 runtime 模块。
- 最后通过报告字段和回归测试收口 phase5。
