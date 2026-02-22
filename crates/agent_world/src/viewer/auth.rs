use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;

use super::protocol::{
    AgentChatRequest, PlayerAuthProof, PlayerAuthScheme, PromptControlApplyRequest,
    PromptControlRollbackRequest,
};

const VIEWER_PLAYER_AUTH_PAYLOAD_VERSION: u8 = 1;
pub const VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX: &str = "awviewauth:v1:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptControlAuthIntent {
    Preview,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPlayerAuth {
    pub player_id: String,
    pub public_key: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PromptFieldMode {
    Unchanged,
    Clear,
    Set,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PromptFieldPatch<'a> {
    mode: PromptFieldMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PromptControlApplySigningPayload<'a> {
    operation: &'static str,
    agent_id: &'a str,
    player_id: &'a str,
    public_key: &'a str,
    nonce: u64,
    expected_version: Option<u64>,
    updated_by: Option<&'a str>,
    system_prompt_override: PromptFieldPatch<'a>,
    short_term_goal_override: PromptFieldPatch<'a>,
    long_term_goal_override: PromptFieldPatch<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PromptControlRollbackSigningPayload<'a> {
    operation: &'static str,
    agent_id: &'a str,
    player_id: &'a str,
    public_key: &'a str,
    nonce: u64,
    to_version: u64,
    expected_version: Option<u64>,
    updated_by: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AgentChatSigningPayload<'a> {
    operation: &'static str,
    agent_id: &'a str,
    player_id: &'a str,
    public_key: &'a str,
    nonce: u64,
    message: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ViewerPlayerAuthSigningEnvelope<'a, T>
where
    T: Serialize,
{
    version: u8,
    payload: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor: Option<&'a str>,
}

pub fn sign_prompt_control_apply_auth_proof(
    intent: PromptControlAuthIntent,
    request: &PromptControlApplyRequest,
    nonce: u64,
    signer_public_key_hex: &str,
    signer_private_key_hex: &str,
) -> Result<PlayerAuthProof, String> {
    if nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let player_id =
        normalize_required_field(request.player_id.as_str(), "prompt_control player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "prompt_control public_key",
    )?;
    let signer_public_key =
        normalize_public_key_field(signer_public_key_hex, "prompt_control signer public key")?;
    if signer_public_key != request_public_key {
        return Err("prompt_control public_key does not match signer public key".to_string());
    }

    let signing_key =
        signing_key_from_hex(signer_private_key_hex, "prompt_control signer private key")?;
    verify_keypair_match(
        &signing_key,
        signer_public_key.as_str(),
        "prompt_control signer public key",
    )?;

    let signing_payload = build_prompt_control_apply_signing_payload(
        intent,
        request,
        player_id.as_str(),
        request_public_key.as_str(),
        nonce,
    )?;
    sign_player_auth_proof(
        signing_key,
        player_id,
        signer_public_key,
        nonce,
        signing_payload,
    )
}

pub fn verify_prompt_control_apply_auth_proof(
    intent: PromptControlAuthIntent,
    request: &PromptControlApplyRequest,
    proof: &PlayerAuthProof,
) -> Result<VerifiedPlayerAuth, String> {
    verify_proof_scheme(proof)?;
    let request_player_id =
        normalize_required_field(request.player_id.as_str(), "prompt_control player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "prompt_control public_key",
    )?;
    let proof_player_id =
        normalize_required_field(proof.player_id.as_str(), "auth proof player_id")?;
    let proof_public_key =
        normalize_public_key_field(proof.public_key.as_str(), "auth proof public key")?;
    if request_player_id != proof_player_id {
        return Err("auth proof player_id does not match request player_id".to_string());
    }
    if request_public_key != proof_public_key {
        return Err("auth proof public_key does not match request public_key".to_string());
    }
    if proof.nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let signing_payload = build_prompt_control_apply_signing_payload(
        intent,
        request,
        proof_player_id.as_str(),
        proof_public_key.as_str(),
        proof.nonce,
    )?;
    verify_player_auth_signature(
        proof_public_key.as_str(),
        proof.signature.as_str(),
        signing_payload.as_slice(),
    )?;
    Ok(VerifiedPlayerAuth {
        player_id: proof_player_id,
        public_key: proof_public_key,
        nonce: proof.nonce,
    })
}

pub fn sign_prompt_control_rollback_auth_proof(
    request: &PromptControlRollbackRequest,
    nonce: u64,
    signer_public_key_hex: &str,
    signer_private_key_hex: &str,
) -> Result<PlayerAuthProof, String> {
    if nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let player_id =
        normalize_required_field(request.player_id.as_str(), "prompt_control player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "prompt_control public_key",
    )?;
    let signer_public_key =
        normalize_public_key_field(signer_public_key_hex, "prompt_control signer public key")?;
    if signer_public_key != request_public_key {
        return Err("prompt_control public_key does not match signer public key".to_string());
    }

    let signing_key =
        signing_key_from_hex(signer_private_key_hex, "prompt_control signer private key")?;
    verify_keypair_match(
        &signing_key,
        signer_public_key.as_str(),
        "prompt_control signer public key",
    )?;

    let signing_payload = build_prompt_control_rollback_signing_payload(
        request,
        player_id.as_str(),
        request_public_key.as_str(),
        nonce,
    )?;
    sign_player_auth_proof(
        signing_key,
        player_id,
        signer_public_key,
        nonce,
        signing_payload,
    )
}

pub fn verify_prompt_control_rollback_auth_proof(
    request: &PromptControlRollbackRequest,
    proof: &PlayerAuthProof,
) -> Result<VerifiedPlayerAuth, String> {
    verify_proof_scheme(proof)?;
    let request_player_id =
        normalize_required_field(request.player_id.as_str(), "prompt_control player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "prompt_control public_key",
    )?;
    let proof_player_id =
        normalize_required_field(proof.player_id.as_str(), "auth proof player_id")?;
    let proof_public_key =
        normalize_public_key_field(proof.public_key.as_str(), "auth proof public key")?;
    if request_player_id != proof_player_id {
        return Err("auth proof player_id does not match request player_id".to_string());
    }
    if request_public_key != proof_public_key {
        return Err("auth proof public_key does not match request public_key".to_string());
    }
    if proof.nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let signing_payload = build_prompt_control_rollback_signing_payload(
        request,
        proof_player_id.as_str(),
        proof_public_key.as_str(),
        proof.nonce,
    )?;
    verify_player_auth_signature(
        proof_public_key.as_str(),
        proof.signature.as_str(),
        signing_payload.as_slice(),
    )?;
    Ok(VerifiedPlayerAuth {
        player_id: proof_player_id,
        public_key: proof_public_key,
        nonce: proof.nonce,
    })
}

pub fn sign_agent_chat_auth_proof(
    request: &AgentChatRequest,
    nonce: u64,
    signer_public_key_hex: &str,
    signer_private_key_hex: &str,
) -> Result<PlayerAuthProof, String> {
    if nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let player_id =
        normalize_required_optional_field(request.player_id.as_deref(), "agent_chat player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "agent_chat public_key",
    )?;
    let signer_public_key =
        normalize_public_key_field(signer_public_key_hex, "agent_chat signer public key")?;
    if signer_public_key != request_public_key {
        return Err("agent_chat public_key does not match signer public key".to_string());
    }

    let signing_key =
        signing_key_from_hex(signer_private_key_hex, "agent_chat signer private key")?;
    verify_keypair_match(
        &signing_key,
        signer_public_key.as_str(),
        "agent_chat signer public key",
    )?;

    let signing_payload = build_agent_chat_signing_payload(
        request,
        player_id.as_str(),
        request_public_key.as_str(),
        nonce,
    )?;
    sign_player_auth_proof(
        signing_key,
        player_id,
        signer_public_key,
        nonce,
        signing_payload,
    )
}

pub fn verify_agent_chat_auth_proof(
    request: &AgentChatRequest,
    proof: &PlayerAuthProof,
) -> Result<VerifiedPlayerAuth, String> {
    verify_proof_scheme(proof)?;
    let request_player_id =
        normalize_required_optional_field(request.player_id.as_deref(), "agent_chat player_id")?;
    let request_public_key = normalize_required_optional_public_key(
        request.public_key.as_deref(),
        "agent_chat public_key",
    )?;
    let proof_player_id =
        normalize_required_field(proof.player_id.as_str(), "auth proof player_id")?;
    let proof_public_key =
        normalize_public_key_field(proof.public_key.as_str(), "auth proof public key")?;
    if request_player_id != proof_player_id {
        return Err("auth proof player_id does not match request player_id".to_string());
    }
    if request_public_key != proof_public_key {
        return Err("auth proof public_key does not match request public_key".to_string());
    }
    if proof.nonce == 0 {
        return Err("auth nonce must be greater than zero".to_string());
    }
    let signing_payload = build_agent_chat_signing_payload(
        request,
        proof_player_id.as_str(),
        proof_public_key.as_str(),
        proof.nonce,
    )?;
    verify_player_auth_signature(
        proof_public_key.as_str(),
        proof.signature.as_str(),
        signing_payload.as_slice(),
    )?;
    Ok(VerifiedPlayerAuth {
        player_id: proof_player_id,
        public_key: proof_public_key,
        nonce: proof.nonce,
    })
}

fn build_prompt_control_apply_signing_payload(
    intent: PromptControlAuthIntent,
    request: &PromptControlApplyRequest,
    player_id: &str,
    public_key: &str,
    nonce: u64,
) -> Result<Vec<u8>, String> {
    let payload = PromptControlApplySigningPayload {
        operation: prompt_control_intent_operation(intent),
        agent_id: request.agent_id.as_str(),
        player_id,
        public_key,
        nonce,
        expected_version: request.expected_version,
        updated_by: request.updated_by.as_deref(),
        system_prompt_override: prompt_field_patch(&request.system_prompt_override),
        short_term_goal_override: prompt_field_patch(&request.short_term_goal_override),
        long_term_goal_override: prompt_field_patch(&request.long_term_goal_override),
    };
    encode_signing_payload(payload)
}

fn build_prompt_control_rollback_signing_payload(
    request: &PromptControlRollbackRequest,
    player_id: &str,
    public_key: &str,
    nonce: u64,
) -> Result<Vec<u8>, String> {
    let payload = PromptControlRollbackSigningPayload {
        operation: "prompt_control_rollback",
        agent_id: request.agent_id.as_str(),
        player_id,
        public_key,
        nonce,
        to_version: request.to_version,
        expected_version: request.expected_version,
        updated_by: request.updated_by.as_deref(),
    };
    encode_signing_payload(payload)
}

fn build_agent_chat_signing_payload(
    request: &AgentChatRequest,
    player_id: &str,
    public_key: &str,
    nonce: u64,
) -> Result<Vec<u8>, String> {
    let payload = AgentChatSigningPayload {
        operation: "agent_chat",
        agent_id: request.agent_id.as_str(),
        player_id,
        public_key,
        nonce,
        message: request.message.as_str(),
    };
    encode_signing_payload(payload)
}

fn encode_signing_payload<T>(payload: T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    let envelope = ViewerPlayerAuthSigningEnvelope {
        version: VIEWER_PLAYER_AUTH_PAYLOAD_VERSION,
        payload,
        actor: None,
    };
    serde_cbor::to_vec(&envelope).map_err(|err| format!("encode auth payload failed: {err}"))
}

fn prompt_control_intent_operation(intent: PromptControlAuthIntent) -> &'static str {
    match intent {
        PromptControlAuthIntent::Preview => "prompt_control_preview",
        PromptControlAuthIntent::Apply => "prompt_control_apply",
    }
}

fn prompt_field_patch(value: &Option<Option<String>>) -> PromptFieldPatch<'_> {
    match value {
        None => PromptFieldPatch {
            mode: PromptFieldMode::Unchanged,
            value: None,
        },
        Some(None) => PromptFieldPatch {
            mode: PromptFieldMode::Clear,
            value: None,
        },
        Some(Some(next)) => PromptFieldPatch {
            mode: PromptFieldMode::Set,
            value: Some(next.as_str()),
        },
    }
}

