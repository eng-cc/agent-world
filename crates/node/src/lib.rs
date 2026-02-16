use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Sequencer,
    Storage,
    Observer,
}

impl NodeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeRole::Sequencer => "sequencer",
            NodeRole::Storage => "storage",
            NodeRole::Observer => "observer",
        }
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeRole {
    type Err = NodeError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "sequencer" => Ok(NodeRole::Sequencer),
            "storage" => Ok(NodeRole::Storage),
            "observer" => Ok(NodeRole::Observer),
            _ => Err(NodeError::InvalidRole {
                role: raw.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub node_id: String,
    pub world_id: String,
    pub tick_interval: Duration,
    pub role: NodeRole,
}

impl NodeConfig {
    pub fn new(
        node_id: impl Into<String>,
        world_id: impl Into<String>,
        role: NodeRole,
    ) -> Result<Self, NodeError> {
        let node_id = node_id.into();
        let world_id = world_id.into();
        if node_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "node_id cannot be empty".to_string(),
            });
        }
        if world_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "world_id cannot be empty".to_string(),
            });
        }
        Ok(Self {
            node_id,
            world_id,
            tick_interval: Duration::from_millis(200),
            role,
        })
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Result<Self, NodeError> {
        if interval.is_zero() {
            return Err(NodeError::InvalidConfig {
                reason: "tick_interval must be positive".to_string(),
            });
        }
        self.tick_interval = interval;
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub node_id: String,
    pub world_id: String,
    pub role: NodeRole,
    pub running: bool,
    pub tick_count: u64,
    pub last_tick_unix_ms: Option<i64>,
}

#[derive(Debug)]
pub struct NodeRuntime {
    config: NodeConfig,
    running: Arc<AtomicBool>,
    tick_count: Arc<AtomicU64>,
    last_tick_unix_ms: Arc<AtomicI64>,
    stop_tx: Option<mpsc::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

impl NodeRuntime {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            tick_count: Arc::new(AtomicU64::new(0)),
            last_tick_unix_ms: Arc::new(AtomicI64::new(-1)),
            stop_tx: None,
            worker: None,
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn start(&mut self) -> Result<(), NodeError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(NodeError::AlreadyRunning {
                node_id: self.config.node_id.clone(),
            });
        }

        let tick_interval = self.config.tick_interval;
        let worker_name = format!("aw-node-{}", self.config.node_id);
        let running = Arc::clone(&self.running);
        let tick_count = Arc::clone(&self.tick_count);
        let last_tick_unix_ms = Arc::clone(&self.last_tick_unix_ms);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let worker = thread::Builder::new()
            .name(worker_name)
            .spawn(move || {
                loop {
                    match stop_rx.recv_timeout(tick_interval) {
                        Ok(()) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            tick_count.fetch_add(1, Ordering::Relaxed);
                            last_tick_unix_ms.store(now_unix_ms(), Ordering::Relaxed);
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                running.store(false, Ordering::SeqCst);
            })
            .map_err(|err| NodeError::ThreadSpawnFailed {
                reason: err.to_string(),
            })?;

        self.stop_tx = Some(stop_tx);
        self.worker = Some(worker);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), NodeError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(NodeError::NotRunning {
                node_id: self.config.node_id.clone(),
            });
        }
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| NodeError::ThreadJoinFailed {
                node_id: self.config.node_id.clone(),
            })?;
        }
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn snapshot(&self) -> NodeSnapshot {
        let last_tick = self.last_tick_unix_ms.load(Ordering::Relaxed);
        NodeSnapshot {
            node_id: self.config.node_id.clone(),
            world_id: self.config.world_id.clone(),
            role: self.config.role,
            running: self.running.load(Ordering::SeqCst),
            tick_count: self.tick_count.load(Ordering::Relaxed),
            last_tick_unix_ms: (last_tick >= 0).then_some(last_tick),
        }
    }
}

impl Drop for NodeRuntime {
    fn drop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        self.running.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeError {
    InvalidRole { role: String },
    InvalidConfig { reason: String },
    AlreadyRunning { node_id: String },
    NotRunning { node_id: String },
    ThreadSpawnFailed { reason: String },
    ThreadJoinFailed { node_id: String },
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_parse_roundtrip() {
        for role in [NodeRole::Sequencer, NodeRole::Storage, NodeRole::Observer] {
            let parsed = NodeRole::from_str(role.as_str()).expect("parse role");
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn runtime_start_and_stop_updates_snapshot() {
        let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
            .expect("config")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick interval");
        let mut runtime = NodeRuntime::new(config);
        runtime.start().expect("start");
        thread::sleep(Duration::from_millis(35));
        let running = runtime.snapshot();
        assert!(running.running);
        assert!(running.tick_count >= 2);
        assert!(running.last_tick_unix_ms.is_some());

        runtime.stop().expect("stop");
        let stopped = runtime.snapshot();
        assert!(!stopped.running);
        assert!(stopped.tick_count >= running.tick_count);
    }

    #[test]
    fn runtime_rejects_double_start() {
        let config = NodeConfig::new("node-b", "world-b", NodeRole::Sequencer).expect("config");
        let mut runtime = NodeRuntime::new(config);
        runtime.start().expect("first start");
        let err = runtime.start().expect_err("second start must fail");
        assert!(matches!(err, NodeError::AlreadyRunning { .. }));
        runtime.stop().expect("stop");
    }
}
