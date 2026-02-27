mod auth;
mod demo;
mod live;
mod protocol;
mod server;
mod web_bridge;

pub use auth::{
    sign_agent_chat_auth_proof, sign_prompt_control_apply_auth_proof,
    sign_prompt_control_rollback_auth_proof, verify_agent_chat_auth_proof,
    verify_prompt_control_apply_auth_proof, verify_prompt_control_rollback_auth_proof,
    PromptControlAuthIntent, VerifiedPlayerAuth, VIEWER_PLAYER_AUTH_SIGNATURE_V1_PREFIX,
};
pub use demo::{generate_viewer_demo, ViewerDemoError, ViewerDemoSummary};
pub use live::{
    ViewerLiveDecisionMode, ViewerLiveScriptPacingMode, ViewerLiveServer, ViewerLiveServerConfig,
    ViewerLiveServerError,
};
pub use protocol::{
    AgentChatAck, AgentChatError, AgentChatRequest, PlayerAuthProof, PlayerAuthScheme,
    PromptControlAck, PromptControlApplyRequest, PromptControlCommand, PromptControlError,
    PromptControlOperation, PromptControlRollbackRequest, ViewerControl, ViewerRequest,
    ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};
pub use server::{ViewerServer, ViewerServerConfig, ViewerServerError};
pub use web_bridge::{ViewerWebBridge, ViewerWebBridgeConfig, ViewerWebBridgeError};