fn sign_player_auth_proof(
    signing_key: SigningKey,
    player_id: String,
    public_key: String,
    nonce: u64,
    signing_payload: Vec<u8>,
) -> Result<PlayerAuthProof, String> {
    let signature: Signature = signing_key.sign(signing_payload.as_slice());
    Ok(PlayerAuthProof {
        scheme: PlayerAuthScheme::Ed25519,
        player_id,
        public_key,
        nonce,
        signature: format!(
            "{VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX}{}",
            hex::encode(signature.to_bytes())
        ),
    })
}

fn verify_player_auth_signature(
    public_key_hex: &str,
    signature: &str,
    signing_payload: &[u8],
) -> Result<(), String> {
    let public_key_bytes = decode_hex_array::<32>(public_key_hex, "auth public key")?;
    let signature_hex = signature
        .strip_prefix(VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX)
        .ok_or_else(|| "auth signature is not awviewauth:v1".to_string())?;
    let signature_bytes = decode_hex_array::<64>(signature_hex, "auth signature")?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|err| format!("parse auth public key failed: {err}"))?;
    verifying_key
        .verify(signing_payload, &Signature::from_bytes(&signature_bytes))
        .map_err(|err| format!("verify auth signature failed: {err}"))
}

fn verify_proof_scheme(proof: &PlayerAuthProof) -> Result<(), String> {
    match proof.scheme {
        PlayerAuthScheme::Ed25519 => Ok(()),
    }
}

