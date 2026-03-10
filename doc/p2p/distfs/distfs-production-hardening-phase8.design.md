# DistFS 生产化增强 Phase 8 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase8.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase8.project.md`

## 1. 设计定位
定义 reason-aware 退避策略 multiplier 完整治理化方案：把按失败原因分层的倍率参数全部暴露到 CLI/runtime。

## 2. 设计结构
- 参数外显层：为 hash mismatch、missing sample、timeout、io error、signature invalid、unknown 等失败原因增加 multiplier 参数。
- runtime 接线层：把 CLI 参数接到 `DistfsProbeRuntimeConfig.adaptive_policy`。
- 可观测层：补齐参数解析和运行时配置可观测字段。
- 主入口控行层：通过模块化保持帮助文本和解析逻辑可维护。

## 3. 关键接口 / 入口
- `--reward-distfs-adaptive-multiplier-*`
- `DistfsProbeRuntimeConfig.adaptive_policy`
- 参数解析测试
- 配置可观测测试

## 4. 约束与边界
- 不做链上动态参数下发和多模板自动切换。
- 参数治理化不能突破 `backoff_max_ms` 等上限保护。
- 重点是把已有策略外显，而不是引入新调度模型。
- 帮助文本膨胀要靠模块化和测试维持可维护性。

## 5. 设计演进计划
- 先补 multiplier CLI 参数。
- 再接 runtime adaptive_policy。
- 最后通过参数解析和回归测试收口 phase8。
