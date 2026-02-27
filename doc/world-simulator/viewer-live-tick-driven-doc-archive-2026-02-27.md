# Viewer Live 旧 Tick 驱动文档归档（2026-02-27）

## 目标
- 将已下线的 Viewer Live 旧 tick 驱动（TimerPulse / tick-driven 兼容阶段）设计与项目管理文档归档。
- 保持 `doc/world-simulator/` 下仅保留当前有效的 event_drive 收口阶段文档，降低误读成本。

## 范围
- 归档对象：Viewer Live Phase 1~7 的设计文档与对应项目管理文档。
- 归档位置：`doc/world-simulator/archive/`。
- 更新归档索引：`doc/world-simulator/archive/README.md`。

不在范围内：
- Phase 8~10 文档（已收口到 event_drive 语义，仍作为当前有效阶段记录保留）。
- 历史 devlog。

## 接口/数据
- 文档文件路径变更：从 `doc/world-simulator/` 移动到 `doc/world-simulator/archive/`。
- 归档索引新增条目：标注归档原因与替代文档。

## 里程碑
1. M0：建档（本设计文档 + 项目管理文档）。
2. M1：移动旧 tick 驱动阶段文档到 archive。
3. M2：更新 archive README 并完成收口。

## 风险
- 若遗漏成对 `.md/.project.md` 文件，可能造成阶段记录不完整。
- 若误归档当前有效阶段文档，会影响后续追踪。
