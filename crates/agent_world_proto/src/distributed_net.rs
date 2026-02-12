//! Distributed network adapter abstractions (libp2p-ready).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
}

impl NetworkSubscription {
    pub fn new(topic: String, inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>) -> Self {
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
