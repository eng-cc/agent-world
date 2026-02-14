# Viewer 工业风可视化缺口收敛（项目管理文档）

## 任务拆解

### IV0 文档与挂载
- [x] IV0.1 输出设计文档（`doc/world-simulator/viewer-industrial-visual-closure.md`）
- [x] IV0.2 输出项目管理文档（本文件）
- [x] IV0.3 在总项目文档挂载分册入口

### IVA 产线 + 物流可视化（缺口 1、2）
- [x] IVA.1 产线摘要：工厂/配方/制成品可视实体统计与近期产线产出
- [x] IVA.2 物流路由：资源/电力传输路由聚合与吞吐摘要
- [x] IVA.3 右侧面板接入工业链路区块（支持中英文）
- [x] IVA.4 单测与回归（`agent_world_viewer`）

### IVB 经营面板（缺口 3）
- [x] IVB.1 经营指标口径定义（供需/成本/收益/库存健康）
- [x] IVB.2 面板落地与阈值提示
- [x] IVB.3 单测与回归

### IVC 多尺度导航 + 告警根因（缺口 5、6）
- [x] IVC.1 世界/区域/节点三级导航与快速聚焦
- [x] IVC.2 告警等级与根因链摘要
- [x] IVC.3 告警对象跳转联动与回归

### IVD 产品化表现层（缺口 7）
- [x] IVD.1 视觉层级与语义色板统一
- [x] IVD.2 关键状态动效/过渡（可开关）
- [x] IVD.3 截图闭环与视觉回归收口

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_observe_section_card.rs`
- `crates/agent_world_viewer/src/ui_text.rs`
- `crates/agent_world_viewer/src/ui_locale_text.rs`
- `crates/agent_world_viewer/src/world_overlay.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `doc/world-simulator/viewer-open-world-sandbox-readiness.md`

## 状态
- 当前阶段：IVD（产品化表现层）完成
- 下一阶段：工业风可视化缺口收敛任务 A~D 已全部收口，转入后续可玩化增量任务
- 最近更新：完成 IVD（语义色板卡片 + 可开关动效 + 截图闭环 + `agent_world_viewer` 全量回归 196 tests，2026-02-14）
