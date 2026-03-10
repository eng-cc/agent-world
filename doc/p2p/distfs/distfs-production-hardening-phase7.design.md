# DistFS 生产化增强 Phase 7 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase7.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase7.project.md`

## 1. 设计定位
定义 DistFS 自适应 challenge 调度进一步收敛方案：在 phase6 基础上继续提高 reason-aware 调度与可观测性闭环。

## 2. 设计结构
- reason-aware 层：按失败原因分层调度与退避。
- 调度收敛层：让 probe runtime 更稳定地表达不同失败场景。
- 报告增强层：补齐与 reason-aware 调度相关的可观测字段。
- slave 文档层：维持 phase7 仅管理增量目标。

## 3. 关键接口 / 入口
- reason-aware 调度语义
- phase7 可观测字段
- probe runtime 接线
- phase7 回归测试

## 4. 约束与边界
- 不在本阶段新增新的 challenge 类型。
- phase7 延续 phase1 主入口，不重复定义通用基线。
- reason-aware 只做调度增强，不改变验证算法本身。
- 可观测字段需服务调优，不做无意义堆叠。

## 5. 设计演进计划
- 先冻结 phase7 增量目标。
- 再落 reason-aware 调度增强。
- 最后通过测试和项目状态收口。
