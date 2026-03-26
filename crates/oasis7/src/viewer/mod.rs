mod auth;
#[cfg(not(target_arch = "wasm32"))]
mod demo;
#[cfg(not(target_arch = "wasm32"))]
mod live;
mod protocol;
#[cfg(not(target_arch = "wasm32"))]
mod runtime_live;
#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(not(target_arch = "wasm32"))]
mod web_bridge;

pub use auth::{
    sign_hosted_prompt_control_strong_auth_grant,
    sign_agent_chat_auth_proof, sign_gameplay_action_auth_proof,
    sign_prompt_control_apply_auth_proof, sign_prompt_control_rollback_auth_proof,
    sign_session_register_auth_proof, verify_agent_chat_auth_proof,
    verify_hosted_prompt_control_apply_strong_auth_grant,
    verify_hosted_prompt_control_rollback_strong_auth_grant,
    verify_gameplay_action_auth_proof, verify_prompt_control_apply_auth_proof,
    verify_prompt_control_rollback_auth_proof, verify_session_register_auth_proof,
    PromptControlAuthIntent, VerifiedPlayerAuth,
    VIEWER_HOSTED_STRONG_AUTH_GRANT_SIGNATURE_V1_PREFIX,
    VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX,
};
#[cfg(not(target_arch = "wasm32"))]
pub use demo::{generate_viewer_demo, ViewerDemoError, ViewerDemoSummary};
#[cfg(not(target_arch = "wasm32"))]
pub use live::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerLiveServerError,
};
pub use protocol::{
    AgentChatAck, AgentChatError, AgentChatRequest, AuthoritativeBatchFinality,
    AuthoritativeChallengeAck, AuthoritativeChallengeCommand, AuthoritativeChallengeError,
    AuthoritativeChallengeResolveRequest, AuthoritativeChallengeStatus,
    AuthoritativeChallengeSubmitRequest, AuthoritativeFinalityState,
    AuthoritativeReconnectSyncRequest, AuthoritativeRecoveryAck, AuthoritativeRecoveryCommand,
    AuthoritativeRecoveryError, AuthoritativeRecoveryStatus, AuthoritativeRollbackRequest,
    AuthoritativeSessionRegisterRequest, AuthoritativeSessionRevokeRequest,
    AuthoritativeSessionRotateRequest, ControlCompletionAck, ControlCompletionStatus,
    GameplayActionAck, GameplayActionError, GameplayActionRequest, LiveControl, PlaybackControl,
    HostedStrongAuthGrant, PlayerAuthProof, PlayerAuthScheme, PromptControlAck, PromptControlApplyRequest,
    PromptControlCommand, PromptControlError, PromptControlOperation,
    PromptControlRollbackRequest, ViewerControl, ViewerControlProfile, ViewerRequest,
    ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};
#[cfg(not(target_arch = "wasm32"))]
pub use runtime_live::{
    ViewerRuntimeLiveServer, ViewerRuntimeLiveServerConfig, ViewerRuntimeLiveServerError,
};
#[cfg(not(target_arch = "wasm32"))]
pub use server::{ViewerServer, ViewerServerConfig, ViewerServerError};
#[cfg(not(target_arch = "wasm32"))]
pub use web_bridge::{ViewerWebBridge, ViewerWebBridgeConfig, ViewerWebBridgeError};
