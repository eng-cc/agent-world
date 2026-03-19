# oasis7 Runtime：WASM SDK Wire 类型收敛设计

- 对应需求文档: `doc/world-runtime/wasm/wasm-sdk-wire-types-dedup.prd.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-sdk-wire-types-dedup.project.md`

## 1. 设计定位
定义 WASM SDK Wire 类型收敛设计，避免协议/SDK/宿主之间重复定义和转换漂移。

## 2. 设计结构
- 类型统一层：收敛 wire types 到单一权威定义。
- 转换适配层：减少宿主、协议和 SDK 之间的重复转换。
- 版本兼容层：为旧类型路径提供迁移和兼容策略。
- 回归校验层：校验序列化、反序列化与跨边界一致性。

## 3. 关键接口 / 入口
- 权威 wire types 定义
- SDK/协议转换入口
- 兼容迁移层
- 序列化回归用例

## 4. 约束与边界
- 同一语义不得维护多份并行类型定义。
- 类型迁移需要兼顾已有模块兼容。
- 不在本专题改写所有上层协议。

## 5. 设计演进计划
- 先识别重复类型。
- 再收敛权威定义与转换。
- 最后固定兼容和回归。
