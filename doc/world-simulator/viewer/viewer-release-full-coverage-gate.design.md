# Viewer 发布全覆盖门禁设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-release-full-coverage-gate.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-release-full-coverage-gate.project.md`

## 1. 设计定位
定义 Viewer 发布前的全覆盖门禁方案：把功能、视觉、连接态和闭环脚本纳入统一 gate，避免带着局部通过、整体未收敛的问题进入发布节奏。

## 2. 设计结构
- 功能覆盖层：固定 required/full 套件与 viewer 专项脚本的执行矩阵。
- 视觉门禁层：把截图、连接态和关键帧质量纳入硬门禁。
- 闭环执行层：统一 viewer 启动、桥接、截图和语义检查路径。
- 发布接入层：支持 nightly 或 release-candidate 人工门禁接入。

## 3. 关键接口 / 入口
- viewer 覆盖脚本入口
- 连接态/截图硬门禁
- `testing-manual.md`
- 发布节奏接入点

## 4. 约束与边界
- 本专题治理门禁和验证路径，不直接改渲染或玩法逻辑。
- 门禁必须强调连接态与视觉产物，不能只看编译/语义接口。
- 全覆盖并不替代现有 required/full，而是在其上增加发布约束。
- 人工门禁接入是后续使用方式，不是本轮自动化 CI 常驻要求。

## 5. 设计演进计划
- 先冻结覆盖矩阵和硬门槛。
- 再补视觉/连接态 gate。
- 最后把门禁结果接入发布节奏。