fn normalize_required_optional_field(raw: Option<&str>, label: &str) -> Result<String, String> {
    let Some(raw) = raw else {
        return Err(format!("{label} is required"));
    };
    normalize_required_field(raw, label)
}

fn normalize_required_optional_public_key(
    raw: Option<&str>,
    label: &str,
) -> Result<String, String> {
    let Some(raw) = raw else {
        return Err(format!("{label} is required"));
    };
    normalize_public_key_field(raw, label)
}

fn normalize_required_field(raw: &str, label: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{label} is empty"));
    }
    Ok(value.to_string())
}

fn normalize_public_key_field(raw: &str, label: &str) -> Result<String, String> {
    let normalized = normalize_required_field(raw, label)?;
    let bytes = decode_hex_array::<32>(normalized.as_str(), label)?;
    Ok(hex::encode(bytes))
}

fn signing_key_from_hex(private_key_hex: &str, label: &str) -> Result<SigningKey, String> {
    let private_key_bytes = decode_hex_array::<32>(private_key_hex, label)?;
    Ok(SigningKey::from_bytes(&private_key_bytes))
}

fn verify_keypair_match(
    signing_key: &SigningKey,
    public_key_hex: &str,
    label: &str,
) -> Result<(), String> {
    let expected_public_key = hex::encode(signing_key.verifying_key().to_bytes());
    if expected_public_key != public_key_hex {
        return Err(format!(
            "{label} does not match private key: expected={expected_public_key} actual={public_key_hex}"
        ));
    }
    Ok(())
}

