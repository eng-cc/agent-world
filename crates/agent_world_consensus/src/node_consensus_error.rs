#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConsensusError {
    pub reason: String,
}

impl std::fmt::Display for NodeConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.as_str())
    }
}

impl std::error::Error for NodeConsensusError {}
