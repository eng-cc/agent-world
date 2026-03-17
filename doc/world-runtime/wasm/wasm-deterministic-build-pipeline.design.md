# Agent World Runtime：WASM Docker 确定性构建与工件治理管线设计

- 对应需求文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.prd.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.project.md`

## 1. 设计定位
本设计把 Agent World 的 WASM 发布级构建从“宿主机护栏 + keyed hash 对账”升级为“Docker canonical builder”。设计目标不是否定现有脚本和 build suite，而是把它们放进同一个 pinned 容器镜像里，让宿主平台不再参与发布 hash 的生成。

## 2. 现状盘点

| 层级 | 当前实现事实（2026-03-17） | 与 Docker-first 目标的差距 |
| --- | --- | --- |
| 容器基础设施 | 仓库内当前没有 `Dockerfile`、`.dockerignore` 或专用容器构建脚本。 | 还没有真正的 canonical builder image。 |
| 构建入口 | `scripts/build-wasm-module.sh` 目前直接在 host 上固定 toolchain/env/path remap。 | 入口仍然依赖宿主机 Rust 安装与本地 shell 环境。 |
| 构建工具 | `tools/wasm_build_suite` 已做 `--locked`、workspace compile-time guard、canonical packaging。 | 工具本身可复用，但应转移到容器内执行。 |
| manifest 策略 | 现有 builtin manifest 以 keyed platform token 记录 `darwin-arm64` / `linux-x86_64`。 | 这是 drift 兜底，不是 drift 消除；Docker-first 的目标是只保留一个 canonical publish hash。 |
| source compile | `compile_module_artifact_from_source` 仍在 runtime 进程里直接调用 host builder。 | 这与“发布级构建必须走同一容器镜像”相冲突。 |
| CI 校验 | 现有 multi-runner workflow 比较不同宿主平台各自产出的 hash/identity。 | 应改为比较“不同宿主跑同一 Docker builder 的输出”是否一致。 |

## 3. 设计原则
- 原则-1：publish hash 只来自 canonical container，不来自宿主机。
- 原则-2：builder image digest 是构建身份的一部分，必须进入 receipt 与审计链路。
- 原则-3：runtime 节点是 binary consumer，不是 source builder。
- 原则-4：CI 验证 Docker reproducibility，不拥有生产 manifest 写权限。
- 原则-5：source compile 要么走同一外部 Docker builder，要么明确降级为 dev/test only。

## 4. 目标态架构

```text
repo source / ModuleSourcePackage
          |
          v
host wrapper
scripts/build-wasm-module.sh
          |
          v
docker run --platform linux/amd64
builder-image@sha256:...
          |
          v
containerized wasm_build_suite
          |
          v
canonical packaged wasm
          |
          +------------------------------+
          |                              |
          v                              v
     build receipt                identity / release evidence
          |                              |
          +--------------+---------------+
                         v
        single canonical publish hash
      (manifest token: linux-x86_64=<sha256>)
                         |
                         v
               DistFS / release manifest
                         |
                         v
              runtime fetch -> verify -> execute
