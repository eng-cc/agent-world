#[derive(Debug, Clone)]
pub struct ExecutionWriteConfig {
    pub segment: super::segmenter::SegmentConfig,
    pub codec: String,
}

impl Default for ExecutionWriteConfig {
    fn default() -> Self {
        Self {
            segment: super::segmenter::SegmentConfig::default(),
            codec: super::distributed::WIRE_ENCODING_CBOR.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionWriteResult {
    pub block: super::distributed::WorldBlock,
    pub block_hash: String,
    pub block_ref: super::distributed::BlobRef,
    pub block_announce: super::distributed::BlockAnnounce,
    pub head_announce: super::distributed::WorldHeadAnnounce,
    pub snapshot_manifest: super::distributed::SnapshotManifest,
    pub snapshot_manifest_ref: super::distributed::BlobRef,
    pub journal_segments: Vec<super::segmenter::JournalSegmentRef>,
    pub journal_segments_ref: super::distributed::BlobRef,
}

include!("../../../agent_world_net/src/execution_storage.rs");
