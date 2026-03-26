use serde::Serialize;

pub(super) const DEFAULT_DEPLOYMENT_MODE: &str = "trusted_local_only";
#[allow(dead_code)]
pub(super) const HOSTED_PLAYER_ACCESS_VERDICT: &str = "specified_not_implemented";
#[allow(dead_code)]
const DEFAULT_MAX_GUEST_SESSIONS: u64 = 32;
#[allow(dead_code)]
const DEFAULT_MAX_PLAYER_SESSIONS: u64 = 8;
#[allow(dead_code)]
const DEFAULT_ISSUE_RATE_LIMIT_PER_MINUTE: u64 = 60;
#[allow(dead_code)]
const DEFAULT_WORLD_FULL_POLICY: &str = "reject";
#[allow(dead_code)]
const DEFAULT_KICK_POLICY: &str = "operator_audit_required";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeploymentMode {
    TrustedLocalOnly,
    HostedPublicJoin,
}

impl DeploymentMode {
    pub(super) fn parse(raw: &str, label: &str) -> Result<Self, String> {
        match raw.trim() {
            "trusted_local_only" => Ok(Self::TrustedLocalOnly),
            "hosted_public_join" => Ok(Self::HostedPublicJoin),
            _ => Err(format!(
                "{label} must be one of: trusted_local_only|hosted_public_join"
            )),
        }
    }

