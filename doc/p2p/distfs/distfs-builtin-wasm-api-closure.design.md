# Builtin Wasm DistFS API 闭环设计

- 对应需求文档: `doc/p2p/distfs/distfs-builtin-wasm-api-closure.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-builtin-wasm-api-closure.project.md`

## 1. 设计定位
定义 builtin wasm 工件通过 `agent_world_distfs` API 完成写入、读取、hash 校验与脚本接线的统一路径。

## 2. 设计结构
- 存储层：`LocalCasStore` 提供可选 hash 算法策略。
- 工具层：hydrate 工具根据 manifest 与 built wasm 目录写入 blob。
- 运行时层：builtin wasm 读取统一经由 DistFS API 与校验逻辑。

## 3. 关键接口 / 入口
- `LocalCasStore::new_with_hash_algorithm` / `BlobStore::put|get|has`
- `hydrate_builtin_wasm` 工具入口
- runtime builtin wasm artifact 读取接口

## 4. 约束与边界
- 默认 `blake3` 行为不得被破坏。
- 缺失或 hash 校验失败必须返回带定位信息的错误。
- git 仍只跟踪 manifest，不直接追踪 wasm blob。

## 5. 设计演进计划
- 先补齐专题 Design 与互链。
- 再按 Project 任务拆解推进实现与测试闭环。
