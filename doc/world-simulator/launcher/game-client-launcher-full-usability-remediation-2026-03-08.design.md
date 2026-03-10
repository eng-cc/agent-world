# 启动器全量可用性修复设计（2026-03-08）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.project.md`

## 1. 设计定位
定义启动器围绕高频使用路径的全量可用性修复包，将配置、启停、反馈、转账、浏览器与错误提示中的剩余可用性缺口统一收敛到一轮 remediation 中。

## 2. 设计结构
- 主路径修复层：覆盖启动、停止、打开页面、状态理解等高频操作。
- 子能力补齐层：对反馈、转账、浏览器、配置编辑等次级路径进行一致性修补。
- 错误收敛层：统一错误卡片、禁用原因与下一步建议。
- 跨端收口层：native/web 通过同一状态与交互语义完成回归。

## 3. 关键接口 / 入口
- 主面板 CTA 与状态区
- 配置编辑与错误卡片
- 反馈/转账/浏览器子窗口
- native/web 共用 launcher 状态模型

## 4. 约束与边界
- remediation 只修正体验缺口，不改动底层业务协议。
- 所有修复项需优先服务高频路径，避免过度堆叠 UI 复杂度。
- native/web 必须保持行为和文案语义一致。
- 本轮不引入新技术栈或新的远端依赖。

## 5. 设计演进计划
- 先盘点高频路径阻碍项并分层归类。
- 再按主路径优先顺序补齐可用性缺口。
- 最后以跨端回归和文档回写收口 remediation 成果。
