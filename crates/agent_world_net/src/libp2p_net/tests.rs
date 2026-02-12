use super::*;

#[test]
fn libp2p_network_generates_peer_id() {
    let network = Libp2pNetwork::new(Libp2pNetworkConfig::default());
    assert!(!network.peer_id().to_string().is_empty());
}

#[test]
fn dht_get_providers_collects_results() {
    let (sender, receiver) = oneshot::channel();
    let mut pending = PendingDhtQuery::GetProviders {
        response: Some(sender),
        providers: HashSet::new(),
        error: None,
    };
    let key_label = "providers".to_string();
    let key = RecordKey::new(&key_label);
    let mut providers = HashSet::new();
    providers.insert(PeerId::random());
    providers.insert(PeerId::random());
    let expected: HashSet<String> = providers.iter().map(|peer| peer.to_string()).collect();
    let result =
        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { key, providers }));
    handle_dht_progress(&mut pending, result, true);
    let records = futures::executor::block_on(receiver)
        .expect("oneshot")
        .expect("get providers");
    let actual: HashSet<String> = records
        .into_iter()
        .map(|record| record.provider_id)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn dht_get_world_head_decodes_record() {
    let head = WorldHeadAnnounce {
        world_id: "w1".to_string(),
        height: 9,
        block_hash: "b1".to_string(),
        state_root: "s1".to_string(),
        timestamp_ms: 42,
        signature: "sig".to_string(),
    };
    let payload = to_canonical_cbor(&head).expect("encode head");
    let key_label = "head".to_string();
    let record = kad::Record {
        key: RecordKey::new(&key_label),
        value: payload,
        publisher: None,
        expires: None,
    };
    let peer_record = kad::PeerRecord { peer: None, record };
    let (sender, receiver) = oneshot::channel();
    let mut pending = PendingDhtQuery::GetWorldHead {
        response: Some(sender),
        head: None,
        error: None,
    };
    let result = kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record)));
    handle_dht_progress(&mut pending, result, true);
    let loaded = futures::executor::block_on(receiver)
        .expect("oneshot")
        .expect("get head");
    assert_eq!(loaded, Some(head));
}

#[test]
fn dht_get_membership_directory_decodes_record() {
    let snapshot = MembershipDirectorySnapshot {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 99,
        reason: Some("sync".to_string()),
        validators: vec!["seq-1".to_string(), "seq-2".to_string()],
        quorum_threshold: 2,
        signature_key_id: None,
        signature: None,
    };
    let payload = to_canonical_cbor(&snapshot).expect("encode snapshot");
    let key_label = "membership".to_string();
    let record = kad::Record {
        key: RecordKey::new(&key_label),
        value: payload,
        publisher: None,
        expires: None,
    };
    let peer_record = kad::PeerRecord { peer: None, record };
    let (sender, receiver) = oneshot::channel();
    let mut pending = PendingDhtQuery::GetMembershipDirectory {
        response: Some(sender),
        snapshot: None,
        error: None,
    };
    let result = kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record)));
    handle_dht_progress(&mut pending, result, true);

    let loaded = futures::executor::block_on(receiver)
        .expect("oneshot")
        .expect("get membership");
    assert_eq!(loaded, Some(snapshot));
}

#[test]
fn republish_interval_gate() {
    assert!(!should_republish(100, 150, 100));
    assert!(should_republish(100, 200, 100));
    assert!(should_republish(100, 201, 100));
    assert!(!should_republish(100, 200, 0));
}
