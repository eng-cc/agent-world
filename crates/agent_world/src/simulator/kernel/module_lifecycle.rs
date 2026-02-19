use super::super::world_model::{InstalledModuleState, ModuleArtifactState};
use super::super::ModuleInstallTarget;
use super::types::{RejectReason, WorldEventKind};
use super::WorldKernel;
use crate::runtime::{compile_module_artifact_from_source, ModuleSourcePackage};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

impl WorldKernel {
    pub(super) fn apply_compile_module_artifact_from_source(
        &mut self,
        publisher_agent_id: String,
        module_id: String,
        manifest_path: String,
        source_files: BTreeMap<String, Vec<u8>>,
    ) -> WorldEventKind {
        if !self.model.agents.contains_key(&publisher_agent_id) {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: publisher_agent_id,
                },
            };
        }

        let module_id = module_id.trim().to_string();
        if module_id.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "compile module source rejected: module_id cannot be empty".to_string()
                    ],
                },
            };
        }

        let manifest_path = manifest_path.trim().to_string();
        if manifest_path.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "compile module source rejected: manifest_path cannot be empty".to_string(),
                    ],
                },
            };
        }

        if source_files.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "compile module source rejected: source_files cannot be empty".to_string(),
                    ],
                },
            };
        }

        let source_package = ModuleSourcePackage {
            manifest_path,
            files: source_files,
        };
        let compiled_bytes =
            match compile_module_artifact_from_source(module_id.as_str(), &source_package) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!("compile module source rejected: {err:?}")],
                        },
                    };
                }
            };
        let wasm_hash = sha256_hex(compiled_bytes.as_slice());

        self.apply_deploy_module_artifact_internal(
            publisher_agent_id,
            wasm_hash,
            compiled_bytes,
            Some(module_id),
        )
    }

    pub(super) fn apply_deploy_module_artifact(
        &mut self,
        publisher_agent_id: String,
        wasm_hash: String,
        wasm_bytes: Vec<u8>,
        module_id_hint: Option<String>,
    ) -> WorldEventKind {
        self.apply_deploy_module_artifact_internal(
            publisher_agent_id,
            wasm_hash,
            wasm_bytes,
            module_id_hint,
        )
    }

    pub(super) fn apply_install_module_from_artifact(
        &mut self,
        installer_agent_id: String,
        module_id: String,
        module_version: String,
        wasm_hash: String,
        activate: bool,
    ) -> WorldEventKind {
        if !self.model.agents.contains_key(&installer_agent_id) {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: installer_agent_id,
                },
            };
        }

        let module_id = module_id.trim().to_string();
        if module_id.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec!["install module rejected: module_id cannot be empty".to_string()],
                },
            };
        }

        let module_version = module_version.trim().to_string();
        if module_version.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "install module rejected: module_version cannot be empty".to_string()
                    ],
                },
            };
        }

        let wasm_hash = wasm_hash.trim().to_string();
        if wasm_hash.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec!["install module rejected: wasm_hash cannot be empty".to_string()],
                },
            };
        }

        let Some(artifact) = self.model.module_artifacts.get(&wasm_hash) else {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![format!(
                        "install module rejected: module artifact missing for hash {wasm_hash}"
                    )],
                },
            };
        };

        if artifact.publisher_agent_id != installer_agent_id {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![format!(
                        "install module rejected: installer {} is not artifact owner {}",
                        installer_agent_id, artifact.publisher_agent_id
                    )],
                },
            };
        }

        self.model.installed_modules.insert(
            module_id.clone(),
            InstalledModuleState {
                module_id: module_id.clone(),
                module_version: module_version.clone(),
                wasm_hash: wasm_hash.clone(),
                installer_agent_id: installer_agent_id.clone(),
                install_target: ModuleInstallTarget::SelfAgent,
                active: activate,
                installed_at_tick: self.time,
            },
        );

        WorldEventKind::ModuleInstalled {
            installer_agent_id,
            module_id,
            module_version,
            wasm_hash,
            install_target: ModuleInstallTarget::SelfAgent,
            active: activate,
        }
    }

    fn apply_deploy_module_artifact_internal(
        &mut self,
        publisher_agent_id: String,
        wasm_hash: String,
        wasm_bytes: Vec<u8>,
        module_id_hint: Option<String>,
    ) -> WorldEventKind {
        if !self.model.agents.contains_key(&publisher_agent_id) {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: publisher_agent_id,
                },
            };
        }

        let wasm_hash = wasm_hash.trim().to_string();
        if wasm_hash.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "deploy module artifact rejected: wasm_hash cannot be empty".to_string()
                    ],
                },
            };
        }
        if wasm_bytes.is_empty() {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![
                        "deploy module artifact rejected: wasm_bytes cannot be empty".to_string(),
                    ],
                },
            };
        }

        let computed_hash = sha256_hex(wasm_bytes.as_slice());
        if computed_hash != wasm_hash {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![format!(
                        "deploy module artifact rejected: hash mismatch expected={wasm_hash} got={computed_hash}"
                    )],
                },
            };
        }

        if let Some(existing) = self.model.module_artifacts.get(&wasm_hash) {
            if existing.wasm_bytes != wasm_bytes {
                return WorldEventKind::ActionRejected {
                    reason: RejectReason::RuleDenied {
                        notes: vec![format!(
                            "deploy module artifact rejected: hash {} already exists with different bytes",
                            wasm_hash
                        )],
                    },
                };
            }
        }

        self.model.module_artifacts.insert(
            wasm_hash.clone(),
            ModuleArtifactState {
                wasm_hash: wasm_hash.clone(),
                publisher_agent_id: publisher_agent_id.clone(),
                module_id_hint: module_id_hint.clone(),
                wasm_bytes: wasm_bytes.clone(),
                deployed_at_tick: self.time,
            },
        );

        WorldEventKind::ModuleArtifactDeployed {
            publisher_agent_id,
            wasm_hash,
            bytes_len: wasm_bytes.len() as u64,
            wasm_bytes,
            module_id_hint,
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
