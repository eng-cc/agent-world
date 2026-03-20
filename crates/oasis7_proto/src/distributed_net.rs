//! Distributed network adapter abstractions (libp2p-ready).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const DEFAULT_SUBSCRIPTION_INBOX_MAX_MESSAGES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub topic: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub protocol: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub payload: Vec<u8>,
}

pub trait DistributedNetwork<E> {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), E>;
    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, E>;
    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, E>;
    fn request_with_providers(
        &self,
        protocol: &str,
        payload: &[u8],
        _providers: &[String],
    ) -> Result<Vec<u8>, E> {
        self.request(protocol, payload)
    }
    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, E> + Send + Sync>,
    ) -> Result<(), E>;
}

#[derive(Debug, Clone)]
pub struct NetworkSubscription {
    topic: String,
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    max_inbox_messages: usize,
}

impl NetworkSubscription {
    pub fn new(topic: String, inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>) -> Self {
        Self::with_max_inbox_messages(topic, inbox, DEFAULT_SUBSCRIPTION_INBOX_MAX_MESSAGES)
    }

    pub fn with_max_inbox_messages(
        topic: String,
        inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
        max_inbox_messages: usize,
    ) -> Self {
        Self {
            topic,
            inbox,
            max_inbox_messages: max_inbox_messages.max(1),
        }
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn max_inbox_messages(&self) -> usize {
        self.max_inbox_messages
    }

    pub fn drain(&self) -> Vec<Vec<u8>> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.remove(&self.topic).unwrap_or_default()
    }
}

pub fn push_bounded_inbox_message(
    inbox: &Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    topic: &str,
    payload: Vec<u8>,
    max_inbox_messages: usize,
) {
    let max_inbox_messages = max_inbox_messages.max(1);
    let mut inbox = inbox.lock().expect("lock inbox");
    let entries = inbox.entry(topic.to_string()).or_default();
    entries.push(payload);
    let overflow = entries.len().saturating_sub(max_inbox_messages);
    if overflow > 0 {
        entries.drain(0..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_bounded_inbox_message_evicts_oldest_messages() {
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        push_bounded_inbox_message(&inbox, "topic-a", b"m1".to_vec(), 2);
        push_bounded_inbox_message(&inbox, "topic-a", b"m2".to_vec(), 2);
        push_bounded_inbox_message(&inbox, "topic-a", b"m3".to_vec(), 2);

        let mut guard = inbox.lock().expect("lock inbox");
        let queued = guard.remove("topic-a").expect("topic queue");
        assert_eq!(queued, vec![b"m2".to_vec(), b"m3".to_vec()]);
    }

    #[test]
    fn network_subscription_new_uses_default_bounded_limit() {
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        let subscription = NetworkSubscription::new("topic-a".to_string(), Arc::clone(&inbox));
        assert_eq!(
            subscription.max_inbox_messages(),
            DEFAULT_SUBSCRIPTION_INBOX_MAX_MESSAGES
        );
    }
}
