# Agent World：P2P 长跑持续 Chaos 注入（项目管理文档）

## 任务拆解
- [x] C0：方案建档
  - [x] 设计文档：`doc/testing/p2p-longrun-continuous-chaos-injection-2026-02-24.md`
  - [x] 项目管理文档：`doc/testing/p2p-longrun-continuous-chaos-injection-2026-02-24.project.md`
- [x] C1：实现持续注入调度核心
  - [x] 新增 continuous chaos CLI 参数解析与校验
  - [x] 注入循环支持固定计划 + 连续注入混合模式
- [x] C2：实现证据与统计扩展
  - [x] `run_config.json` 增加 continuous chaos 配置
  - [x] `summary.json` / `summary.md` 增加 plan/continuous/total 计数
- [x] C3：测试手册接线
  - [x] 更新 `testing-manual.md` S9（continuous chaos 示例与说明）
- [ ] C4：验证与收口
  - [ ] 执行一次短窗 continuous chaos 实跑
  - [ ] 回写 `doc/devlog/2026-02-24.md` 与状态收口

## 依赖
- `scripts/p2p-longrun-soak.sh`
- `testing-manual.md`
- `doc/testing/p2p-storage-consensus-longrun-online-stability-2026-02-24.md`

## 状态
- 当前阶段：C0/C1/C2/C3 已完成，C4 待开始。
- 阻塞项：无。
- 下一步：执行 C4（完成 continuous chaos 实跑与最终收口）。
- 最近更新：2026-02-24。
