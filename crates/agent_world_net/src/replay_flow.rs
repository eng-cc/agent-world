use agent_world_proto::distributed::SnapshotManifest;
use agent_world_proto::distributed_storage::JournalSegmentRef;

pub fn load_manifest_and_segments<E>(
    snapshot_ref: &str,
    journal_ref: &str,
    mut fetch_blob: impl FnMut(&str) -> Result<Vec<u8>, E>,
    mut verify_blob: impl FnMut(&str, &[u8]) -> Result<(), E>,
    mut store_blob: impl FnMut(&str, &[u8]) -> Result<(), E>,
    map_decode_error: impl Fn(serde_cbor::Error) -> E + Copy,
) -> Result<(SnapshotManifest, Vec<JournalSegmentRef>), E> {
    let manifest_bytes = fetch_blob(snapshot_ref)?;
    verify_blob(snapshot_ref, &manifest_bytes)?;
    let manifest: SnapshotManifest =
        serde_cbor::from_slice(&manifest_bytes).map_err(map_decode_error)?;

    let segments_bytes = fetch_blob(journal_ref)?;
    verify_blob(journal_ref, &segments_bytes)?;
    let segments: Vec<JournalSegmentRef> =
        serde_cbor::from_slice(&segments_bytes).map_err(map_decode_error)?;

    for chunk in &manifest.chunks {
        let bytes = fetch_blob(&chunk.content_hash)?;
        store_blob(&chunk.content_hash, &bytes)?;
    }
    for segment in &segments {
        let bytes = fetch_blob(&segment.content_hash)?;
        store_blob(&segment.content_hash, &bytes)?;
    }
    Ok((manifest, segments))
}
