//! Distributed network adapter abstractions (libp2p-ready).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::error::WorldError;

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

pub trait DistributedNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError>;
    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError>;
    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError>;
    fn request_with_providers(
        &self,
        protocol: &str,
        payload: &[u8],
        _providers: &[String],
    ) -> Result<Vec<u8>, WorldError> {
        self.request(protocol, payload)
    }
    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError>;
}

#[derive(Debug, Clone)]
pub struct NetworkSubscription {
    topic: String,
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
}

impl NetworkSubscription {
    pub(crate) fn new(topic: String, inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>) -> Self {
        Self { topic, inbox }
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn drain(&self) -> Vec<Vec<u8>> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.remove(&self.topic).unwrap_or_default()
    }
}

#[derive(Clone, Default)]
pub struct InMemoryNetwork {
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    published: Arc<Mutex<Vec<NetworkMessage>>>,
    handlers: Arc<Mutex<HashMap<String, Handler>>>,
}

type Handler = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>;

impl InMemoryNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn published(&self) -> Vec<NetworkMessage> {
        self.published.lock().expect("lock published").clone()
    }
}

impl DistributedNetwork for InMemoryNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        let message = NetworkMessage {
            topic: topic.to_string(),
            payload: payload.to_vec(),
        };
        {
            let mut published = self.published.lock().expect("lock published");
            published.push(message.clone());
        }
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox
            .entry(topic.to_string())
            .or_default()
            .push(message.payload);
        Ok(())
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.entry(topic.to_string()).or_default();
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let handler = {
            let handlers = self.handlers.lock().expect("lock handlers");
            handlers.get(protocol).cloned()
        };
        let handler = handler.ok_or_else(|| WorldError::NetworkProtocolUnavailable {
            protocol: protocol.to_string(),
        })?;
        handler(payload)
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        let mut handlers = self.handlers.lock().expect("lock handlers");
        handlers.insert(protocol.to_string(), Arc::from(handler));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_publish_delivers_to_subscribers() {
        let network = InMemoryNetwork::new();
        let subscription = network.subscribe("aw.w1.action").expect("subscribe");

        network
            .publish("aw.w1.action", b"payload")
            .expect("publish");

        let messages = subscription.drain();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"payload".to_vec());
    }

    #[test]
    fn in_memory_request_invokes_handler() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(
                "/aw/rr/1.0.0/get_world_head",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-ok");
                    Ok(out)
                }),
            )
            .expect("register handler");

        let response = network
            .request("/aw/rr/1.0.0/get_world_head", b"ping")
            .expect("request");
        assert_eq!(response, b"ping-ok".to_vec());
    }
}
