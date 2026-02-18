use super::super::world_model::{InstalledModuleState, ModuleArtifactState};
use super::WorldKernel;
use sha2::{Digest, Sha256};

impl WorldKernel {
    pub(super) fn replay_module_artifact_deployed(
        &mut self,
        publisher_agent_id: &str,
        wasm_hash: &str,
        wasm_bytes: &[u8],
        bytes_len: u64,
        module_id_hint: Option<&str>,
    ) -> Result<(), String> {
        if !self.model.agents.contains_key(publisher_agent_id) {
            return Err(format!(
                "module artifact publisher not found: {}",
                publisher_agent_id
            ));
        }
        if wasm_hash.trim().is_empty() {
            return Err("module artifact hash cannot be empty".to_string());
        }
        if wasm_bytes.is_empty() {
            return Err("module artifact bytes cannot be empty".to_string());
        }
        if bytes_len != wasm_bytes.len() as u64 {
            return Err(format!(
                "module artifact bytes_len mismatch: expected {} got {}",
                bytes_len,
                wasm_bytes.len()
            ));
        }

        let computed_hash = sha256_hex(wasm_bytes);
        if computed_hash != wasm_hash {
            return Err(format!(
                "module artifact hash mismatch: expected {} got {}",
                wasm_hash, computed_hash
            ));
        }

        if let Some(existing) = self.model.module_artifacts.get(wasm_hash) {
            if existing.wasm_bytes != wasm_bytes {
                return Err(format!(
                    "module artifact hash {} already exists with different bytes",
                    wasm_hash
                ));
            }
        }

        self.model.module_artifacts.insert(
            wasm_hash.to_string(),
            ModuleArtifactState {
                wasm_hash: wasm_hash.to_string(),
                publisher_agent_id: publisher_agent_id.to_string(),
                module_id_hint: module_id_hint.map(|value| value.to_string()),
                wasm_bytes: wasm_bytes.to_vec(),
                deployed_at_tick: self.time,
            },
        );

        Ok(())
    }

    pub(super) fn replay_module_installed(
        &mut self,
        installer_agent_id: &str,
        module_id: &str,
        module_version: &str,
        wasm_hash: &str,
        active: bool,
    ) -> Result<(), String> {
        if !self.model.agents.contains_key(installer_agent_id) {
            return Err(format!(
                "module installer not found: {}",
                installer_agent_id
            ));
        }
        if module_id.trim().is_empty() {
            return Err("installed module_id cannot be empty".to_string());
        }
        if module_version.trim().is_empty() {
            return Err("installed module_version cannot be empty".to_string());
        }
        if wasm_hash.trim().is_empty() {
            return Err("installed module wasm_hash cannot be empty".to_string());
        }

        let Some(artifact) = self.model.module_artifacts.get(wasm_hash) else {
            return Err(format!(
                "installed module references missing artifact: {}",
                wasm_hash
            ));
        };
        if artifact.publisher_agent_id != installer_agent_id {
            return Err(format!(
                "installed module owner mismatch: installer {} artifact owner {}",
                installer_agent_id, artifact.publisher_agent_id
            ));
        }

        self.model.installed_modules.insert(
            module_id.to_string(),
            InstalledModuleState {
                module_id: module_id.to_string(),
                module_version: module_version.to_string(),
                wasm_hash: wasm_hash.to_string(),
                installer_agent_id: installer_agent_id.to_string(),
                active,
                installed_at_tick: self.time,
            },
        );

        Ok(())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