    #[allow(dead_code)]
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::TrustedLocalOnly => "trusted_local_only",
            Self::HostedPublicJoin => "hosted_public_join",
        }
    }

    #[allow(dead_code)]
    pub(super) fn browser_signer_bootstrap_mode(self) -> &'static str {
        match self {
            Self::TrustedLocalOnly => "trusted_local_bootstrap_allowed",
            Self::HostedPublicJoin => "disabled_for_public_player_plane",
        }
    }

    #[allow(dead_code)]
    pub(super) fn gui_agent_action_surface(self) -> &'static str {
        match self {
            Self::TrustedLocalOnly => "legacy_shared_local_preview",
            Self::HostedPublicJoin => "legacy_private_control_plane_only",
        }
    }

    #[allow(dead_code)]
    pub(super) fn requires_loopback_private_control(self) -> bool {
        matches!(self, Self::HostedPublicJoin)
    }

    #[allow(dead_code)]
    pub(super) fn disables_browser_signer_bootstrap(self) -> bool {
        matches!(self, Self::HostedPublicJoin)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(super) struct HostedAdmissionControlContract {
    pub(super) max_guest_sessions: u64,
    pub(super) max_player_sessions: u64,
    pub(super) issue_rate_limit_per_minute: u64,
    pub(super) world_full_policy: String,
    pub(super) kick_policy: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(super) struct HostedPlayerAccessContract {
    pub(super) deployment_mode: String,
    pub(super) verdict: String,
    pub(super) browser_signer_bootstrap: String,
    pub(super) gui_agent_action_surface: String,
    pub(super) public_state_route: String,
    pub(super) public_endpoints: Vec<String>,
    pub(super) private_endpoints: Vec<String>,
    pub(super) session_ladder: Vec<String>,
    pub(super) action_matrix: Vec<HostedActionAccessPolicy>,
    pub(super) admission: HostedAdmissionControlContract,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(super) struct HostedActionAccessPolicy {
    pub(super) action_id: String,
    pub(super) required_auth: String,
    pub(super) availability: String,
    pub(super) reason: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(super) struct HostedViewerAccessHint {
    pub(super) deployment_mode: String,
    pub(super) verdict: String,
    pub(super) browser_signer_bootstrap: String,
    pub(super) session_ladder: Vec<String>,
    pub(super) action_matrix: Vec<HostedActionAccessPolicy>,
}

#[allow(dead_code)]
pub(super) fn hosted_player_access_contract(mode: DeploymentMode) -> HostedPlayerAccessContract {
    HostedPlayerAccessContract {
        deployment_mode: mode.as_str().to_string(),
        verdict: HOSTED_PLAYER_ACCESS_VERDICT.to_string(),
        browser_signer_bootstrap: mode.browser_signer_bootstrap_mode().to_string(),
        gui_agent_action_surface: mode.gui_agent_action_surface().to_string(),
        public_state_route: "/api/public/state".to_string(),
        public_endpoints: web_launcher_public_endpoints()
            .into_iter()
            .map(|value| (*value).to_string())
            .collect(),
        private_endpoints: web_launcher_private_endpoints()
            .into_iter()
            .map(|value| (*value).to_string())
            .collect(),
        session_ladder: vec![
            "guest_session".to_string(),
            "player_session".to_string(),
            "strong_auth".to_string(),
        ],
        action_matrix: hosted_action_matrix(mode),
        admission: HostedAdmissionControlContract {
            max_guest_sessions: DEFAULT_MAX_GUEST_SESSIONS,
            max_player_sessions: DEFAULT_MAX_PLAYER_SESSIONS,
            issue_rate_limit_per_minute: DEFAULT_ISSUE_RATE_LIMIT_PER_MINUTE,
            world_full_policy: DEFAULT_WORLD_FULL_POLICY.to_string(),
            kick_policy: DEFAULT_KICK_POLICY.to_string(),
        },
    }
}

#[allow(dead_code)]
pub(super) fn hosted_viewer_access_hint(mode: DeploymentMode) -> HostedViewerAccessHint {
    HostedViewerAccessHint {
        deployment_mode: mode.as_str().to_string(),
        verdict: HOSTED_PLAYER_ACCESS_VERDICT.to_string(),
        browser_signer_bootstrap: mode.browser_signer_bootstrap_mode().to_string(),
        session_ladder: vec![
            "guest_session".to_string(),
            "player_session".to_string(),
            "strong_auth".to_string(),
        ],
        action_matrix: hosted_action_matrix(mode),
    }
}

#[allow(dead_code)]
fn hosted_action_matrix(mode: DeploymentMode) -> Vec<HostedActionAccessPolicy> {
    let strong_auth_availability = match mode {
        DeploymentMode::TrustedLocalOnly => "trusted_local_preview_only",
        DeploymentMode::HostedPublicJoin => "blocked_until_strong_auth",
    };
    let strong_auth_reason = match mode {
        DeploymentMode::TrustedLocalOnly => {
            "trusted local preview may still use preview bootstrap; hosted/public strong-auth lane remains pending"
        }
        DeploymentMode::HostedPublicJoin => {
            "hosted public join keeps this action behind strong_auth/private plane until the dedicated proof lane lands"
        }
    };
    vec![
        HostedActionAccessPolicy {
            action_id: "gameplay_action".to_string(),
            required_auth: "player_session".to_string(),
            availability: "public_player_plane".to_string(),
            reason: "core gameplay input stays on the player_session lane".to_string(),
        },
        HostedActionAccessPolicy {
            action_id: "agent_chat".to_string(),
            required_auth: "player_session".to_string(),
            availability: "public_player_plane".to_string(),
            reason: "agent chat currently stays on the low-risk player_session lane".to_string(),
        },
        HostedActionAccessPolicy {
            action_id: "prompt_control_preview".to_string(),
            required_auth: "strong_auth".to_string(),
            availability: strong_auth_availability.to_string(),
            reason: strong_auth_reason.to_string(),
        },
        HostedActionAccessPolicy {
            action_id: "prompt_control_apply".to_string(),
            required_auth: "strong_auth".to_string(),
            availability: strong_auth_availability.to_string(),
            reason: strong_auth_reason.to_string(),
        },
        HostedActionAccessPolicy {
            action_id: "prompt_control_rollback".to_string(),
            required_auth: "strong_auth".to_string(),
            availability: strong_auth_availability.to_string(),
            reason: strong_auth_reason.to_string(),
        },
        HostedActionAccessPolicy {
            action_id: "main_token_transfer".to_string(),
            required_auth: "strong_auth".to_string(),
            availability: strong_auth_availability.to_string(),
            reason: strong_auth_reason.to_string(),
        },
    ]
}

#[allow(dead_code)]
pub(super) fn web_launcher_public_endpoints() -> &'static [&'static str] {
    &[
        "/healthz",
        "/api/public/player-session/admission",
        "/api/public/player-session/refresh",
        "/api/public/state",
        "/api/public/player-session/issue",
        "/api/public/player-session/release",
        "/api/chain/transfer",
        "/api/chain/transfer/accounts",
        "/api/chain/transfer/status",
        "/api/chain/transfer/history",
        "/api/chain/explorer/overview",
        "/api/chain/explorer/transactions",
        "/api/chain/explorer/transaction",
        "/api/chain/explorer/blocks",
        "/api/chain/explorer/block",
        "/api/chain/explorer/txs",
        "/api/chain/explorer/tx",
        "/api/chain/explorer/search",
        "/api/chain/explorer/address",
        "/api/chain/explorer/contracts",
        "/api/chain/explorer/contract",
        "/api/chain/explorer/assets",
        "/api/chain/explorer/mempool",
        "/api/chain/feedback",
    ]
}

#[allow(dead_code)]
pub(super) fn web_launcher_private_endpoints() -> &'static [&'static str] {
    &[
        "/",
        "/api/state",
        "/api/gui-agent/capabilities",
        "/api/gui-agent/state",
        "/api/gui-agent/action",
        "/api/ui/schema",
        "/api/start",
        "/api/stop",
        "/api/chain/start",
        "/api/chain/stop",
    ]
}
