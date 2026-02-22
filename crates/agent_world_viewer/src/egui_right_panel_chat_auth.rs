use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ViewerAuthSigner {
    pub(super) player_id: String,
    pub(super) public_key: String,
    pub(super) private_key: String,
}

pub(super) fn resolve_required_env_trimmed<F>(get_env: &F, key: &str) -> Result<String, String>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(raw) = get_env(key) else {
        return Err(format!("{key} is not set"));
    };
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{key} is empty"));
    }
    Ok(value.to_string())
}

pub(super) fn resolve_viewer_player_id_from<F>(get_env: &F) -> Result<String, String>
where
    F: Fn(&str) -> Option<String>,
{
    match get_env(VIEWER_PLAYER_ID_ENV) {
        Some(raw) => {
            let value = raw.trim();
            if value.is_empty() {
                Err(format!("{VIEWER_PLAYER_ID_ENV} is empty"))
            } else {
                Ok(value.to_string())
            }
        }
        None => Ok(VIEWER_PLAYER_ID.to_string()),
    }
}

pub(super) fn resolve_viewer_auth_signer_from<F>(get_env: F) -> Result<ViewerAuthSigner, String>
where
    F: Fn(&str) -> Option<String>,
{
    let player_id = resolve_viewer_player_id_from(&get_env)?;
    let public_key = resolve_required_env_trimmed(&get_env, VIEWER_AUTH_PUBLIC_KEY_ENV)?;
    let private_key = resolve_required_env_trimmed(&get_env, VIEWER_AUTH_PRIVATE_KEY_ENV)?;
    Ok(ViewerAuthSigner {
        player_id,
        public_key,
        private_key,
    })
}

pub(super) fn resolve_viewer_auth_signer() -> Result<ViewerAuthSigner, String> {
    resolve_viewer_auth_signer_from(|key| std::env::var(key).ok())
}

pub(super) fn next_viewer_auth_nonce() -> Result<u64, String> {
    let nonce = VIEWER_AUTH_NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    if nonce == 0 {
        return Err("viewer auth nonce exhausted".to_string());
    }
    Ok(nonce)
}

pub(super) fn attach_prompt_control_apply_auth(
    request: &mut agent_world::viewer::PromptControlApplyRequest,
    signer: &ViewerAuthSigner,
    nonce: u64,
    intent: agent_world::viewer::PromptControlAuthIntent,
) -> Result<(), String> {
    request.player_id = signer.player_id.clone();
    request.updated_by = Some(signer.player_id.clone());
    request.public_key = Some(signer.public_key.clone());
    request.auth = None;
    let proof = agent_world::viewer::sign_prompt_control_apply_auth_proof(
        intent,
        request,
        nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )?;
    request.auth = Some(proof);
    Ok(())
}

pub(super) fn attach_agent_chat_auth(
    request: &mut agent_world::viewer::AgentChatRequest,
    signer: &ViewerAuthSigner,
    nonce: u64,
) -> Result<(), String> {
    request.player_id = Some(signer.player_id.clone());
    request.public_key = Some(signer.public_key.clone());
    request.auth = None;
    let proof = agent_world::viewer::sign_agent_chat_auth_proof(
        request,
        nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )?;
    request.auth = Some(proof);
    Ok(())
}

pub(super) fn sign_prompt_control_apply_request(
    request: &mut agent_world::viewer::PromptControlApplyRequest,
    intent: agent_world::viewer::PromptControlAuthIntent,
) -> Result<(), String> {
    let signer = resolve_viewer_auth_signer()?;
    let nonce = next_viewer_auth_nonce()?;
    attach_prompt_control_apply_auth(request, &signer, nonce, intent)
}

pub(super) fn sign_agent_chat_request(
    request: &mut agent_world::viewer::AgentChatRequest,
) -> Result<(), String> {
    let signer = resolve_viewer_auth_signer()?;
    let nonce = next_viewer_auth_nonce()?;
    attach_agent_chat_auth(request, &signer, nonce)
}

pub(super) fn sync_viewer_auth_nonce_from_state(state: &ViewerState) {
    let Ok(player_id) = resolve_viewer_player_id_from(&|key| std::env::var(key).ok()) else {
        return;
    };
    let Some(snapshot) = state.snapshot.as_ref() else {
        return;
    };
    let Some(last_nonce) = snapshot
        .model
        .player_auth_last_nonce
        .get(player_id.as_str())
    else {
        return;
    };

    let desired = last_nonce.saturating_add(1).max(1);
    let mut current = VIEWER_AUTH_NONCE_COUNTER.load(Ordering::SeqCst);
    while current < desired {
        match VIEWER_AUTH_NONCE_COUNTER.compare_exchange(
            current,
            desired,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}
