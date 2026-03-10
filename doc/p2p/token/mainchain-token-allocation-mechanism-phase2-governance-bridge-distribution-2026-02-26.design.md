# Agent World 主链 Token 分配机制二期：地址绑定 + 治理绑定 + 分发闭环（2026-02-26）设计

- 对应需求文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.project.md`

## 1. 设计定位
定义主链 Token 分配机制二期的地址绑定、治理绑定与分发闭环设计，让主链分发与治理桥接形成可审计执行路径。

## 2. 设计结构
- 地址绑定层：为分配对象建立稳定地址映射与校验规则。
- 治理绑定层：把分发资格与治理身份/决议绑定。
- 分发执行层：按治理结果执行 token 分发与状态更新。
- 审计回滚层：记录分发证据、异常阻断与必要回滚信息。

## 3. 关键接口 / 入口
- 地址绑定模型
- 治理绑定记录
- token 分发执行入口
- 分发审计/回滚记录

## 4. 约束与边界
- 地址与治理身份绑定必须唯一且可追踪。
- 未满足治理条件不得发放。
- 不在本专题重写主 token 总量模型。

## 5. 设计演进计划
- 先补地址绑定。
- 再接治理桥接。
- 最后完成分发闭环和审计。