```

关键变化：
- 宿主平台不再直接生成发布 hash。
- `linux-x86_64` 容器平台成为唯一 canonical publish 平台。
- keyed manifest 从“多宿主发布 hash 集合”收敛为“单 canonical token”。

## 5. 详细设计

### 5.1 Canonical Builder Image
目标新增一份固定 builder image，例如：
- `docker/wasm-builder/Dockerfile`
- 镜像通过 digest pin 引用，而不是仅用 tag。

镜像内必须固定：
- Rust toolchain 版本
- `rust-src`
- `wasm32-unknown-unknown`
- canonical packaging 所需依赖
- `tools/wasm_build_suite`
- 统一 workspace 路径，例如 `/workspace`

设计要求：
- builder image 是发布级构建环境的真源。
- `build_manifest_hash` 的输入至少包含：
  - `builder_image_digest`
  - `container_platform=linux-x86_64`
  - build profile
  - canonicalizer version
  - build suite version / contract version

### 5.2 Host Wrapper
`scripts/build-wasm-module.sh` 的职责要改写为：
- 校验 Docker 可用。
- 组装 `docker run` 参数。
- 绑定只读源码挂载与可写输出目录。
- 把 `module_id`、`manifest_path`、`profile` 转交给容器内 build suite。

约束改为：
- 不再保留 host-native fallback。
- Docker 不可用时直接失败，避免任何宿主直编结果进入发布链路。
- wrapper 只接受同一 workspace root 下的 `manifest_path + out_dir`，并统一映射到容器内 `/workspace`。

### 5.3 Containerized Build Suite
`tools/wasm_build_suite` 不需要被替换，但需要重新定位：
- 过去：host 工具。
- 未来：container-internal canonical builder。

保留能力：
- `cargo metadata/build --locked`
- workspace `build.rs` / `proc-macro` guard
- custom section stripping
- metadata 输出

新增能力：
- 输出 `build receipt`
- receipt 至少包含：
  - `builder_image_ref`
  - `builder_image_digest`
  - `container_platform`
  - `module_id`
  - `source_manifest_path`
  - `source_hash`
  - `build_manifest_hash`
  - `canonicalizer_version`
  - `wasm_hash_sha256`
  - `wasm_size_bytes`

### 5.4 Manifest / Identity Migration
当前 manifest 的主要问题是把宿主平台差异直接放进发布清单。

Docker-first 目标态：
- 发布级 manifest 只保留一个 token：
  - `linux-x86_64=<sha256>`
- 这里的 `linux-x86_64` 表示 canonical builder container platform，而不是要求宿主机必须是 Linux。

迁移策略：
1. 读路径先兼容多 token。
2. 写路径改为只写 canonical token。
3. CI 阻止新的 `darwin-arm64` 发布 token 进入仓库。
4. release manifest / identity / attestation 逐步只引用 canonical token。

Identity manifest 需要扩展：
- 从“source hash + build manifest hash + hash tokens”
- 升级为“source hash + build manifest hash + builder image digest + canonical token”

### 5.5 Source Package Compile Policy
这是本设计最大的边界调整。

当前问题：
- `compile_module_artifact_from_source` 在 runtime 进程里直接调用 host builder。
- 即使 builder script 以后转成 Docker wrapper，也意味着 runtime 生产节点默认要有 Docker daemon 权限。

目标态：
- 生产态 source compile 外移到发布节点或专用 builder worker。
- runtime 只接收已经构建好的 wasm artifact + build receipt。
- 如果短期不能外移，则 production config 必须默认禁用该路径，只允许 dev/test 显式开启。

推荐两段式流程：
1. runtime / 发布层提交 `ModuleSourcePackage`
2. external builder worker 调用 Docker canonical builder，返回：
   - `wasm_bytes`
   - `source_bundle_hash`
   - `build receipt`

这样 runtime 继续是 binary-first consumer。

### 5.6 CI And Release Verification
CI 需要改成比较容器输出，而不是比较 host-native 输出。

目标 workflow：
1. macOS runner 安装 Docker Desktop / 可用 Docker CLI。
2. Linux runner 安装 Docker。
3. 两边都运行同一个 wrapper script。
4. 两边 summary 都导出 canonical container hash。
5. compare job 要求两边 hash 完全一致。

release gate 需要新增的固定结论：
- `builder image digest matched`
- `canonical token matched`
- `docker-only path enforced`

### 5.7 Runtime Consumption
runtime 的最终消费模型不变，仍是 binary-first：
1. 从 release manifest / DistFS 获取 binary。
2. 校验 canonical hash。
3. 校验 identity / signature / receipt 绑定。
4. 注册 artifact 并执行。

真正变化的是：
- canonical hash 现在来自 Docker builder。
- runtime 不再需要解释多个宿主平台发布 hash。

## 6. 角色分工

| 角色 | 负责内容 |
| --- | --- |
| `producer_system_designer` | 明确“容器解决漂移、runtime 不再直接编译源码”的目标边界 |
| `wasm_platform_engineer` | builder image、wrapper、receipt、manifest migration 设计与实现 |
| `runtime_engineer` | source compile 外移或 gating、release manifest 消费、runtime 拒绝路径 |
| `qa_engineer` | multi-runner Docker compare、失败签名、门禁矩阵 |

## 7. 失败模型与阻断点

| 失败点 | 触发位置 | 阻断行为 |
| --- | --- | --- |
| Docker 不可用 | host wrapper | 直接失败，不允许回退成发布级 native build |
| image digest 未 pin 或不匹配 | wrapper / receipt verify | 阻断发布 |
| container output 与 tracked canonical token 不一致 | sync/check | 阻断并报告 `module_id + expected + actual` |
| macOS/Linux 跑同一容器得出不同 hash | CI compare | 阻断并归类为 builder reproducibility defect |
| production runtime 仍启用 host source compile | runtime config gate | 启动即拒绝或 action rejected |

## 8. 迁移计划
- M0：修正文档为 Docker-first 目标态。
- M1：新增 builder image 与 wrapper，保留读路径兼容。
- M2：manifest/identity 只写 canonical token。
- M3：runtime source compile 外移到 external builder 或 production 默认禁用。
- M4：CI / release gate 全量切换到 Docker reproducibility compare。

## 9. 设计边界
- Docker 只解决发布级构建确定性，不进入 wasm 执行期 sandbox。
- 本设计不要求立即删除现有 build suite；它是容器内核心执行器。
- 本设计不把 CI 变成生产发布者；CI 只是运行同一容器镜像做验证。