fn decode_hex_array<const N: usize>(raw: &str, label: &str) -> Result<[u8; N], String> {
    let bytes = hex::decode(raw).map_err(|err| format!("decode {label} failed: {err}"))?;
    if bytes.len() != N {
        return Err(format!(
            "{label} length mismatch: expected {N} bytes, got {}",
            bytes.len()
        ));
    }
    let mut fixed = [0_u8; N];
    fixed.copy_from_slice(bytes.as_slice());
    Ok(fixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signer() -> (String, String) {
        let private_key = [7_u8; 32];
        let signing_key = SigningKey::from_bytes(&private_key);
        (
            hex::encode(signing_key.verifying_key().to_bytes()),
            hex::encode(private_key),
        )
    }

    #[test]
    fn prompt_control_apply_auth_sign_and_verify_roundtrip() {
        let (public_key, private_key) = test_signer();
        let request = PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: Some(public_key.clone()),
            auth: None,
            expected_version: Some(3),
            updated_by: Some("player-a".to_string()),
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: Some(None),
            long_term_goal_override: None,
        };
        let proof = sign_prompt_control_apply_auth_proof(
            PromptControlAuthIntent::Apply,
            &request,
            11,
            public_key.as_str(),
            private_key.as_str(),
        )
        .expect("sign proof");
        let verified = verify_prompt_control_apply_auth_proof(
            PromptControlAuthIntent::Apply,
            &request,
            &proof,
        )
        .expect("verify proof");
        assert_eq!(verified.player_id, "player-a");
        assert_eq!(verified.public_key, public_key);
        assert_eq!(verified.nonce, 11);
    }

    #[test]
    fn prompt_control_apply_auth_verify_rejects_tamper() {
        let (public_key, private_key) = test_signer();
        let request = PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: Some(public_key.clone()),
            auth: None,
            expected_version: Some(3),
            updated_by: Some("player-a".to_string()),
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        };
        let proof = sign_prompt_control_apply_auth_proof(
            PromptControlAuthIntent::Apply,
            &request,
            12,
            public_key.as_str(),
            private_key.as_str(),
        )
        .expect("sign proof");

        let mut tampered = request.clone();
        tampered.system_prompt_override = Some(Some("tampered".to_string()));
        let err = verify_prompt_control_apply_auth_proof(
            PromptControlAuthIntent::Apply,
            &tampered,
            &proof,
        )
        .expect_err("tampered payload must fail");
        assert!(err.contains("verify auth signature failed"));
    }

    #[test]
    fn agent_chat_auth_verify_rejects_player_mismatch() {
        let (public_key, private_key) = test_signer();
        let request = AgentChatRequest {
            agent_id: "agent-0".to_string(),
            message: "hello".to_string(),
            player_id: Some("player-a".to_string()),
            public_key: Some(public_key.clone()),
            auth: None,
        };
        let mut proof =
            sign_agent_chat_auth_proof(&request, 15, public_key.as_str(), private_key.as_str())
                .expect("sign proof");
        proof.player_id = "player-b".to_string();
        let err = verify_agent_chat_auth_proof(&request, &proof).expect_err("player mismatch");
        assert!(err.contains("player_id"));
    }

    #[test]
    fn agent_chat_auth_verify_rejects_invalid_signature_prefix() {
        let (public_key, private_key) = test_signer();
        let request = AgentChatRequest {
            agent_id: "agent-0".to_string(),
            message: "hello".to_string(),
            player_id: Some("player-a".to_string()),
            public_key: Some(public_key.clone()),
            auth: None,
        };
        let mut proof =
            sign_agent_chat_auth_proof(&request, 16, public_key.as_str(), private_key.as_str())
                .expect("sign proof");
        proof.signature = "badprefix:deadbeef".to_string();
        let err = verify_agent_chat_auth_proof(&request, &proof).expect_err("invalid prefix");
        assert!(err.contains("awviewauth:v1"));
    }
}
