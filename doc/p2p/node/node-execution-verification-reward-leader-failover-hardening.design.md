# Agent World Runtime：节点执行校验与奖励 Leader/Failover 生产化收口设计

- 对应需求文档: `doc/p2p/node/node-execution-verification-reward-leader-failover-hardening.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-execution-verification-reward-leader-failover-hardening.project.md`

## 1. 设计定位
定义节点执行校验、奖励发放与 leader/failover 切换的生产化收口设计，确保主备切换下奖励链路仍可追踪。

## 2. 设计结构
- 执行校验层：对执行结果做可重放、可验证校验。
- 奖励发放层：把校验通过的结果纳入奖励发放流程。
- leader/failover 切换层：在主备切换期间保持奖励状态连续。
- 生产回归层：以故障切换和奖励一致性为核心做回归。

## 3. 关键接口 / 入口
- 执行校验入口
- 奖励发放状态机
- leader/failover 控制面
- 故障切换回归用例

## 4. 约束与边界
- failover 期间不得出现重复发奖或漏发。
- 主备切换状态必须可观测、可回放。
- 不在本专题扩大到跨区域容灾编排。

## 5. 设计演进计划
- 先固化执行校验与奖励状态。
- 再补 leader/failover 切换守卫。
- 最后执行生产化回归。
