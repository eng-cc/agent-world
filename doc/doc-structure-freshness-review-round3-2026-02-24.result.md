# Doc 目录结构与内容时效复核（Round 3）结果文档

## 复核范围
- 全量文档：`doc/**/*.md`
- 目录结构焦点：`doc/` 顶层混放文档、专题目录聚类、归档边界
- 本轮执行日期：2026-02-24

## 结构调整结果
- 顶层文档数量由 35 收敛到 16（保留入口/治理文档为主）。
- 新增目录：
  - `doc/nonviewer/`
  - `doc/nonviewer/archive/`
  - `doc/engineering/`
  - `doc/engineering/archive/`

### 迁移到主题目录（活跃文档）
- `doc/indirect-control-tick-lifecycle-long-term-memory*.md` -> `doc/world-simulator/`
- `doc/resource-kind-compound-hardware-hard-migration*.md` -> `doc/world-simulator/`
- `doc/viewer-visual-upgrade*.md` -> `doc/world-simulator/`
- `doc/nonviewer-onchain-auth-protocol-hardening*.md` -> `doc/nonviewer/`
- `doc/nonviewer-longrun-traceable-memory-archive-hardening-2026-02-23*.md` -> `doc/nonviewer/`
- `doc/oversized-rust-file-splitting-round3-2026-02-23*.md` -> `doc/engineering/`

### 本轮归档（老旧文档）
- `doc/nonviewer/archive/nonviewer-release-readiness-hardening*.md`
- `doc/nonviewer/archive/nonviewer-longrun-memory-safety-traceability-2026-02-23*.md`
- `doc/engineering/archive/oversized-rust-file-splitting*.md`
- `doc/engineering/archive/oversized-rust-file-splitting-round2-2026-02-23*.md`

> 以上归档文件均新增了归档警示头与归档日期（2026-02-24）。

## 引用与规范校验
- 迁移路径在非 `doc/devlog` 文档中的旧路径残留：`0`
- 非 `doc/devlog` 文档超 500 行：`0`
- `doc/README.md` 已同步 round3 治理入口与新增目录分层。

## 结论
- 目录结构：`通过`（顶层收敛、主题分层清晰）
- 归档边界：`通过`（活跃/历史文档分离明确）
- 后续建议：新增 non-viewer/engineering 文档时默认落在对应子目录，避免再次回流到 `doc/` 顶层。
