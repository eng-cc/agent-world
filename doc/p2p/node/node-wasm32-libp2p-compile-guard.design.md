# oasis7 Runtime：Node libp2p wasm32 编译兼容守卫设计

- 对应需求文档: `doc/p2p/node/node-wasm32-libp2p-compile-guard.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-wasm32-libp2p-compile-guard.project.md`

## 1. 设计定位
定义 Node libp2p 在 wasm32 目标下的编译兼容守卫，确保不支持能力被显式隔离且构建结果可预测。

## 2. 设计结构
- 目标识别层：识别 wasm32 目标与受限能力集合。
- 条件编译层：为 libp2p 相关模块添加明确 guard。
- 替代路径层：在受限目标下提供可接受的禁用或占位策略。
- 构建回归层：固化 wasm32 编译检查与失败签名。

## 3. 关键接口 / 入口
- wasm32 target guard
- libp2p 条件编译入口
- 受限能力占位路径
- wasm32 编译回归命令

## 4. 约束与边界
- 不支持的能力必须在编译期显式暴露。
- guard 不能影响非 wasm32 主路径。
- 不在本专题扩展 wasm 端完整网络实现。

## 5. 设计演进计划
- 先补目标 guard。
- 再整理替代路径。
- 最后执行 wasm32 编译回归。
