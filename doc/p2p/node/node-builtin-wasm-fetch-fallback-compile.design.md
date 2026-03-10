# World Runtime：Builtin Wasm 先拉取后编译回退设计

- 对应需求文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.project.md`

## 1. 设计定位
定义 builtin wasm 运行路径的“先拉取、后编译、失败回退”设计，保证节点在缺失本地产物时仍能通过统一来源恢复执行能力。

## 2. 设计结构
- 远端拉取层：优先从标准分发来源获取 builtin wasm 产物与元数据。
- 本地编译回退层：拉取失败或版本不匹配时转入本地编译兜底。
- 缓存校验层：对获取到的 wasm 做版本/哈希校验并复用本地缓存。
- 启动收口层：把 fetch 与 compile 路径统一成单一节点启动入口。

## 3. 关键接口 / 入口
- builtin wasm 拉取入口
- 本地 wasm 编译入口
- 产物缓存/哈希校验
- 节点启动装载链路

## 4. 约束与边界
- 优先保证节点可启动与产物一致性，不先追求最快路径。
- 远端拉取失败必须有本地回退，不得让节点直接失能。
- 不在本专题扩展新的 wasm 发布协议。

## 5. 设计演进计划
- 先统一 fetch/compile 判定顺序。
- 再补齐缓存、校验与失败回退。
- 最后用定向编译与启动回归固化门禁。
