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
    sign_agent_chat_auth_proof, sign_prompt_control_apply_auth_proof,
    sign_prompt_control_rollback_auth_proof, verify_agent_chat_auth_proof,
    verify_prompt_control_apply_auth_proof, verify_prompt_control_rollback_auth_proof,
    PromptControlAuthIntent, VerifiedPlayerAuth, VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX,
};
#[cfg(not(target_arch = "wasm32"))]
pub use demo::{generate_viewer_demo, ViewerDemoError, ViewerDemoSummary};
#[cfg(not(target_arch = "wasm32"))]
pub use live::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerLiveServerError,
};
pub use protocol::{
    AgentChatAck, AgentChatError, AgentChatRequest, ControlCompletionAck, ControlCompletionStatus,
    LiveControl, PlaybackControl, PlayerAuthProof, PlayerAuthScheme, PromptControlAck,
    PromptControlApplyRequest, PromptControlCommand, PromptControlError, PromptControlOperation,
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
