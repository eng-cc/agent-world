# Non-Viewer 链上鉴权协议重构（生产级）设计

- 对应需求文档: `doc/headless-runtime/nonviewer/nonviewer-onchain-auth-protocol-hardening.prd.md`
- 对应项目管理文档: `doc/headless-runtime/nonviewer/nonviewer-onchain-auth-protocol-hardening.project.md`

## 1. 设计定位
定义 Non-Viewer 链上鉴权协议生产级重构方案，统一认证凭证、链上验证与运行时门禁。

## 2. 设计结构
- 鉴权模型层：定义链上身份、凭证和验证语义。
- 协议接线层：把鉴权协议接到 headless runtime 主路径。
- 门禁保护层：无效凭证或异常状态在入口直接阻断。
- 审计回归层：记录鉴权决策并固化生产回归。

## 3. 关键接口 / 入口
- 链上鉴权凭证
- 协议验证入口
- 运行时门禁检查
- 鉴权审计记录

## 4. 约束与边界
- 鉴权失败必须显式阻断。
- 生产协议路径需可观测、可回放。
- 不在本专题扩展更大范围身份系统。

## 5. 设计演进计划
- 先冻结协议模型。
- 再接运行时门禁。
- 最后沉淀审计与回归。
