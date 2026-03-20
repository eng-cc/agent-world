use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeError {
    InvalidRole { role: String },
    InvalidConfig { reason: String },
    Consensus { reason: String },
    Gossip { reason: String },
    Replication { reason: String },
    Execution { reason: String },
    AlreadyRunning { node_id: String },
    NotRunning { node_id: String },
    ThreadSpawnFailed { reason: String },
    ThreadJoinFailed { node_id: String },
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::InvalidRole { role } => {
                write!(f, "invalid node role: {}", role)
            }
            NodeError::InvalidConfig { reason } => write!(f, "invalid node config: {}", reason),
            NodeError::Consensus { reason } => write!(f, "node consensus error: {}", reason),
            NodeError::Gossip { reason } => write!(f, "node gossip error: {}", reason),
            NodeError::Replication { reason } => write!(f, "node replication error: {}", reason),
            NodeError::Execution { reason } => write!(f, "node execution error: {}", reason),
            NodeError::AlreadyRunning { node_id } => {
                write!(f, "node runtime already running: {}", node_id)
            }
            NodeError::NotRunning { node_id } => write!(f, "node runtime not running: {}", node_id),
            NodeError::ThreadSpawnFailed { reason } => {
                write!(f, "failed to spawn node thread: {}", reason)
            }
            NodeError::ThreadJoinFailed { node_id } => {
                write!(f, "failed to join node thread: {}", node_id)
            }
        }
    }
}

impl std::error::Error for NodeError {}
