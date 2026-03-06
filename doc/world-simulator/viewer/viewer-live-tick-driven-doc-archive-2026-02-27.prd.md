# Viewer Live 旧 Tick 驱动文档归档（2026-02-27）

审计轮次: 4
- 对应项目管理文档: doc/world-simulator/viewer/viewer-live-tick-driven-doc-archive-2026-02-27.prd.project.md

## 1. Executive Summary
- 将已下线的 Viewer Live 旧 tick 驱动（TimerPulse / tick-driven 兼容阶段）设计与项目管理文档完成清理与替代链收口。
- 保持 `doc/world-simulator/` 下仅保留当前有效的 event_drive 收口阶段文档，降低误读成本。

## 2. User Experience & Functionality
- 归档对象：Viewer Live Phase 1~7 的设计文档与对应项目管理文档（已从仓库移除旧阶段文档）。

不在范围内：
- Phase 8~10 文档（已收口到 event_drive 语义，仍作为当前有效阶段记录保留）。
- 历史 devlog。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 历史专题索引新增条目：标注清理原因与替代文档。

## 5. Risks & Roadmap
1. M0：建档（本设计文档 + 项目管理文档）。
2. M1：清理旧 tick 驱动阶段文档并收口替代链（不再保留 archive 目录）。
3. M2：更新历史专题索引并完成收口。

### Technical Risks
- 若遗漏成对 `.md/.project.md` 文件，可能造成阶段记录不完整。
- 若误归档当前有效阶段文档，会影响后续追踪。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
