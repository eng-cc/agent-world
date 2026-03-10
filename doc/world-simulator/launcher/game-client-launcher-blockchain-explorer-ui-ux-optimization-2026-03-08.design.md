# 启动器区块链浏览器视觉与交互优化设计（2026-03-08）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.project.md`

## 1. 设计定位
定义 explorer 面板在不改后端协议前提下的 UI/UX 提升方案：增强概览层级、状态徽标、列表-详情联读、筛选恢复与请求状态可见性，提高 native/web 一致的排障效率。

## 2. 设计结构
- 概览卡片层：以链高度、节点信息、交易状态计数形成可扫描概览。
- 列表详情层：区块/交易视图支持双栏联读，点击列表即时刷新详情。
- 筛选恢复层：为筛选输入统一提供 Apply/Reset 动作。
- 状态提示层：请求中、链未就绪、空态等情况在窗口内直接可见。

## 3. 关键接口 / 入口
- explorer overview / blocks / txs / search 视图
- 状态徽标 `accepted/pending/confirmed/failed/timeout`
- `Apply/Reset` 筛选动作
- `explorer_window.rs` / `explorer_window_p1.rs`

## 4. 约束与边界
- 不新增 explorer 后端 API 或字段。
- native/web 继续复用同一 egui UI 逻辑。
- 列表顺序与状态枚举必须沿用现有接口语义。
- 本轮只提升可读性与操作效率，不重构核心状态机。

## 5. 设计演进计划
- 先重构概览与列表/详情信息结构。
- 再补筛选清空与请求状态提示。
- 最后通过 native/wasm 回归验证交互效率与一致性。
