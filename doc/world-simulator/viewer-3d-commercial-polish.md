# Agent World Viewer：3D 商业化精致度收敛（资产/材质/光照/后处理）

## 目标
- 把当前 Viewer 3D 从“调试可视化风格”提升到“可对外发行的商业化视觉基线”。
- 聚焦四个核心缺口并形成可配置、可回归、可演进的技术底座：
  - 1）资产层：从单一基础几何体扩展为可分档的资产表达能力。
  - 2）材质层：从单色/弱 PBR 过渡到一致的工业风 PBR 材质体系。
  - 3）光照层：从基础双方向光升级为可控三点光与阴影策略。
  - 4）后处理层：接入色调映射、去色带、Bloom 与基础色彩分级。

## 范围

### In Scope
- 新增 Viewer 3D 渲染档位（Debug/Balanced/Cinematic）与统一环境变量入口。
- 新增“资产管线配置层”：在不引入外部 DCC 流程的前提下，支持不同几何精度与可切换资产表达模式。
- 完成工业风 PBR 材质参数分层（主体、覆盖层、元素块、设施）。
- 完成三点光照（key/fill/rim）与阴影开关策略（按档位）。
- 完成后处理最小可发行链路：Tonemapping、DebandDither、Bloom、ColorGrading。
- 补齐 `agent_world_viewer` 单测与配置解析回归。

### Out of Scope
- 本期不引入完整外部美术内容生产管线（高模烘焙、动画资产、贴图工具链）。
- 本期不改 Viewer 协议与模拟内核语义。
- 本期不实现照片级渲染目标（PBR+后处理为“商业可发行基线”，非电影级）。

## 接口 / 数据

### 1) 渲染档位与统一配置
- 新增 `ViewerRenderProfile`：`debug | balanced | cinematic`。
- 新增环境变量：
  - `AGENT_WORLD_VIEWER_RENDER_PROFILE`
- 新增/扩展配置分组：
  - `assets`：几何精度、Location 壳层开关。
  - `materials`：碎片是否 unlit、发光增强、透明度。
  - `lighting`：阴影开关、环境亮度、fill/rim 比例。
  - `post_process`：Tonemapping、Bloom、DebandDither、ColorGrading。

### 2) 资产层（缺口 1）
- 引入“几何精度分档”：
  - Debug：低成本基础几何。
  - Balanced：中等细分。
  - Cinematic：高细分可发行观感。
- 新增 Location 壳层可选渲染路径（保持与 frag 分块兼容）。

### 3) 材质层（缺口 2）
- 统一工业风材质口径：
  - Agent/设施/资产：PBR 参数成组管理（base/roughness/metallic/emissive）。
  - 元素分块：支持“可读性优先”与“质感优先”两套策略切换。

### 4) 光照层（缺口 3）
- 光照结构：`key + fill + rim`。
- 光照强度：复用物理口径 `exposed_illuminance_lux`，再按 fill/rim 比例分配。
- 阴影策略：按档位默认值控制（Balanced 保守、Cinematic 强化）。

### 5) 后处理层（缺口 4）
- 相机接入组件：
  - `Tonemapping`
  - `DebandDither`
  - `Bloom`
  - `ColorGrading`
- 提供可控开关与参数，默认保证 Web/Native 双端可运行。

## 里程碑
- C3D-0：设计文档与项目管理文档。
- C3D-1：渲染档位 + 资产分档落地（含配置解析测试）。
- C3D-2：材质分层落地（含元素材质回归）。
- C3D-3：三点光照与阴影策略落地。
- C3D-4：后处理链路接入与回归。
- C3D-5：文档/手册/devlog 收口与验证。

## 风险
- 视觉增强带来性能波动：
  - 缓解：档位隔离 + 自动降级并存，默认保持 Balanced 稳定。
- Web 平台后处理兼容性风险：
  - 缓解：Web 默认保守参数，可通过配置降级关闭 Bloom。
- 材质升级影响可读性：
  - 缓解：保留 Debug/Balanced 风格，Cinematic 作为可选高保真档位。
- 渲染配置项增加导致运维复杂：
  - 缓解：统一 `AGENT_WORLD_VIEWER_RENDER_PROFILE` 作为主入口，其它变量为细化覆盖。
