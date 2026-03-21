# 启动器反馈分布式提交设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.project.md`

## 1. 设计定位
定义启动器反馈从“仅本地落盘”升级为“分布式提交优先、失败回落本地保存”的双路径闭环，使反馈先进入链运行时与 DistFS/P2P 网络，再以本地 JSON 作为兜底。

## 2. 设计结构
- 远端提交流层：`oasis7_chain_runtime` 提供 `POST /v1/chain/feedback/submit`，封装并签名 `FeedbackCreateRequest`。
- 启动器代理层：桌面启动器优先调用远端接口提交反馈。
- 本地回落层：远端失败时自动生成本地 JSON 包，并保留远端错误签名。
- 状态反馈层：UI 区分“已提交到分布式网络”和“已本地保存回落”两类结果。

## 3. 关键接口 / 入口
- `POST /v1/chain/feedback/submit`
- `NodeRuntime::submit_feedback`
- `feedback_p2p`
- `submit_feedback_with_fallback`
- `feedback_id/event_id`

## 4. 约束与边界
- 分布式提交优先，但必须保留本地保存兜底路径。
- 远端错误签名不能在回落时丢失，便于排障。
- 本阶段不扩展附件上传、历史查询或额外业务协议。
- 链运行时与启动器 JSON 字段名必须保持一致，避免协议漂移。

## 5. 设计演进计划
- 先接通 runtime 反馈提交接口并默认启用 `feedback_p2p`。
- 再将启动器提交流程改为远端优先、失败回落。
- 最后补回归测试，固定连接拒绝等失败路径的可追溯行为。
