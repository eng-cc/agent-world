# Agent World Simulator：Viewer Location 细粒度渲染（项目管理文档）

## 任务拆解
- [x] LFR1：输出设计文档与项目管理文档
- [x] LFR2：Location 细粒度渲染实现（主球体 + 细节子节点）
- [x] LFR3：新增 `asteroid_fragment_detail_bootstrap` 场景并接入解析/测试
- [x] LFR4：脚本场景白名单同步与截图闭环验证
- [x] LFR5：回归测试、文档收口、devlog 更新

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world/src/simulator/scenario.rs`
- `crates/agent_world/scenarios/*.json`
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：已完成
- 最近更新：完成 LFR5（测试通过 + 截图闭环 + 文档收口，2026-02-10）
