# Agent World: Builtin Wasm 清单收敛为“每平台 1 个 Canonical Hash”（设计文档）

> 归档说明（2026-02-20）：该方案已被 `doc/p2p/builtin-wasm-identity-consensus.md` 取代，不再作为现行实现依据。

## 目标
- 将 builtin wasm hash 清单策略从“任意机器 hash 可累积”收敛为“每个受支持平台仅 1 个 canonical hash”。
- 在保持跨平台运行时可加载能力的同时，避免清单持续膨胀与来源不明 hash 混入。
- 保持 required gate 与本地 pre-commit 的 m1 校验一致且可判定。

## 范围

### In Scope
- 修改 m1 hash manifest 数据格式：
  - 从 `module_id hash1 hash2 ...`
  - 到 `module_id <platform>=<hash> ...`
- 修改 `scripts/sync-m1-builtin-wasm-artifacts.sh`：
  - `--check` 仅校验“当前平台”的 canonical hash；
  - 同步模式仅更新当前平台 hash，不再追加任意新 hash；
  - 校验平台键唯一且属于允许的平台集合。
- 修改 runtime/hydrator/测试对 manifest token 的解析：支持 `platform=hash`，并兼容 legacy 裸 hash token。
- 更新 m1 清单为平台键值格式。

### Out of Scope
- 扩展 m4 清单到完整多平台 canonical 数据（本次仅保证解析兼容）。
- 修改 wasm hash 算法与 DistFS 协议。
- 调整 CI job 拓扑。

## 接口 / 数据
- m1 manifest 行格式：
  - `module_id darwin-arm64=<sha256> linux-x86_64=<sha256>`
- 平台标识来源：`uname -s` + `uname -m` 归一化。
- 允许平台集合：默认 `darwin-arm64,linux-x86_64`（可通过环境变量扩展）。
- 入口脚本：`scripts/sync-m1-builtin-wasm-artifacts.sh`。

## 里程碑
- M1：设计文档与项目管理文档创建。
- M2：`sync-m1` 脚本实现平台 canonical 校验/更新策略。
- M3：runtime + hydrator + 测试解析兼容 `platform=hash`。
- M4：迁移 m1 清单数据并完成 required 回归。

## 风险
- 若当前平台不在允许集合，`sync/check` 会直接失败；需显式扩展平台集合。
- 清单格式迁移后，旧解析逻辑若未同步会导致运行时加载失败。
- 平台 key 命名约定需稳定，否则会造成重复平台槽位。
