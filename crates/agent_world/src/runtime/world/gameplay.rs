use super::super::{
    ActiveGameplayModule, GameplayKindCoverage, GameplayModeReadiness, ModuleRegistry, ModuleRole,
    GAMEPLAY_BASELINE_KINDS,
};
use super::World;

impl World {
    pub fn gameplay_modules(&self) -> Vec<ActiveGameplayModule> {
        let mut out = Vec::new();
        for (module_id, version) in &self.module_registry.active {
            let key = ModuleRegistry::record_key(module_id, version);
            let Some(record) = self.module_registry.records.get(&key) else {
                continue;
            };
            if record.manifest.role != ModuleRole::Gameplay {
                continue;
            }
            let Some(gameplay) = &record.manifest.abi_contract.gameplay else {
                continue;
            };
            let mut game_modes = gameplay.game_modes.clone();
            game_modes.sort();
            out.push(ActiveGameplayModule {
                module_id: module_id.clone(),
                version: version.clone(),
                kind: gameplay.kind,
                game_modes,
            });
        }
        out.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        out
    }

    pub fn gameplay_mode_readiness(&self, mode: impl Into<String>) -> GameplayModeReadiness {
        let mode = mode.into();
        let mode = mode.trim().to_string();
        let mut active_modules = self
            .gameplay_modules()
            .into_iter()
            .filter(|module| module.game_modes.iter().any(|entry| entry == &mode))
            .collect::<Vec<_>>();
        active_modules.sort_by(|left, right| left.module_id.cmp(&right.module_id));

        let mut coverage = Vec::new();
        let mut missing_kinds = Vec::new();
        for kind in GAMEPLAY_BASELINE_KINDS {
            let active_count = active_modules
                .iter()
                .filter(|module| module.kind == kind)
                .count() as u32;
            if active_count == 0 {
                missing_kinds.push(kind);
            }
            coverage.push(GameplayKindCoverage { kind, active_count });
        }

        GameplayModeReadiness {
            mode,
            active_modules,
            coverage,
            missing_kinds,
        }
    }
}
