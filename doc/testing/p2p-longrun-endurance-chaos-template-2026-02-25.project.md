# Agent World：P2P 长跑 180 分钟 Chaos 模板（项目管理文档）

## 任务拆解
- [x] C0：方案建档
  - [x] 设计文档：`doc/testing/p2p-longrun-endurance-chaos-template-2026-02-25.md`
  - [x] 项目管理文档：`doc/testing/p2p-longrun-endurance-chaos-template-2026-02-25.project.md`
- [x] C1：新增大规模固定 chaos 模板文件
  - [x] 新增 `doc/testing/chaos-plans/p2p-soak-endurance-full-chaos-v1.json`
  - [x] 覆盖 180 分钟窗口、多动作、多节点轮换
- [ ] C2：手册接线
  - [ ] 更新 `testing-manual.md` S9，提供模板化命令
  - [ ] 显式声明与 `test_tier_required/test_tier_full` 的语义边界
- [ ] C3：验证与收口
  - [ ] 执行短窗校验（schema + 可执行性）
  - [ ] 回写 `doc/devlog/2026-02-25.md` 与状态收口

## 依赖
- `scripts/p2p-longrun-soak.sh`
- `testing-manual.md`
- `doc/testing/p2p-longrun-continuous-chaos-injection-2026-02-24.md`

## 状态
- 当前阶段：C0/C1 已完成，C2 待开始。
- 阻塞项：无。
- 下一步：执行 C2，完成 S9 手册接线并明确语义边界。
- 最近更新：2026-02-25。
