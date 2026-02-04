use crate::geometry::{great_circle_distance_cm, GeoPos};
use crate::models::RobotBodySpec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io;
use std::path::Path;

// ============================================================================
// Agent Interface (observe → decide → act)
// ============================================================================

/// Core trait for Agent behavior in the world.
///
/// An Agent follows the observe → decide → act loop:
/// 1. `observe`: Receive the current observation of the world
/// 2. `decide`: Based on observation, decide what action to take
/// 3. `act`: The kernel executes the decided action and produces events
///
/// This trait is designed to be implemented by various agent types:
/// - Simple rule-based agents
/// - LLM-powered agents
/// - Scripted agents for testing
pub trait AgentBehavior {
    /// Returns the agent's unique identifier.
    fn agent_id(&self) -> &str;

    /// Called when the agent receives a new observation.
    /// Returns an optional action to take, or None if the agent chooses to wait.
    fn decide(&mut self, observation: &Observation) -> AgentDecision;

    /// Called after an action is executed to notify the agent of the result.
    /// This allows the agent to update internal state based on action outcomes.
    fn on_action_result(&mut self, _result: &ActionResult) {
        // Default: no-op
    }

    /// Called when an event affecting this agent occurs.
    /// This allows the agent to react to external events.
    fn on_event(&mut self, _event: &WorldEvent) {
        // Default: no-op
    }
}

/// The result of an agent's decision process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentDecision {
    /// The agent decides to perform an action.
    Act(Action),
    /// The agent decides to wait/skip this turn.
    Wait,
    /// The agent decides to wait for a specific number of ticks.
    WaitTicks(u64),
}

impl AgentDecision {
    /// Returns true if the agent decided to act.
    pub fn is_act(&self) -> bool {
        matches!(self, AgentDecision::Act(_))
    }

    /// Returns the action if the agent decided to act.
    pub fn action(&self) -> Option<&Action> {
        match self {
            AgentDecision::Act(action) => Some(action),
            _ => None,
        }
    }
}

/// Result of an action execution, providing feedback to the agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    /// The action that was executed.
    pub action: Action,
    /// The action ID assigned by the kernel.
    pub action_id: ActionId,
    /// Whether the action succeeded.
    pub success: bool,
    /// The resulting event (success or rejection).
    pub event: WorldEvent,
}

impl ActionResult {
    /// Returns true if the action was rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self.event.kind, WorldEventKind::ActionRejected { .. })
    }

    /// Returns the rejection reason if the action was rejected.
    pub fn reject_reason(&self) -> Option<&RejectReason> {
        match &self.event.kind {
            WorldEventKind::ActionRejected { reason } => Some(reason),
            _ => None,
        }
    }
}

/// An agent registration with behavior and state tracking.
#[derive(Debug)]
pub struct RegisteredAgent<B: AgentBehavior> {
    /// The agent behavior implementation.
    pub behavior: B,
    /// Number of ticks to wait before next decision.
    pub wait_until: Option<WorldTime>,
    /// Total actions taken by this agent.
    pub action_count: u64,
    /// Total decisions made by this agent.
    pub decision_count: u64,
    /// Per-agent quota (overrides runner-level quota).
    pub quota: Option<AgentQuota>,
    /// Rate limiting state.
    pub rate_limit_state: RateLimitState,
}

impl<B: AgentBehavior> RegisteredAgent<B> {
    /// Create a new registered agent.
    pub fn new(behavior: B) -> Self {
        Self {
            behavior,
            wait_until: None,
            action_count: 0,
            decision_count: 0,
            quota: None,
            rate_limit_state: RateLimitState::default(),
        }
    }

    /// Create a new registered agent with quota.
    pub fn with_quota(behavior: B, quota: AgentQuota) -> Self {
        Self {
            behavior,
            wait_until: None,
            action_count: 0,
            decision_count: 0,
            quota: Some(quota),
            rate_limit_state: RateLimitState::default(),
        }
    }

    /// Returns true if the agent is ready to act at the given time.
    pub fn is_ready(&self, now: WorldTime) -> bool {
        match self.wait_until {
            Some(until) => now >= until,
            None => true,
        }
    }

    /// Check if the agent has exhausted its quota.
    pub fn is_quota_exhausted(&self) -> bool {
        if let Some(quota) = &self.quota {
            quota.is_exhausted(self.action_count, self.decision_count)
        } else {
            false
        }
    }

    /// Check if the agent is rate limited at the given time.
    pub fn is_rate_limited(&self, now: WorldTime, policy: Option<&RateLimitPolicy>) -> bool {
        if let Some(policy) = policy {
            self.rate_limit_state.is_limited(now, policy)
        } else {
            false
        }
    }

    /// Record an action for rate limiting purposes.
    pub fn record_action(&mut self, now: WorldTime, policy: Option<&RateLimitPolicy>) {
        if let Some(policy) = policy {
            self.rate_limit_state.record_action(now, policy);
        }
    }
}

// ============================================================================
// Agent Memory System
// ============================================================================

/// Short-term memory entry: a recent observation with context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// The world time when this memory was created.
    pub time: WorldTime,
    /// The type of memory entry.
    pub kind: MemoryEntryKind,
    /// Optional importance score (0.0 to 1.0).
    pub importance: f64,
}

/// Types of memory entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MemoryEntryKind {
    /// An observation that was received.
    Observation { summary: String },
    /// A decision that was made.
    Decision { decision: AgentDecision },
    /// An action result.
    ActionResult { action: Action, success: bool },
    /// An external event that affected the agent.
    Event { description: String },
    /// A custom note or reflection.
    Note { content: String },
}

impl MemoryEntry {
    /// Create a new memory entry for an observation.
    pub fn observation(time: WorldTime, summary: impl Into<String>) -> Self {
        Self {
            time,
            kind: MemoryEntryKind::Observation {
                summary: summary.into(),
            },
            importance: 0.5,
        }
    }

    /// Create a new memory entry for a decision.
    pub fn decision(time: WorldTime, decision: AgentDecision) -> Self {
        Self {
            time,
            kind: MemoryEntryKind::Decision { decision },
            importance: 0.6,
        }
    }

    /// Create a new memory entry for an action result.
    pub fn action_result(time: WorldTime, action: Action, success: bool) -> Self {
        let importance = if success { 0.5 } else { 0.8 }; // Failed actions are more memorable
        Self {
            time,
            kind: MemoryEntryKind::ActionResult { action, success },
            importance,
        }
    }

    /// Create a new memory entry for an event.
    pub fn event(time: WorldTime, description: impl Into<String>) -> Self {
        Self {
            time,
            kind: MemoryEntryKind::Event {
                description: description.into(),
            },
            importance: 0.7,
        }
    }

    /// Create a new memory entry for a custom note.
    pub fn note(time: WorldTime, content: impl Into<String>) -> Self {
        Self {
            time,
            kind: MemoryEntryKind::Note {
                content: content.into(),
            },
            importance: 0.5,
        }
    }

    /// Set the importance score.
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }
}

/// Short-term memory buffer with configurable capacity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortTermMemory {
    /// The memory entries in chronological order.
    entries: VecDeque<MemoryEntry>,
    /// Maximum number of entries to keep.
    capacity: usize,
    /// Total entries ever added (for statistics).
    total_added: u64,
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl ShortTermMemory {
    /// Create a new short-term memory with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            total_added: 0,
        }
    }

    /// Add a new memory entry.
    pub fn add(&mut self, entry: MemoryEntry) {
        self.total_added += 1;
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Get the most recent N entries.
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.iter().rev().take(n)
    }

    /// Get all entries.
    pub fn all(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.iter()
    }

    /// Get entries since a given time.
    pub fn since(&self, time: WorldTime) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.iter().filter(move |e| e.time >= time)
    }

    /// Get entries with importance above a threshold.
    pub fn important(&self, threshold: f64) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.iter().filter(move |e| e.importance >= threshold)
    }

    /// Get the number of entries in memory.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the memory is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the capacity of the memory.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the total number of entries ever added.
    pub fn total_added(&self) -> u64 {
        self.total_added
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Create a summary of recent memory (for context injection).
    pub fn summarize(&self, max_entries: usize) -> String {
        let recent: Vec<_> = self.recent(max_entries).collect();
        if recent.is_empty() {
            return "No recent memories.".to_string();
        }

        let mut lines = Vec::new();
        for entry in recent.iter().rev() {
            let line = match &entry.kind {
                MemoryEntryKind::Observation { summary } => {
                    format!("[T{}] Observed: {}", entry.time, summary)
                }
                MemoryEntryKind::Decision { decision } => {
                    format!("[T{}] Decided: {:?}", entry.time, decision)
                }
                MemoryEntryKind::ActionResult { action, success } => {
                    let status = if *success { "succeeded" } else { "failed" };
                    format!("[T{}] Action {:?} {}", entry.time, action, status)
                }
                MemoryEntryKind::Event { description } => {
                    format!("[T{}] Event: {}", entry.time, description)
                }
                MemoryEntryKind::Note { content } => {
                    format!("[T{}] Note: {}", entry.time, content)
                }
            };
            lines.push(line);
        }
        lines.join("\n")
    }
}

/// Long-term memory entry: a summarized or indexed piece of knowledge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LongTermMemoryEntry {
    /// Unique identifier for this entry.
    pub id: String,
    /// The world time when this was stored.
    pub created_at: WorldTime,
    /// Last accessed time.
    pub last_accessed: WorldTime,
    /// Access count (for LRU-like eviction).
    pub access_count: u64,
    /// The content of the memory.
    pub content: String,
    /// Tags for categorization and retrieval.
    pub tags: Vec<String>,
    /// Importance score.
    pub importance: f64,
}

impl LongTermMemoryEntry {
    /// Create a new long-term memory entry.
    pub fn new(id: impl Into<String>, created_at: WorldTime, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            created_at,
            last_accessed: created_at,
            access_count: 0,
            content: content.into(),
            tags: Vec::new(),
            importance: 0.5,
        }
    }

    /// Add a tag to this entry.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the importance score.
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Mark this entry as accessed.
    pub fn mark_accessed(&mut self, time: WorldTime) {
        self.last_accessed = time;
        self.access_count += 1;
    }
}

/// Long-term memory store with basic retrieval.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LongTermMemory {
    /// All stored entries, indexed by ID.
    entries: BTreeMap<String, LongTermMemoryEntry>,
    /// Maximum number of entries to keep.
    max_entries: Option<usize>,
    /// Counter for generating unique IDs.
    next_id: u64,
}

impl LongTermMemory {
    /// Create a new long-term memory store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new long-term memory store with a capacity limit.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries: Some(max_entries),
            next_id: 0,
        }
    }

    /// Store a new entry and return its ID.
    pub fn store(&mut self, content: impl Into<String>, time: WorldTime) -> String {
        let id = format!("mem-{}", self.next_id);
        self.next_id += 1;

        let entry = LongTermMemoryEntry::new(&id, time, content);
        self.entries.insert(id.clone(), entry);

        // Evict if over capacity (remove least important)
        if let Some(max) = self.max_entries {
            while self.entries.len() > max {
                if let Some(to_remove) = self
                    .entries
                    .iter()
                    .min_by(|a, b| {
                        a.1.importance
                            .partial_cmp(&b.1.importance)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(k, _)| k.clone())
                {
                    self.entries.remove(&to_remove);
                } else {
                    break;
                }
            }
        }

        id
    }

    /// Store an entry with tags.
    pub fn store_with_tags(
        &mut self,
        content: impl Into<String>,
        time: WorldTime,
        tags: Vec<String>,
    ) -> String {
        let id = self.store(content, time);
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.tags = tags;
        }
        id
    }

    /// Retrieve an entry by ID.
    pub fn get(&mut self, id: &str, time: WorldTime) -> Option<&LongTermMemoryEntry> {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.mark_accessed(time);
        }
        self.entries.get(id)
    }

    /// Search entries by tag.
    pub fn search_by_tag(&self, tag: &str) -> Vec<&LongTermMemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Search entries by content (simple substring match).
    pub fn search_by_content(&self, query: &str) -> Vec<&LongTermMemoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .values()
            .filter(|e| e.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get the most important entries.
    pub fn top_by_importance(&self, n: usize) -> Vec<&LongTermMemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.into_iter().take(n).collect()
    }

    /// Get the most recently accessed entries.
    pub fn recently_accessed(&self, n: usize) -> Vec<&LongTermMemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        entries.into_iter().take(n).collect()
    }

    /// Get all entries.
    pub fn all(&self) -> impl Iterator<Item = &LongTermMemoryEntry> {
        self.entries.values()
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the memory is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Remove an entry by ID.
    pub fn remove(&mut self, id: &str) -> Option<LongTermMemoryEntry> {
        self.entries.remove(id)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Combined memory system for an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentMemory {
    /// Short-term memory buffer.
    pub short_term: ShortTermMemory,
    /// Long-term memory store.
    pub long_term: LongTermMemory,
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self {
            short_term: ShortTermMemory::default(),
            long_term: LongTermMemory::new(),
        }
    }
}

impl AgentMemory {
    /// Create a new agent memory with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new agent memory with custom capacities.
    pub fn with_capacities(short_term_capacity: usize, long_term_capacity: usize) -> Self {
        Self {
            short_term: ShortTermMemory::new(short_term_capacity),
            long_term: LongTermMemory::with_capacity(long_term_capacity),
        }
    }

    /// Record an observation.
    pub fn record_observation(&mut self, time: WorldTime, summary: impl Into<String>) {
        self.short_term.add(MemoryEntry::observation(time, summary));
    }

    /// Record a decision.
    pub fn record_decision(&mut self, time: WorldTime, decision: AgentDecision) {
        self.short_term.add(MemoryEntry::decision(time, decision));
    }

    /// Record an action result.
    pub fn record_action_result(&mut self, time: WorldTime, action: Action, success: bool) {
        self.short_term
            .add(MemoryEntry::action_result(time, action, success));
    }

    /// Record an event.
    pub fn record_event(&mut self, time: WorldTime, description: impl Into<String>) {
        self.short_term.add(MemoryEntry::event(time, description));
    }

    /// Record a note.
    pub fn record_note(&mut self, time: WorldTime, content: impl Into<String>) {
        self.short_term.add(MemoryEntry::note(time, content));
    }

    /// Consolidate recent short-term memories into long-term storage.
    ///
    /// This is a simple consolidation that stores important recent memories.
    /// More sophisticated implementations could use summarization or embedding.
    pub fn consolidate(&mut self, time: WorldTime, importance_threshold: f64) {
        let important: Vec<_> = self
            .short_term
            .important(importance_threshold)
            .cloned()
            .collect();

        for entry in important {
            let content = match &entry.kind {
                MemoryEntryKind::Observation { summary } => summary.clone(),
                MemoryEntryKind::Decision { decision } => format!("Decision: {:?}", decision),
                MemoryEntryKind::ActionResult { action, success } => {
                    let status = if *success { "succeeded" } else { "failed" };
                    format!("Action {:?} {}", action, status)
                }
                MemoryEntryKind::Event { description } => description.clone(),
                MemoryEntryKind::Note { content } => content.clone(),
            };

            let id = self.long_term.store(&content, time);
            if let Some(stored) = self.long_term.entries.get_mut(&id) {
                stored.importance = entry.importance;
            }
        }
    }

    /// Get a summary of recent context for decision-making.
    pub fn context_summary(&self, max_recent: usize) -> String {
        self.short_term.summarize(max_recent)
    }
}

/// Quota limits for an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentQuota {
    /// Maximum number of actions the agent can take.
    pub max_actions: Option<u64>,
    /// Maximum number of decisions the agent can make.
    pub max_decisions: Option<u64>,
}

impl AgentQuota {
    /// Create a quota with a maximum number of actions.
    pub fn max_actions(limit: u64) -> Self {
        Self {
            max_actions: Some(limit),
            max_decisions: None,
        }
    }

    /// Create a quota with a maximum number of decisions.
    pub fn max_decisions(limit: u64) -> Self {
        Self {
            max_actions: None,
            max_decisions: Some(limit),
        }
    }

    /// Create a quota with both action and decision limits.
    pub fn new(max_actions: Option<u64>, max_decisions: Option<u64>) -> Self {
        Self {
            max_actions,
            max_decisions,
        }
    }

    /// Check if the quota is exhausted.
    pub fn is_exhausted(&self, action_count: u64, decision_count: u64) -> bool {
        if let Some(max) = self.max_actions {
            if action_count >= max {
                return true;
            }
        }
        if let Some(max) = self.max_decisions {
            if decision_count >= max {
                return true;
            }
        }
        false
    }

    /// Returns remaining actions, or None if unlimited.
    pub fn remaining_actions(&self, action_count: u64) -> Option<u64> {
        self.max_actions.map(|max| max.saturating_sub(action_count))
    }

    /// Returns remaining decisions, or None if unlimited.
    pub fn remaining_decisions(&self, decision_count: u64) -> Option<u64> {
        self.max_decisions.map(|max| max.saturating_sub(decision_count))
    }
}

/// Rate limiting policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitPolicy {
    /// Maximum actions per time window.
    pub max_actions_per_window: u64,
    /// Time window size in ticks.
    pub window_size_ticks: u64,
}

impl RateLimitPolicy {
    /// Create a new rate limit policy.
    pub fn new(max_actions_per_window: u64, window_size_ticks: u64) -> Self {
        Self {
            max_actions_per_window,
            window_size_ticks,
        }
    }

    /// Create a policy allowing N actions per tick.
    pub fn actions_per_tick(n: u64) -> Self {
        Self {
            max_actions_per_window: n,
            window_size_ticks: 1,
        }
    }
}

/// Rate limiting state for an agent.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RateLimitState {
    /// Start of the current window.
    pub window_start: WorldTime,
    /// Actions taken in the current window.
    pub actions_in_window: u64,
}

impl RateLimitState {
    /// Check if the agent is rate limited.
    pub fn is_limited(&self, now: WorldTime, policy: &RateLimitPolicy) -> bool {
        if policy.window_size_ticks == 0 {
            return false;
        }
        // Check if we're still in the same window
        let window_end = self.window_start.saturating_add(policy.window_size_ticks);
        if now < window_end {
            // Still in the same window, check action count
            self.actions_in_window >= policy.max_actions_per_window
        } else {
            // New window, not limited
            false
        }
    }

    /// Record an action.
    pub fn record_action(&mut self, now: WorldTime, policy: &RateLimitPolicy) {
        if policy.window_size_ticks == 0 {
            return;
        }
        let window_end = self.window_start.saturating_add(policy.window_size_ticks);
        if now >= window_end {
            // Start a new window
            self.window_start = now;
            self.actions_in_window = 1;
        } else {
            // Same window
            self.actions_in_window += 1;
        }
    }

    /// Reset the rate limit state.
    pub fn reset(&mut self) {
        self.window_start = 0;
        self.actions_in_window = 0;
    }
}

/// Runs the observe → decide → act loop for multiple agents.
///
/// The AgentRunner manages registered agents and coordinates their
/// interactions with the WorldKernel.
pub struct AgentRunner<B: AgentBehavior> {
    agents: BTreeMap<String, RegisteredAgent<B>>,
    /// Cursor for round-robin scheduling.
    scheduler_cursor: Option<String>,
    /// Total ticks executed.
    total_ticks: u64,
    /// Default quota for all agents (can be overridden per-agent).
    default_quota: Option<AgentQuota>,
    /// Rate limit policy for all agents.
    rate_limit_policy: Option<RateLimitPolicy>,
}

impl<B: AgentBehavior> Default for AgentRunner<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: AgentBehavior> AgentRunner<B> {
    /// Create a new agent runner.
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
            scheduler_cursor: None,
            total_ticks: 0,
            default_quota: None,
            rate_limit_policy: None,
        }
    }

    /// Create a new agent runner with a rate limit policy.
    pub fn with_rate_limit(rate_limit: RateLimitPolicy) -> Self {
        Self {
            agents: BTreeMap::new(),
            scheduler_cursor: None,
            total_ticks: 0,
            default_quota: None,
            rate_limit_policy: Some(rate_limit),
        }
    }

    /// Create a new agent runner with a default quota.
    pub fn with_quota(quota: AgentQuota) -> Self {
        Self {
            agents: BTreeMap::new(),
            scheduler_cursor: None,
            total_ticks: 0,
            default_quota: Some(quota),
            rate_limit_policy: None,
        }
    }

    /// Set the default quota for all agents.
    pub fn set_default_quota(&mut self, quota: Option<AgentQuota>) {
        self.default_quota = quota;
    }

    /// Set the rate limit policy for all agents.
    pub fn set_rate_limit(&mut self, policy: Option<RateLimitPolicy>) {
        self.rate_limit_policy = policy;
    }

    /// Get the rate limit policy.
    pub fn rate_limit_policy(&self) -> Option<&RateLimitPolicy> {
        self.rate_limit_policy.as_ref()
    }

    /// Get the default quota.
    pub fn default_quota(&self) -> Option<&AgentQuota> {
        self.default_quota.as_ref()
    }

    /// Register an agent with the runner.
    pub fn register(&mut self, behavior: B) {
        let agent_id = behavior.agent_id().to_string();
        self.agents.insert(agent_id, RegisteredAgent::new(behavior));
    }

    /// Register an agent with a specific quota.
    pub fn register_with_quota(&mut self, behavior: B, quota: AgentQuota) {
        let agent_id = behavior.agent_id().to_string();
        self.agents.insert(agent_id, RegisteredAgent::with_quota(behavior, quota));
    }

    /// Unregister an agent from the runner.
    pub fn unregister(&mut self, agent_id: &str) -> Option<RegisteredAgent<B>> {
        self.agents.remove(agent_id)
    }

    /// Get a reference to a registered agent.
    pub fn get(&self, agent_id: &str) -> Option<&RegisteredAgent<B>> {
        self.agents.get(agent_id)
    }

    /// Get a mutable reference to a registered agent.
    pub fn get_mut(&mut self, agent_id: &str) -> Option<&mut RegisteredAgent<B>> {
        self.agents.get_mut(agent_id)
    }

    /// Returns the number of registered agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Returns the IDs of all registered agents.
    pub fn agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Returns the total ticks executed.
    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    /// Run one tick of the agent loop.
    ///
    /// This method:
    /// 1. Selects the next ready agent (round-robin), respecting quota and rate limits
    /// 2. Gets an observation for that agent
    /// 3. Calls the agent's decide method
    /// 4. Submits the action to the kernel if the agent decides to act
    /// 5. Executes one step and notifies the agent of the result
    ///
    /// Returns the action result if an action was taken, or None if
    /// no agent was ready or all agents chose to wait.
    pub fn tick(&mut self, kernel: &mut WorldKernel) -> Option<AgentTickResult> {
        self.total_ticks += 1;
        let now = kernel.time();

        // Find the next ready agent using round-robin
        // Exclude agents that are quota-exhausted or rate-limited
        let rate_policy = self.rate_limit_policy.as_ref();
        let default_quota = self.default_quota.as_ref();

        let ready_agents: Vec<String> = self
            .agents
            .iter()
            .filter(|(id, agent)| {
                // Check if agent is registered in the world
                if !kernel.model().agents.contains_key(*id) {
                    return false;
                }
                // Check if agent is ready (not waiting)
                if !agent.is_ready(now) {
                    return false;
                }
                // Check quota (per-agent or default)
                let quota = agent.quota.as_ref().or(default_quota);
                if let Some(q) = quota {
                    if q.is_exhausted(agent.action_count, agent.decision_count) {
                        return false;
                    }
                }
                // Check rate limit
                if agent.is_rate_limited(now, rate_policy) {
                    return false;
                }
                true
            })
            .map(|(id, _)| id.clone())
            .collect();

        if ready_agents.is_empty() {
            return None;
        }

        // Round-robin selection
        let agent_id = match &self.scheduler_cursor {
            None => ready_agents[0].clone(),
            Some(cursor) => ready_agents
                .iter()
                .find(|id| id.as_str() > cursor.as_str())
                .cloned()
                .unwrap_or_else(|| ready_agents[0].clone()),
        };

        self.scheduler_cursor = Some(agent_id.clone());

        // Get observation for the selected agent
        let observation = match kernel.observe(&agent_id) {
            Ok(obs) => obs,
            Err(_) => return None,
        };

        // Get decision from the agent
        let agent = self.agents.get_mut(&agent_id)?;
        agent.decision_count += 1;

        let decision = agent.behavior.decide(&observation);

        match decision {
            AgentDecision::Wait => {
                Some(AgentTickResult {
                    agent_id,
                    decision: AgentDecision::Wait,
                    action_result: None,
                    skipped_reason: None,
                })
            }
            AgentDecision::WaitTicks(ticks) => {
                agent.wait_until = Some(now.saturating_add(ticks));
                Some(AgentTickResult {
                    agent_id,
                    decision: AgentDecision::WaitTicks(ticks),
                    action_result: None,
                    skipped_reason: None,
                })
            }
            AgentDecision::Act(action) => {
                agent.action_count += 1;
                // Record action for rate limiting
                let rate_policy = self.rate_limit_policy.as_ref();
                let agent = self.agents.get_mut(&agent_id).unwrap();
                agent.record_action(now, rate_policy);

                let action_id = kernel.submit_action(action.clone());
                let event = kernel.step();

                let action_result = event.map(|event| {
                    let success = !matches!(event.kind, WorldEventKind::ActionRejected { .. });
                    ActionResult {
                        action: action.clone(),
                        action_id,
                        success,
                        event,
                    }
                });

                // Notify agent of the result
                if let Some(ref result) = action_result {
                    let agent = self.agents.get_mut(&agent_id).unwrap();
                    agent.behavior.on_action_result(result);
                }

                Some(AgentTickResult {
                    agent_id,
                    decision: AgentDecision::Act(action),
                    action_result,
                    skipped_reason: None,
                })
            }
        }
    }

    /// Check if an agent is quota-exhausted.
    pub fn is_quota_exhausted(&self, agent_id: &str) -> bool {
        if let Some(agent) = self.agents.get(agent_id) {
            let quota = agent.quota.as_ref().or(self.default_quota.as_ref());
            if let Some(q) = quota {
                return q.is_exhausted(agent.action_count, agent.decision_count);
            }
        }
        false
    }

    /// Check if an agent is rate-limited.
    pub fn is_rate_limited(&self, agent_id: &str, now: WorldTime) -> bool {
        if let Some(agent) = self.agents.get(agent_id) {
            return agent.is_rate_limited(now, self.rate_limit_policy.as_ref());
        }
        false
    }

    /// Reset rate limit state for an agent.
    pub fn reset_rate_limit(&mut self, agent_id: &str) {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.rate_limit_state.reset();
        }
    }

    /// Reset rate limit state for all agents.
    pub fn reset_all_rate_limits(&mut self) {
        for agent in self.agents.values_mut() {
            agent.rate_limit_state.reset();
        }
    }

    /// Run the agent loop for a specified number of ticks.
    ///
    /// Returns the results of all ticks where an agent was active.
    pub fn run(&mut self, kernel: &mut WorldKernel, max_ticks: u64) -> Vec<AgentTickResult> {
        let mut results = Vec::new();
        for _ in 0..max_ticks {
            if let Some(result) = self.tick(kernel) {
                results.push(result);
            }
        }
        results
    }

    /// Run the agent loop until all pending actions are processed.
    pub fn run_until_idle(&mut self, kernel: &mut WorldKernel, max_ticks: u64) -> Vec<AgentTickResult> {
        let mut results = Vec::new();
        let mut consecutive_waits = 0;
        let max_consecutive_waits = self.agents.len().max(1);

        for _ in 0..max_ticks {
            match self.tick(kernel) {
                Some(result) => {
                    if result.action_result.is_some() {
                        consecutive_waits = 0;
                    } else {
                        consecutive_waits += 1;
                    }
                    results.push(result);

                    if consecutive_waits >= max_consecutive_waits {
                        break;
                    }
                }
                None => break,
            }
        }
        results
    }

    /// Broadcast an event to all registered agents.
    pub fn broadcast_event(&mut self, event: &WorldEvent) {
        for agent in self.agents.values_mut() {
            agent.behavior.on_event(event);
        }
    }

    /// Get the current metrics for the runner.
    pub fn metrics(&self) -> RunnerMetrics {
        let mut total_actions = 0u64;
        let mut total_decisions = 0u64;
        let mut agents_active = 0usize;
        let mut agents_quota_exhausted = 0usize;

        for agent in self.agents.values() {
            total_actions += agent.action_count;
            total_decisions += agent.decision_count;
            // Check quota: use agent-specific quota or default quota
            let quota = agent.quota.as_ref().or(self.default_quota.as_ref());
            let is_exhausted = quota
                .map(|q| q.is_exhausted(agent.action_count, agent.decision_count))
                .unwrap_or(false);
            if !is_exhausted {
                agents_active += 1;
            } else {
                agents_quota_exhausted += 1;
            }
        }

        RunnerMetrics {
            total_ticks: self.total_ticks,
            total_agents: self.agents.len(),
            agents_active,
            agents_quota_exhausted,
            total_actions,
            total_decisions,
            actions_per_tick: if self.total_ticks > 0 {
                total_actions as f64 / self.total_ticks as f64
            } else {
                0.0
            },
            decisions_per_tick: if self.total_ticks > 0 {
                total_decisions as f64 / self.total_ticks as f64
            } else {
                0.0
            },
            success_rate: 0.0, // Will be computed in extended metrics
        }
    }

    /// Get detailed metrics with timing information.
    pub fn metrics_with_kernel(&self, kernel: &WorldKernel) -> RunnerMetrics {
        let mut metrics = self.metrics();
        let now = kernel.time();

        // Compute agents that are rate-limited
        let mut agents_rate_limited = 0usize;
        for agent in self.agents.values() {
            if agent.is_rate_limited(now, self.rate_limit_policy.as_ref()) {
                agents_rate_limited += 1;
            }
        }

        // Update active count to exclude rate-limited agents
        metrics.agents_active = metrics.agents_active.saturating_sub(agents_rate_limited);
        metrics
    }

    /// Get per-agent statistics.
    pub fn agent_stats(&self) -> Vec<AgentStats> {
        self.agents
            .iter()
            .map(|(id, agent)| AgentStats {
                agent_id: id.clone(),
                action_count: agent.action_count,
                decision_count: agent.decision_count,
                is_quota_exhausted: agent.is_quota_exhausted(),
                wait_until: agent.wait_until,
            })
            .collect()
    }
}

/// Metrics for the AgentRunner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunnerMetrics {
    /// Total number of ticks executed.
    pub total_ticks: u64,
    /// Total number of registered agents.
    pub total_agents: usize,
    /// Number of agents that are active (not quota-exhausted, not rate-limited).
    pub agents_active: usize,
    /// Number of agents that have exhausted their quota.
    pub agents_quota_exhausted: usize,
    /// Total actions taken across all agents.
    pub total_actions: u64,
    /// Total decisions made across all agents.
    pub total_decisions: u64,
    /// Average actions per tick.
    pub actions_per_tick: f64,
    /// Average decisions per tick.
    pub decisions_per_tick: f64,
    /// Success rate of actions (0.0 to 1.0).
    pub success_rate: f64,
}

impl Default for RunnerMetrics {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            total_agents: 0,
            agents_active: 0,
            agents_quota_exhausted: 0,
            total_actions: 0,
            total_decisions: 0,
            actions_per_tick: 0.0,
            decisions_per_tick: 0.0,
            success_rate: 0.0,
        }
    }
}

/// Statistics for a single agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentStats {
    /// The agent's ID.
    pub agent_id: String,
    /// Total actions taken by this agent.
    pub action_count: u64,
    /// Total decisions made by this agent.
    pub decision_count: u64,
    /// Whether the agent has exhausted its quota.
    pub is_quota_exhausted: bool,
    /// Time until which the agent is waiting (if any).
    pub wait_until: Option<WorldTime>,
}

/// A log entry for runner events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunnerLogEntry {
    /// The tick number when this event occurred.
    pub tick: u64,
    /// The world time when this event occurred.
    pub time: WorldTime,
    /// The event kind.
    pub kind: RunnerLogKind,
}

/// Kinds of runner log events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RunnerLogKind {
    /// An agent was registered.
    AgentRegistered { agent_id: String },
    /// An agent was unregistered.
    AgentUnregistered { agent_id: String },
    /// An agent made a decision.
    AgentDecision {
        agent_id: String,
        decision: AgentDecision,
    },
    /// An action was executed.
    ActionExecuted {
        agent_id: String,
        action: Action,
        success: bool,
    },
    /// An agent was skipped.
    AgentSkipped {
        agent_id: String,
        reason: SkippedReason,
    },
    /// An agent exhausted its quota.
    QuotaExhausted { agent_id: String },
    /// An agent was rate-limited.
    RateLimited { agent_id: String },
    /// Metrics snapshot.
    MetricsSnapshot { metrics: RunnerMetrics },
}

/// Reason why an agent was skipped during scheduling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkippedReason {
    /// Agent has exhausted its quota.
    QuotaExhausted,
    /// Agent is rate limited.
    RateLimited,
    /// Agent is waiting (wait_until not reached).
    Waiting,
    /// Agent is not registered in the world.
    NotInWorld,
}

/// Result of a single agent tick.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentTickResult {
    /// The agent that was ticked.
    pub agent_id: String,
    /// The decision made by the agent.
    pub decision: AgentDecision,
    /// The result of the action if one was taken.
    pub action_result: Option<ActionResult>,
    /// Reason why the agent was skipped (if applicable).
    pub skipped_reason: Option<SkippedReason>,
}

impl AgentTickResult {
    /// Returns true if an action was taken.
    pub fn has_action(&self) -> bool {
        self.action_result.is_some()
    }

    /// Returns true if the action succeeded (or no action was taken).
    pub fn is_success(&self) -> bool {
        self.action_result.as_ref().map(|r| r.success).unwrap_or(true)
    }

    /// Returns true if the agent was skipped.
    pub fn was_skipped(&self) -> bool {
        self.skipped_reason.is_some()
    }
}

pub type AgentId = String;
pub type LocationId = String;
pub type AssetId = String;
pub type WorldTime = u64;
pub type WorldEventId = u64;
pub type ActionId = u64;

pub const CM_PER_KM: i64 = 100_000;
pub const DEFAULT_VISIBILITY_RANGE_CM: i64 = 10_000_000;
pub const DEFAULT_MOVE_COST_PER_KM_ELECTRICITY: i64 = 1;
pub const SNAPSHOT_VERSION: u32 = 1;
pub const JOURNAL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Electricity,
    Hardware,
    Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourceStock {
    pub amounts: BTreeMap<ResourceKind, i64>,
}

impl ResourceStock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, kind: ResourceKind) -> i64 {
        *self.amounts.get(&kind).unwrap_or(&0)
    }

    pub fn set(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        if amount == 0 {
            self.amounts.remove(&kind);
        } else {
            self.amounts.insert(kind, amount);
        }
        Ok(())
    }

    pub fn add(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        let current = self.get(kind);
        self.set(kind, current + amount)
    }

    pub fn remove(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        let current = self.get(kind);
        if current < amount {
            return Err(StockError::Insufficient {
                kind,
                requested: amount,
                available: current,
            });
        }
        self.set(kind, current - amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StockError {
    NegativeAmount { amount: i64 },
    Insufficient {
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResourceOwner {
    Agent { agent_id: AgentId },
    Location { location_id: LocationId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub pos: GeoPos,
    pub body: RobotBodySpec,
    pub location_id: LocationId,
    pub resources: ResourceStock,
}

impl Agent {
    pub fn new(id: impl Into<String>, location_id: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            pos,
            body: RobotBodySpec::default(),
            location_id: location_id.into(),
            resources: ResourceStock::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub resources: ResourceStock,
}

impl Location {
    pub fn new(id: impl Into<String>, name: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pos,
            resources: ResourceStock::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub owner: ResourceOwner,
    pub kind: AssetKind,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AssetKind {
    Resource { kind: ResourceKind },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldModel {
    pub agents: BTreeMap<AgentId, Agent>,
    pub locations: BTreeMap<LocationId, Location>,
    pub assets: BTreeMap<AssetId, Asset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub visibility_range_cm: i64,
    pub move_cost_per_km_electricity: i64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
        }
    }
}

impl WorldConfig {
    pub fn sanitized(mut self) -> Self {
        if self.visibility_range_cm < 0 {
            self.visibility_range_cm = 0;
        }
        if self.move_cost_per_km_electricity < 0 {
            self.move_cost_per_km_electricity = 0;
        }
        self
    }

    pub fn movement_cost(&self, distance_cm: i64) -> i64 {
        movement_cost(distance_cm, self.move_cost_per_km_electricity)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: AgentId,
    pub pos: GeoPos,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
    pub visible_locations: Vec<ObservedLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: AgentId,
    pub location_id: LocationId,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLocation {
    pub location_id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterLocation {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
    },
    RegisterAgent {
        agent_id: AgentId,
        location_id: LocationId,
    },
    MoveAgent {
        agent_id: AgentId,
        to: LocationId,
    },
    TransferResource {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldKernel {
    time: WorldTime,
    config: WorldConfig,
    next_event_id: WorldEventId,
    next_action_id: ActionId,
    pending_actions: VecDeque<ActionEnvelope>,
    journal: Vec<WorldEvent>,
    model: WorldModel,
}

impl WorldKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: WorldConfig) -> Self {
        let mut kernel = Self::default();
        kernel.config = config.sanitized();
        kernel
    }

    pub fn time(&self) -> WorldTime {
        self.time
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: WorldConfig) {
        self.config = config.sanitized();
    }

    pub fn model(&self) -> &WorldModel {
        &self.model
    }

    pub fn journal(&self) -> &[WorldEvent] {
        &self.journal
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            version: SNAPSHOT_VERSION,
            time: self.time,
            config: self.config.clone(),
            model: self.model.clone(),
            next_event_id: self.next_event_id,
            next_action_id: self.next_action_id,
            pending_actions: self.pending_actions.iter().cloned().collect(),
            journal_len: self.journal.len(),
        }
    }

    pub fn journal_snapshot(&self) -> WorldJournal {
        WorldJournal {
            version: JOURNAL_VERSION,
            events: self.journal.clone(),
        }
    }

    pub fn from_snapshot(
        snapshot: WorldSnapshot,
        journal: WorldJournal,
    ) -> Result<Self, PersistError> {
        snapshot.validate_version()?;
        journal.validate_version()?;
        if snapshot.journal_len != journal.events.len() {
            return Err(PersistError::SnapshotMismatch {
                expected: snapshot.journal_len,
                actual: journal.events.len(),
            });
        }
        Ok(Self {
            time: snapshot.time,
            config: snapshot.config.sanitized(),
            next_event_id: snapshot.next_event_id,
            next_action_id: snapshot.next_action_id,
            pending_actions: VecDeque::from(snapshot.pending_actions),
            journal: journal.events,
            model: snapshot.model,
        })
    }

    pub fn replay_from_snapshot(
        snapshot: WorldSnapshot,
        journal: WorldJournal,
    ) -> Result<Self, PersistError> {
        snapshot.validate_version()?;
        journal.validate_version()?;
        if journal.events.len() < snapshot.journal_len {
            return Err(PersistError::SnapshotMismatch {
                expected: snapshot.journal_len,
                actual: journal.events.len(),
            });
        }
        if !snapshot.pending_actions.is_empty() && journal.events.len() > snapshot.journal_len {
            return Err(PersistError::ReplayConflict {
                message: "cannot replay with pending actions in snapshot".to_string(),
            });
        }

        let mut kernel = Self {
            time: snapshot.time,
            config: snapshot.config.sanitized(),
            next_event_id: snapshot.next_event_id,
            next_action_id: snapshot.next_action_id,
            pending_actions: VecDeque::from(snapshot.pending_actions),
            journal: journal.events.clone(),
            model: snapshot.model,
        };

        for event in journal.events.iter().skip(snapshot.journal_len) {
            kernel.apply_event(event)?;
        }
        let events_after_snapshot = journal.events.len() - snapshot.journal_len;
        if events_after_snapshot > 0 {
            kernel.next_action_id = kernel
                .next_action_id
                .saturating_add(events_after_snapshot as u64);
        }

        Ok(kernel)
    }

    pub fn save_to_dir(&self, dir: impl AsRef<Path>) -> Result<(), PersistError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let snapshot_path = dir.join("snapshot.json");
        let journal_path = dir.join("journal.json");
        self.snapshot().save_json(&snapshot_path)?;
        self.journal_snapshot().save_json(&journal_path)?;
        Ok(())
    }

    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, PersistError> {
        let dir = dir.as_ref();
        let snapshot_path = dir.join("snapshot.json");
        let journal_path = dir.join("journal.json");
        let snapshot = WorldSnapshot::load_json(&snapshot_path)?;
        let journal = WorldJournal::load_json(&journal_path)?;
        Self::from_snapshot(snapshot, journal)
    }

    pub fn observe(&self, agent_id: &str) -> Result<Observation, RejectReason> {
        let Some(agent) = self.model.agents.get(agent_id) else {
            return Err(RejectReason::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        };
        let visibility_range_cm = self.config.visibility_range_cm;
        let mut visible_agents = Vec::new();
        for (other_id, other) in &self.model.agents {
            if other_id == agent_id {
                continue;
            }
            let distance_cm = great_circle_distance_cm(agent.pos, other.pos);
            if distance_cm <= visibility_range_cm {
                visible_agents.push(ObservedAgent {
                    agent_id: other_id.clone(),
                    location_id: other.location_id.clone(),
                    pos: other.pos,
                    distance_cm,
                });
            }
        }

        let mut visible_locations = Vec::new();
        for (location_id, location) in &self.model.locations {
            let distance_cm = great_circle_distance_cm(agent.pos, location.pos);
            if distance_cm <= visibility_range_cm {
                visible_locations.push(ObservedLocation {
                    location_id: location_id.clone(),
                    name: location.name.clone(),
                    pos: location.pos,
                    distance_cm,
                });
            }
        }

        Ok(Observation {
            time: self.time,
            agent_id: agent_id.to_string(),
            pos: agent.pos,
            visibility_range_cm,
            visible_agents,
            visible_locations,
        })
    }

    pub fn submit_action(&mut self, action: Action) -> ActionId {
        let id = self.next_action_id;
        self.next_action_id = self.next_action_id.saturating_add(1);
        self.pending_actions.push_back(ActionEnvelope { id, action });
        id
    }

    pub fn pending_actions(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn step(&mut self) -> Option<WorldEvent> {
        let envelope = self.pending_actions.pop_front()?;
        self.time = self.time.saturating_add(1);
        let kind = self.apply_action(envelope.action);
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());
        Some(event)
    }

    pub fn step_until_empty(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.step() {
            events.push(event);
        }
        events
    }

    fn apply_action(&mut self, action: Action) -> WorldEventKind {
        match action {
            Action::RegisterLocation {
                location_id,
                name,
                pos,
            } => {
                if self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationAlreadyExists { location_id },
                    };
                }
                let location = Location::new(location_id.clone(), name.clone(), pos);
                self.model.locations.insert(location_id.clone(), location);
                WorldEventKind::LocationRegistered {
                    location_id,
                    name,
                    pos,
                }
            }
            Action::RegisterAgent {
                agent_id,
                location_id,
            } => {
                if self.model.agents.contains_key(&agent_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentAlreadyExists { agent_id },
                    };
                }
                let Some(location) = self.model.locations.get(&location_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                };
                let agent = Agent::new(agent_id.clone(), location_id.clone(), location.pos);
                self.model.agents.insert(agent_id.clone(), agent);
                WorldEventKind::AgentRegistered {
                    agent_id,
                    location_id,
                    pos: location.pos,
                }
            }
            Action::MoveAgent { agent_id, to } => {
                let Some(location) = self.model.locations.get(&to) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id: to },
                    };
                };
                let Some(agent) = self.model.agents.get_mut(&agent_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentNotFound { agent_id },
                    };
                };
                if agent.location_id == to {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentAlreadyAtLocation {
                            agent_id,
                            location_id: to,
                        },
                    };
                }
                let from = agent.location_id.clone();
                let distance_cm = great_circle_distance_cm(agent.pos, location.pos);
                let electricity_cost = movement_cost(
                    distance_cm,
                    self.config.move_cost_per_km_electricity,
                );
                if electricity_cost > 0 {
                    let available = agent.resources.get(ResourceKind::Electricity);
                    if available < electricity_cost {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::InsufficientResource {
                                owner: ResourceOwner::Agent {
                                    agent_id: agent.id.clone(),
                                },
                                kind: ResourceKind::Electricity,
                                requested: electricity_cost,
                                available,
                            },
                        };
                    }
                    if let Err(err) = agent
                        .resources
                        .remove(ResourceKind::Electricity, electricity_cost)
                    {
                        return WorldEventKind::ActionRejected {
                            reason: match err {
                                StockError::NegativeAmount { amount } => {
                                    RejectReason::InvalidAmount { amount }
                                }
                                StockError::Insufficient {
                                    requested,
                                    available,
                                    ..
                                } => RejectReason::InsufficientResource {
                                    owner: ResourceOwner::Agent {
                                        agent_id: agent.id.clone(),
                                    },
                                    kind: ResourceKind::Electricity,
                                    requested,
                                    available,
                                },
                            },
                        };
                    }
                }
                agent.location_id = to.clone();
                agent.pos = location.pos;
                WorldEventKind::AgentMoved {
                    agent_id,
                    from,
                    to,
                    distance_cm,
                    electricity_cost,
                }
            }
            Action::TransferResource {
                from,
                to,
                kind,
                amount,
            } => match self.validate_transfer(&from, &to, kind, amount) {
                Ok(()) => {
                    if let Err(reason) = self.apply_transfer(&from, &to, kind, amount) {
                        WorldEventKind::ActionRejected { reason }
                    } else {
                        WorldEventKind::ResourceTransferred {
                            from,
                            to,
                            kind,
                            amount,
                        }
                    }
                }
                Err(reason) => WorldEventKind::ActionRejected { reason },
            },
        }
    }

    fn apply_event(&mut self, event: &WorldEvent) -> Result<(), PersistError> {
        if event.id != self.next_event_id {
            return Err(PersistError::ReplayConflict {
                message: format!(
                    "event id mismatch: expected {}, got {}",
                    self.next_event_id, event.id
                ),
            });
        }
        if event.time < self.time {
            return Err(PersistError::ReplayConflict {
                message: format!(
                    "event time regression: current {}, got {}",
                    self.time, event.time
                ),
            });
        }
        self.time = event.time;
        self.next_event_id = self.next_event_id.saturating_add(1);

        match &event.kind {
            WorldEventKind::LocationRegistered {
                location_id,
                name,
                pos,
            } => {
                if self.model.locations.contains_key(location_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location already exists: {location_id}"),
                    });
                }
                self.model.locations.insert(
                    location_id.clone(),
                    Location::new(location_id.clone(), name.clone(), *pos),
                );
            }
            WorldEventKind::AgentRegistered {
                agent_id,
                location_id,
                pos,
            } => {
                if self.model.agents.contains_key(agent_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent already exists: {agent_id}"),
                    });
                }
                if !self.model.locations.contains_key(location_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location not found: {location_id}"),
                    });
                }
                let mut agent = Agent::new(agent_id.clone(), location_id.clone(), *pos);
                agent.pos = *pos;
                self.model.agents.insert(agent_id.clone(), agent);
            }
            WorldEventKind::AgentMoved {
                agent_id,
                from,
                to,
                electricity_cost,
                ..
            } => {
                let Some(location) = self.model.locations.get(to) else {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location not found: {to}"),
                    });
                };
                let Some(agent) = self.model.agents.get_mut(agent_id) else {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent not found: {agent_id}"),
                    });
                };
                if &agent.location_id != from {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent {agent_id} not at expected location {from}"),
                    });
                }
                if *electricity_cost > 0 {
                    let available = agent.resources.get(ResourceKind::Electricity);
                    if available < *electricity_cost {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "insufficient electricity for move: requested {electricity_cost}, available {available}"
                            ),
                        });
                    }
                    if let Err(err) = agent
                        .resources
                        .remove(ResourceKind::Electricity, *electricity_cost)
                    {
                        return Err(PersistError::ReplayConflict {
                            message: format!("failed to apply move cost: {err:?}"),
                        });
                    }
                }
                agent.location_id = to.clone();
                agent.pos = location.pos;
            }
            WorldEventKind::ResourceTransferred {
                from,
                to,
                kind,
                amount,
            } => {
                if *amount <= 0 {
                    return Err(PersistError::ReplayConflict {
                        message: "transfer amount must be positive".to_string(),
                    });
                }
                self.ensure_owner_exists(from).map_err(|reason| {
                    PersistError::ReplayConflict {
                        message: format!("invalid transfer source: {reason:?}"),
                    }
                })?;
                self.ensure_owner_exists(to).map_err(|reason| PersistError::ReplayConflict {
                    message: format!("invalid transfer target: {reason:?}"),
                })?;
                self.remove_from_owner_for_replay(from, *kind, *amount)?;
                self.add_to_owner_for_replay(to, *kind, *amount)?;
            }
            WorldEventKind::ActionRejected { .. } => {}
        }

        Ok(())
    }

    fn validate_transfer(
        &self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        if amount <= 0 {
            return Err(RejectReason::InvalidAmount { amount });
        }

        self.ensure_owner_exists(from)?;
        self.ensure_owner_exists(to)?;
        self.ensure_colocated(from, to)?;

        let available = self.owner_stock(from).map(|stock| stock.get(kind)).unwrap_or(0);
        if available < amount {
            return Err(RejectReason::InsufficientResource {
                owner: from.clone(),
                kind,
                requested: amount,
                available,
            });
        }

        Ok(())
    }

    fn apply_transfer(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        self.remove_from_owner(from, kind, amount)?;
        self.add_to_owner(to, kind, amount)?;
        Ok(())
    }

    fn ensure_owner_exists(&self, owner: &ResourceOwner) -> Result<(), RejectReason> {
        match owner {
            ResourceOwner::Agent { agent_id } => {
                if self.model.agents.contains_key(agent_id) {
                    Ok(())
                } else {
                    Err(RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    })
                }
            }
            ResourceOwner::Location { location_id } => {
                if self.model.locations.contains_key(location_id) {
                    Ok(())
                } else {
                    Err(RejectReason::LocationNotFound {
                        location_id: location_id.clone(),
                    })
                }
            }
        }
    }

    fn ensure_colocated(
        &self,
        from: &ResourceOwner,
        to: &ResourceOwner,
    ) -> Result<(), RejectReason> {
        match (from, to) {
            (
                ResourceOwner::Agent { agent_id },
                ResourceOwner::Location { location_id },
            ) => {
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
                })?;
                if agent.location_id != *location_id {
                    return Err(RejectReason::AgentNotAtLocation {
                        agent_id: agent_id.clone(),
                        location_id: location_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Location { location_id },
                ResourceOwner::Agent { agent_id },
            ) => {
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
                })?;
                if agent.location_id != *location_id {
                    return Err(RejectReason::AgentNotAtLocation {
                        agent_id: agent_id.clone(),
                        location_id: location_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Agent { agent_id },
                ResourceOwner::Agent {
                    agent_id: other_agent_id,
                },
            ) => {
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
                })?;
                let other = self.model.agents.get(other_agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: other_agent_id.clone(),
                    }
                })?;
                if agent.location_id != other.location_id {
                    return Err(RejectReason::AgentsNotCoLocated {
                        agent_id: agent_id.clone(),
                        other_agent_id: other_agent_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Location { location_id },
                ResourceOwner::Location {
                    location_id: other_location_id,
                },
            ) => {
                return Err(RejectReason::LocationTransferNotAllowed {
                    from: location_id.clone(),
                    to: other_location_id.clone(),
                });
            }
        }
        Ok(())
    }

    fn owner_stock(&self, owner: &ResourceOwner) -> Option<&ResourceStock> {
        match owner {
            ResourceOwner::Agent { agent_id } => self.model.agents.get(agent_id).map(|a| &a.resources),
            ResourceOwner::Location { location_id } => {
                self.model.locations.get(location_id).map(|l| &l.resources)
            }
        }
    }

    fn remove_from_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| RejectReason::LocationNotFound {
                    location_id: location_id.clone(),
                })?,
        };

        stock.remove(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
            StockError::Insufficient {
                requested,
                available,
                ..
            } => RejectReason::InsufficientResource {
                owner: owner.clone(),
                kind,
                requested,
                available,
            },
        })
    }

    fn add_to_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| RejectReason::LocationNotFound {
                    location_id: location_id.clone(),
                })?,
        };

        stock.add(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
            StockError::Insufficient { .. } => RejectReason::InvalidAmount { amount },
        })
    }

    fn remove_from_owner_for_replay(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), PersistError> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("agent not found: {agent_id}"),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("location not found: {location_id}"),
                })?,
        };

        stock.remove(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
            StockError::Insufficient {
                requested,
                available,
                ..
            } => PersistError::ReplayConflict {
                message: format!(
                    "insufficient resource {:?}: requested {requested}, available {available}",
                    kind
                ),
            },
        })
    }

    fn add_to_owner_for_replay(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), PersistError> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("agent not found: {agent_id}"),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("location not found: {location_id}"),
                })?,
        };

        stock.add(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
            StockError::Insufficient { .. } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
        })
    }
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}

fn default_snapshot_version() -> u32 {
    SNAPSHOT_VERSION
}

fn default_journal_version() -> u32 {
    JOURNAL_VERSION
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub kind: WorldEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WorldEventKind {
    LocationRegistered {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
    },
    AgentRegistered {
        agent_id: AgentId,
        location_id: LocationId,
        pos: GeoPos,
    },
    AgentMoved {
        agent_id: AgentId,
        from: LocationId,
        to: LocationId,
        distance_cm: i64,
        electricity_cost: i64,
    },
    ResourceTransferred {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
    ActionRejected {
        reason: RejectReason,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists { agent_id: AgentId },
    AgentNotFound { agent_id: AgentId },
    LocationAlreadyExists { location_id: LocationId },
    LocationNotFound { location_id: LocationId },
    AgentAlreadyAtLocation { agent_id: AgentId, location_id: LocationId },
    InvalidAmount { amount: i64 },
    InsufficientResource {
        owner: ResourceOwner,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
    LocationTransferNotAllowed { from: LocationId, to: LocationId },
    AgentNotAtLocation { agent_id: AgentId, location_id: LocationId },
    AgentsNotCoLocated {
        agent_id: AgentId,
        other_agent_id: AgentId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    #[serde(default = "default_snapshot_version")]
    pub version: u32,
    pub time: WorldTime,
    pub config: WorldConfig,
    pub model: WorldModel,
    pub next_event_id: WorldEventId,
    pub next_action_id: ActionId,
    pub pending_actions: Vec<ActionEnvelope>,
    pub journal_len: usize,
}

impl WorldSnapshot {
    pub fn to_json(&self) -> Result<String, PersistError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, PersistError> {
        let snapshot: Self = serde_json::from_str(input)?;
        snapshot.validate_version()?;
        Ok(snapshot)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), PersistError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let snapshot: Self = read_json_from_path(path.as_ref())?;
        snapshot.validate_version()?;
        Ok(snapshot)
    }

    fn validate_version(&self) -> Result<(), PersistError> {
        if self.version == SNAPSHOT_VERSION {
            Ok(())
        } else {
            Err(PersistError::UnsupportedVersion {
                kind: "snapshot".to_string(),
                version: self.version,
                expected: SNAPSHOT_VERSION,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldJournal {
    #[serde(default = "default_journal_version")]
    pub version: u32,
    pub events: Vec<WorldEvent>,
}

impl WorldJournal {
    pub fn new() -> Self {
        Self {
            version: JOURNAL_VERSION,
            events: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, PersistError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, PersistError> {
        let journal: Self = serde_json::from_str(input)?;
        journal.validate_version()?;
        Ok(journal)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), PersistError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let journal: Self = read_json_from_path(path.as_ref())?;
        journal.validate_version()?;
        Ok(journal)
    }

    fn validate_version(&self) -> Result<(), PersistError> {
        if self.version == JOURNAL_VERSION {
            Ok(())
        } else {
            Err(PersistError::UnsupportedVersion {
                kind: "journal".to_string(),
                version: self.version,
                expected: JOURNAL_VERSION,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistError {
    Io(String),
    Serde(String),
    SnapshotMismatch { expected: usize, actual: usize },
    ReplayConflict { message: String },
    UnsupportedVersion {
        kind: String,
        version: u32,
        expected: u32,
    },
}

impl From<io::Error> for PersistError {
    fn from(err: io::Error) -> Self {
        PersistError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(err: serde_json::Error) -> Self {
        PersistError::Serde(err.to_string())
    }
}

fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), PersistError> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, PersistError> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DEFAULT_AGENT_HEIGHT_CM;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pos(lat: f64, lon: f64) -> GeoPos {
        GeoPos {
            lat_deg: lat,
            lon_deg: lon,
        }
    }

    #[test]
    fn resource_stock_add_remove() {
        let mut stock = ResourceStock::new();
        stock.add(ResourceKind::Electricity, 10).unwrap();
        stock.add(ResourceKind::Electricity, 5).unwrap();
        assert_eq!(stock.get(ResourceKind::Electricity), 15);

        stock.remove(ResourceKind::Electricity, 6).unwrap();
        assert_eq!(stock.get(ResourceKind::Electricity), 9);

        let err = stock.remove(ResourceKind::Electricity, 20).unwrap_err();
        assert!(matches!(err, StockError::Insufficient { .. }));
    }

    #[test]
    fn agent_and_location_defaults() {
        let position = pos(0.0, 0.0);
        let location = Location::new("loc-1", "base", position);
        let agent = Agent::new("agent-1", "loc-1", position);

        assert_eq!(location.id, "loc-1");
        assert_eq!(agent.location_id, "loc-1");
        assert_eq!(agent.body.height_cm, DEFAULT_AGENT_HEIGHT_CM);
    }

    #[test]
    fn world_model_starts_empty() {
        let model = WorldModel::default();
        assert!(model.agents.is_empty());
        assert!(model.locations.is_empty());
        assert!(model.assets.is_empty());
    }

    #[test]
    fn kernel_registers_and_moves_agent() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 1.0),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step().unwrap();
        let starting_energy = 500;
        kernel
            .model
            .agents
            .get_mut("agent-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, starting_energy)
            .unwrap();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        let event = kernel.step().unwrap();
        let recorded_cost = match event.kind {
            WorldEventKind::AgentMoved {
                agent_id,
                from,
                to,
                distance_cm,
                electricity_cost,
            } => {
                assert_eq!(agent_id, "agent-1");
                assert_eq!(from, "loc-1");
                assert_eq!(to, "loc-2");
                assert!(distance_cm > 0);
                assert_eq!(electricity_cost, kernel.config().movement_cost(distance_cm));
                electricity_cost
            }
            other => panic!("unexpected event: {other:?}"),
        };

        let agent = kernel.model.agents.get("agent-1").unwrap();
        assert_eq!(agent.location_id, "loc-2");
        assert_eq!(agent.pos, pos(1.0, 1.0));
        assert_eq!(
            agent.resources.get(ResourceKind::Electricity),
            starting_energy - recorded_cost
        );
    }

    #[test]
    fn kernel_move_requires_energy() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::InsufficientResource { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_rejects_move_to_same_location() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-1".to_string(),
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentAlreadyAtLocation { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_observe_visibility_range() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "near".to_string(),
            pos: pos(0.4, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-3".to_string(),
            name: "far".to_string(),
            pos: pos(1.5, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-3".to_string(),
            location_id: "loc-3".to_string(),
        });
        kernel.step_until_empty();

        let observation = kernel.observe("agent-1").unwrap();
        assert!(
            observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-1")
        );
        assert!(
            observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-2")
        );
        assert!(
            !observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-3")
        );
        assert!(
            observation
                .visible_agents
                .iter()
                .any(|agent| agent.agent_id == "agent-2")
        );
        assert!(
            !observation
                .visible_agents
                .iter()
                .any(|agent| agent.agent_id == "agent-3")
        );
        assert_eq!(observation.visibility_range_cm, DEFAULT_VISIBILITY_RANGE_CM);
    }

    #[test]
    fn kernel_config_overrides_defaults() {
        let config = WorldConfig {
            visibility_range_cm: CM_PER_KM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config.clone());
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.1, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let observation = kernel.observe("agent-1").unwrap();
        assert_eq!(observation.visibility_range_cm, config.visibility_range_cm);

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::AgentMoved { electricity_cost, .. } => {
                assert_eq!(electricity_cost, 0);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_snapshot_roundtrip() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let snapshot = kernel.snapshot();
        let journal = kernel.journal_snapshot();
        let restored = WorldKernel::from_snapshot(snapshot, journal).unwrap();

        assert_eq!(restored.time(), kernel.time());
        assert_eq!(restored.config(), kernel.config());
        assert_eq!(restored.model(), kernel.model());
        assert_eq!(restored.journal().len(), kernel.journal().len());
        assert_eq!(restored.pending_actions(), kernel.pending_actions());
    }

    #[test]
    fn kernel_persist_and_restore() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("agent-world-sim-{unique}"));

        kernel.save_to_dir(&dir).unwrap();
        let restored = WorldKernel::load_from_dir(&dir).unwrap();

        assert_eq!(restored.model(), kernel.model());
        assert_eq!(restored.journal().len(), kernel.journal().len());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_rejects_mismatched_journal_len() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.step_until_empty();

        let mut snapshot = kernel.snapshot();
        let mut journal = kernel.journal_snapshot();
        snapshot.journal_len = snapshot.journal_len + 1;
        journal.events.pop();

        let err = WorldKernel::from_snapshot(snapshot, journal).unwrap_err();
        assert!(matches!(err, PersistError::SnapshotMismatch { .. }));
    }

    #[test]
    fn snapshot_version_validation_rejects_unknown() {
        let mut snapshot = WorldKernel::new().snapshot();
        snapshot.version = SNAPSHOT_VERSION + 1;
        let json = snapshot.to_json().unwrap();
        let err = WorldSnapshot::from_json(&json).unwrap_err();
        assert!(matches!(err, PersistError::UnsupportedVersion { .. }));
    }

    #[test]
    fn journal_version_validation_rejects_unknown() {
        let mut journal = WorldKernel::new().journal_snapshot();
        journal.version = JOURNAL_VERSION + 1;
        let json = journal.to_json().unwrap();
        let err = WorldJournal::from_json(&json).unwrap_err();
        assert!(matches!(err, PersistError::UnsupportedVersion { .. }));
    }

    #[test]
    fn kernel_replay_from_snapshot() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 1.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();
        kernel
            .model
            .agents
            .get_mut("agent-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 1000)
            .unwrap();

        let snapshot = kernel.snapshot();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        kernel.step().unwrap();

        let journal = kernel.journal_snapshot();
        let replayed = WorldKernel::replay_from_snapshot(snapshot, journal).unwrap();

        assert_eq!(replayed.model(), kernel.model());
        assert_eq!(replayed.time(), kernel.time());
        assert_eq!(replayed.journal().len(), kernel.journal().len());
    }

    #[test]
    fn kernel_transfer_requires_colocation() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 1.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.step_until_empty();

        kernel
            .model
            .agents
            .get_mut("agent-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 10)
            .unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-2".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 5,
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentsNotCoLocated { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_closed_loop_example() {
        let mut kernel = WorldKernel::new();
        let loc1_pos = pos(0.0, 0.0);
        let loc2_pos = pos(2.0, 2.0);
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "plant".to_string(),
            pos: loc1_pos,
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "lab".to_string(),
            pos: loc2_pos,
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.step_until_empty();

        kernel
            .model
            .locations
            .get_mut("loc-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 1000)
            .unwrap();
        kernel
            .model
            .locations
            .get_mut("loc-2")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 20)
            .unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 500,
        });
        kernel.step().unwrap();
        let move_cost =
            kernel
                .config()
                .movement_cost(great_circle_distance_cm(loc1_pos, loc2_pos));

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        kernel.step().unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 10,
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentNotAtLocation { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-2".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 10,
        });
        kernel.step().unwrap();

        let agent = kernel.model.agents.get("agent-1").unwrap();
        assert_eq!(
            agent.resources.get(ResourceKind::Electricity),
            500 - move_cost + 10
        );
    }

    // ========================================================================
    // Agent Interface Tests
    // ========================================================================

    /// A simple test agent that moves toward a target location.
    struct PatrolAgent {
        id: String,
        target_locations: Vec<String>,
        current_target_index: usize,
        action_results: Vec<bool>,
    }

    impl PatrolAgent {
        fn new(id: impl Into<String>, target_locations: Vec<String>) -> Self {
            Self {
                id: id.into(),
                target_locations,
                current_target_index: 0,
                action_results: Vec::new(),
            }
        }
    }

    impl AgentBehavior for PatrolAgent {
        fn agent_id(&self) -> &str {
            &self.id
        }

        fn decide(&mut self, observation: &Observation) -> AgentDecision {
            if self.target_locations.is_empty() {
                return AgentDecision::Wait;
            }

            let target_id = &self.target_locations[self.current_target_index];

            // Check if we're already at the target
            let current_location = observation
                .visible_locations
                .iter()
                .find(|loc| loc.distance_cm == 0);

            if let Some(current) = current_location {
                if &current.location_id == target_id {
                    // Move to next target
                    self.current_target_index =
                        (self.current_target_index + 1) % self.target_locations.len();
                    let next_target = &self.target_locations[self.current_target_index];

                    return AgentDecision::Act(Action::MoveAgent {
                        agent_id: self.id.clone(),
                        to: next_target.clone(),
                    });
                }
            }

            // Move to current target
            AgentDecision::Act(Action::MoveAgent {
                agent_id: self.id.clone(),
                to: target_id.clone(),
            })
        }

        fn on_action_result(&mut self, result: &ActionResult) {
            self.action_results.push(result.success);
        }
    }

    /// A simple agent that always waits.
    struct WaitingAgent {
        id: String,
        wait_ticks: u64,
    }

    impl WaitingAgent {
        fn new(id: impl Into<String>, wait_ticks: u64) -> Self {
            Self {
                id: id.into(),
                wait_ticks,
            }
        }
    }

    impl AgentBehavior for WaitingAgent {
        fn agent_id(&self) -> &str {
            &self.id
        }

        fn decide(&mut self, _observation: &Observation) -> AgentDecision {
            if self.wait_ticks > 0 {
                AgentDecision::WaitTicks(self.wait_ticks)
            } else {
                AgentDecision::Wait
            }
        }
    }

    #[test]
    fn agent_decision_helpers() {
        let wait = AgentDecision::Wait;
        assert!(!wait.is_act());
        assert!(wait.action().is_none());

        let wait_ticks = AgentDecision::WaitTicks(5);
        assert!(!wait_ticks.is_act());
        assert!(wait_ticks.action().is_none());

        let action = Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        };
        let act = AgentDecision::Act(action.clone());
        assert!(act.is_act());
        assert_eq!(act.action(), Some(&action));
    }

    #[test]
    fn action_result_helpers() {
        let action = Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        };

        // Success result
        let success_event = WorldEvent {
            id: 1,
            time: 1,
            kind: WorldEventKind::AgentMoved {
                agent_id: "agent-1".to_string(),
                from: "loc-1".to_string(),
                to: "loc-2".to_string(),
                distance_cm: 1000,
                electricity_cost: 1,
            },
        };
        let success_result = ActionResult {
            action: action.clone(),
            action_id: 1,
            success: true,
            event: success_event,
        };
        assert!(!success_result.is_rejected());
        assert!(success_result.reject_reason().is_none());

        // Rejected result
        let reject_event = WorldEvent {
            id: 2,
            time: 2,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: "agent-1".to_string(),
                },
            },
        };
        let reject_result = ActionResult {
            action,
            action_id: 2,
            success: false,
            event: reject_event,
        };
        assert!(reject_result.is_rejected());
        assert!(matches!(
            reject_result.reject_reason(),
            Some(RejectReason::AgentNotFound { .. })
        ));
    }

    #[test]
    fn agent_runner_register_and_tick() {
        // Zero-cost movement config for simpler testing
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        // Setup world
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "patrol-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        // Create agent runner
        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        let patrol_agent = PatrolAgent::new(
            "patrol-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        );
        runner.register(patrol_agent);

        assert_eq!(runner.agent_count(), 1);
        assert_eq!(runner.agent_ids(), vec!["patrol-1".to_string()]);

        // Run one tick - agent should move from loc-1 to loc-2
        let result = runner.tick(&mut kernel);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.agent_id, "patrol-1");
        assert!(result.has_action());
        assert!(result.is_success());

        // Verify agent moved
        let agent = kernel.model().agents.get("patrol-1").unwrap();
        assert_eq!(agent.location_id, "loc-2");

        // Check agent stats
        let registered = runner.get("patrol-1").unwrap();
        assert_eq!(registered.action_count, 1);
        assert_eq!(registered.decision_count, 1);
        assert_eq!(registered.behavior.action_results.len(), 1);
        assert!(registered.behavior.action_results[0]);
    }

    #[test]
    fn agent_runner_round_robin() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        // Setup world with two agents
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-a".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-b".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        runner.register(PatrolAgent::new(
            "agent-a",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));
        runner.register(PatrolAgent::new(
            "agent-b",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run ticks - should alternate between agents
        let result1 = runner.tick(&mut kernel).unwrap();
        let result2 = runner.tick(&mut kernel).unwrap();
        let result3 = runner.tick(&mut kernel).unwrap();

        // Round-robin: agent-a, agent-b, agent-a
        assert_eq!(result1.agent_id, "agent-a");
        assert_eq!(result2.agent_id, "agent-b");
        assert_eq!(result3.agent_id, "agent-a");
    }

    #[test]
    fn agent_runner_wait_ticks() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "waiter".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<WaitingAgent> = AgentRunner::new();
        runner.register(WaitingAgent::new("waiter", 3));

        // First tick - agent decides to wait 3 ticks
        let result = runner.tick(&mut kernel).unwrap();
        assert_eq!(result.decision, AgentDecision::WaitTicks(3));
        assert!(!result.has_action());

        // Agent should not be ready for the next 3 ticks
        let registered = runner.get("waiter").unwrap();
        assert!(registered.wait_until.is_some());
    }

    #[test]
    fn agent_runner_run_multiple_ticks() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "patrol-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        runner.register(PatrolAgent::new(
            "patrol-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run 4 ticks
        let results = runner.run(&mut kernel, 4);
        assert_eq!(results.len(), 4);
        assert_eq!(runner.total_ticks(), 4);

        // Each result should have an action
        for result in &results {
            assert!(result.has_action());
        }

        // Agent should have patrolled back and forth
        let registered = runner.get("patrol-1").unwrap();
        assert_eq!(registered.action_count, 4);
    }

    #[test]
    fn agent_runner_unregister() {
        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        runner.register(PatrolAgent::new("agent-1", vec![]));
        runner.register(PatrolAgent::new("agent-2", vec![]));

        assert_eq!(runner.agent_count(), 2);

        let removed = runner.unregister("agent-1");
        assert!(removed.is_some());
        assert_eq!(runner.agent_count(), 1);
        assert!(runner.get("agent-1").is_none());
        assert!(runner.get("agent-2").is_some());
    }

    #[test]
    fn registered_agent_is_ready() {
        let agent = RegisteredAgent::new(PatrolAgent::new("test", vec![]));
        assert!(agent.is_ready(0));
        assert!(agent.is_ready(100));

        let mut waiting_agent = RegisteredAgent::new(PatrolAgent::new("test2", vec![]));
        waiting_agent.wait_until = Some(10);
        assert!(!waiting_agent.is_ready(5));
        assert!(waiting_agent.is_ready(10));
        assert!(waiting_agent.is_ready(15));
    }

    // ========================================================================
    // Quota and Rate Limiting Tests
    // ========================================================================

    #[test]
    fn agent_quota_max_actions() {
        let quota = AgentQuota::max_actions(5);
        assert!(!quota.is_exhausted(0, 0));
        assert!(!quota.is_exhausted(4, 0));
        assert!(quota.is_exhausted(5, 0));
        assert!(quota.is_exhausted(10, 0));

        assert_eq!(quota.remaining_actions(3), Some(2));
        assert_eq!(quota.remaining_actions(5), Some(0));
        assert_eq!(quota.remaining_decisions(10), None);
    }

    #[test]
    fn agent_quota_max_decisions() {
        let quota = AgentQuota::max_decisions(3);
        assert!(!quota.is_exhausted(0, 0));
        assert!(!quota.is_exhausted(100, 2));
        assert!(quota.is_exhausted(100, 3));
        assert!(quota.is_exhausted(0, 5));

        assert_eq!(quota.remaining_decisions(1), Some(2));
        assert_eq!(quota.remaining_actions(100), None);
    }

    #[test]
    fn agent_quota_both_limits() {
        let quota = AgentQuota::new(Some(5), Some(10));
        assert!(!quota.is_exhausted(4, 9));
        assert!(quota.is_exhausted(5, 9)); // actions exhausted
        assert!(quota.is_exhausted(4, 10)); // decisions exhausted
        assert!(quota.is_exhausted(5, 10)); // both exhausted
    }

    #[test]
    fn rate_limit_policy_actions_per_tick() {
        let policy = RateLimitPolicy::actions_per_tick(2);
        assert_eq!(policy.max_actions_per_window, 2);
        assert_eq!(policy.window_size_ticks, 1);
    }

    #[test]
    fn rate_limit_state_basic() {
        let policy = RateLimitPolicy::new(2, 5); // 2 actions per 5 ticks
        let mut state = RateLimitState::default();

        // Initially not limited
        assert!(!state.is_limited(0, &policy));

        // Record first action
        state.record_action(0, &policy);
        assert!(!state.is_limited(0, &policy));

        // Record second action - should hit the limit
        state.record_action(0, &policy);
        assert!(state.is_limited(0, &policy));
        assert!(state.is_limited(4, &policy)); // Still in same window

        // New window should reset
        assert!(!state.is_limited(5, &policy));

        // Recording action in new window resets count
        state.record_action(5, &policy);
        assert!(!state.is_limited(5, &policy));
        assert_eq!(state.window_start, 5);
        assert_eq!(state.actions_in_window, 1);
    }

    #[test]
    fn rate_limit_state_reset() {
        let policy = RateLimitPolicy::new(1, 10);
        let mut state = RateLimitState::default();

        state.record_action(0, &policy);
        assert!(state.is_limited(0, &policy));

        state.reset();
        assert!(!state.is_limited(0, &policy));
        assert_eq!(state.window_start, 0);
        assert_eq!(state.actions_in_window, 0);
    }

    #[test]
    fn agent_runner_with_quota() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "patrol-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        // Create runner with default quota of 3 actions
        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::with_quota(AgentQuota::max_actions(3));
        runner.register(PatrolAgent::new(
            "patrol-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run 5 ticks - should only get 3 results due to quota
        let mut action_count = 0;
        for _ in 0..5 {
            if let Some(result) = runner.tick(&mut kernel) {
                if result.has_action() {
                    action_count += 1;
                }
            }
        }
        assert_eq!(action_count, 3);
        assert!(runner.is_quota_exhausted("patrol-1"));
    }

    #[test]
    fn agent_runner_with_rate_limit() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "patrol-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        // Create runner with rate limit of 1 action per 10 ticks
        let mut runner: AgentRunner<PatrolAgent> =
            AgentRunner::with_rate_limit(RateLimitPolicy::new(1, 10));
        runner.register(PatrolAgent::new(
            "patrol-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // First tick should succeed
        let result = runner.tick(&mut kernel);
        assert!(result.is_some());
        assert!(result.as_ref().unwrap().has_action());

        // Second tick should be rate-limited (no agent ready)
        let result = runner.tick(&mut kernel);
        assert!(result.is_none());

        // Agent should be rate-limited
        let now = kernel.time();
        assert!(runner.is_rate_limited("patrol-1", now));
    }

    #[test]
    fn agent_runner_per_agent_quota() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-a".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-b".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        // agent-a has quota of 2 actions
        runner.register_with_quota(
            PatrolAgent::new("agent-a", vec!["loc-1".to_string(), "loc-2".to_string()]),
            AgentQuota::max_actions(2),
        );
        // agent-b has no quota (unlimited)
        runner.register(PatrolAgent::new(
            "agent-b",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run 10 ticks
        let results = runner.run(&mut kernel, 10);

        // agent-a should have only 2 actions
        let agent_a = runner.get("agent-a").unwrap();
        assert_eq!(agent_a.action_count, 2);
        assert!(runner.is_quota_exhausted("agent-a"));

        // agent-b should have more actions (limited only by round-robin)
        let agent_b = runner.get("agent-b").unwrap();
        assert!(agent_b.action_count > 2);
        assert!(!runner.is_quota_exhausted("agent-b"));

        // Should have gotten results for all ticks
        assert!(!results.is_empty());
    }

    #[test]
    fn agent_runner_reset_rate_limits() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "patrol-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> =
            AgentRunner::with_rate_limit(RateLimitPolicy::new(1, 100));
        runner.register(PatrolAgent::new(
            "patrol-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // First action
        runner.tick(&mut kernel);
        let now = kernel.time();
        assert!(runner.is_rate_limited("patrol-1", now));

        // Reset rate limit
        runner.reset_rate_limit("patrol-1");
        assert!(!runner.is_rate_limited("patrol-1", now));

        // Can take another action
        let result = runner.tick(&mut kernel);
        assert!(result.is_some());
        assert!(result.unwrap().has_action());
    }

    // ========================================================================
    // Observability and Metrics Tests
    // ========================================================================

    #[test]
    fn runner_metrics_basic() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        runner.register(PatrolAgent::new(
            "agent-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));
        runner.register(PatrolAgent::new(
            "agent-2",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Initial metrics
        let metrics = runner.metrics();
        assert_eq!(metrics.total_ticks, 0);
        assert_eq!(metrics.total_agents, 2);
        assert_eq!(metrics.agents_active, 2);
        assert_eq!(metrics.agents_quota_exhausted, 0);
        assert_eq!(metrics.total_actions, 0);
        assert_eq!(metrics.total_decisions, 0);

        // Run some ticks
        runner.run(&mut kernel, 4);

        // Check metrics after running
        let metrics = runner.metrics();
        assert_eq!(metrics.total_ticks, 4);
        assert_eq!(metrics.total_agents, 2);
        assert!(metrics.total_actions > 0);
        assert!(metrics.total_decisions > 0);
        assert!(metrics.actions_per_tick > 0.0);
        assert!(metrics.decisions_per_tick > 0.0);
    }

    #[test]
    fn runner_metrics_with_quota() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        // Create runner with quota of 2 actions
        let mut runner: AgentRunner<PatrolAgent> =
            AgentRunner::with_quota(AgentQuota::max_actions(2));
        runner.register(PatrolAgent::new(
            "agent-1",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run until quota exhausted
        runner.run(&mut kernel, 5);

        let metrics = runner.metrics();
        assert_eq!(metrics.agents_quota_exhausted, 1);
        assert_eq!(metrics.agents_active, 0); // All agents quota-exhausted
        assert_eq!(metrics.total_actions, 2); // Limited by quota
    }

    #[test]
    fn runner_agent_stats() {
        let config = WorldConfig {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: 0,
        };
        let mut kernel = WorldKernel::with_config(config);

        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(0.01, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-a".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-b".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        let mut runner: AgentRunner<PatrolAgent> = AgentRunner::new();
        runner.register_with_quota(
            PatrolAgent::new("agent-a", vec!["loc-1".to_string(), "loc-2".to_string()]),
            AgentQuota::max_actions(1),
        );
        runner.register(PatrolAgent::new(
            "agent-b",
            vec!["loc-1".to_string(), "loc-2".to_string()],
        ));

        // Run a few ticks
        runner.run(&mut kernel, 4);

        // Check per-agent stats
        let stats = runner.agent_stats();
        assert_eq!(stats.len(), 2);

        let agent_a_stats = stats.iter().find(|s| s.agent_id == "agent-a").unwrap();
        assert_eq!(agent_a_stats.action_count, 1);
        assert!(agent_a_stats.is_quota_exhausted);

        let agent_b_stats = stats.iter().find(|s| s.agent_id == "agent-b").unwrap();
        assert!(agent_b_stats.action_count >= 1);
        assert!(!agent_b_stats.is_quota_exhausted);
    }

    #[test]
    fn runner_log_entry_serialization() {
        let entry = RunnerLogEntry {
            tick: 10,
            time: 100,
            kind: RunnerLogKind::AgentRegistered {
                agent_id: "test-agent".to_string(),
            },
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("AgentRegistered"));
        assert!(json.contains("test-agent"));

        let parsed: RunnerLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tick, 10);
        assert_eq!(parsed.time, 100);
    }

    #[test]
    fn runner_metrics_default() {
        let metrics = RunnerMetrics::default();
        assert_eq!(metrics.total_ticks, 0);
        assert_eq!(metrics.total_agents, 0);
        assert_eq!(metrics.agents_active, 0);
        assert_eq!(metrics.total_actions, 0);
        assert_eq!(metrics.actions_per_tick, 0.0);
        assert_eq!(metrics.success_rate, 0.0);
    }

    // ========================================================================
    // Agent Memory Tests
    // ========================================================================

    #[test]
    fn short_term_memory_basic() {
        let mut mem = ShortTermMemory::new(5);
        assert!(mem.is_empty());
        assert_eq!(mem.capacity(), 5);

        // Add some entries
        mem.add(MemoryEntry::observation(1, "Saw agent-b"));
        mem.add(MemoryEntry::decision(2, AgentDecision::Wait));
        mem.add(MemoryEntry::note(3, "Feeling good"));

        assert_eq!(mem.len(), 3);
        assert_eq!(mem.total_added(), 3);

        // Recent entries
        let recent: Vec<_> = mem.recent(2).collect();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].time, 3); // Most recent first
        assert_eq!(recent[1].time, 2);
    }

    #[test]
    fn short_term_memory_capacity_eviction() {
        let mut mem = ShortTermMemory::new(3);

        mem.add(MemoryEntry::observation(1, "First"));
        mem.add(MemoryEntry::observation(2, "Second"));
        mem.add(MemoryEntry::observation(3, "Third"));
        assert_eq!(mem.len(), 3);

        // Adding a 4th should evict the first
        mem.add(MemoryEntry::observation(4, "Fourth"));
        assert_eq!(mem.len(), 3);
        assert_eq!(mem.total_added(), 4);

        let all: Vec<_> = mem.all().collect();
        assert_eq!(all[0].time, 2); // First was evicted
        assert_eq!(all[2].time, 4);
    }

    #[test]
    fn short_term_memory_since_filter() {
        let mut mem = ShortTermMemory::new(10);

        mem.add(MemoryEntry::observation(5, "Early"));
        mem.add(MemoryEntry::observation(10, "Middle"));
        mem.add(MemoryEntry::observation(15, "Late"));

        let since_10: Vec<_> = mem.since(10).collect();
        assert_eq!(since_10.len(), 2);
        assert!(since_10.iter().all(|e| e.time >= 10));
    }

    #[test]
    fn short_term_memory_importance_filter() {
        let mut mem = ShortTermMemory::new(10);

        mem.add(MemoryEntry::observation(1, "Low importance").with_importance(0.3));
        mem.add(MemoryEntry::observation(2, "High importance").with_importance(0.9));
        mem.add(MemoryEntry::observation(3, "Medium importance").with_importance(0.5));

        let important: Vec<_> = mem.important(0.7).collect();
        assert_eq!(important.len(), 1);
        assert_eq!(important[0].time, 2);
    }

    #[test]
    fn short_term_memory_summarize() {
        let mut mem = ShortTermMemory::new(10);

        mem.add(MemoryEntry::observation(1, "Saw base"));
        mem.add(MemoryEntry::note(2, "Planning to move"));

        let summary = mem.summarize(5);
        assert!(summary.contains("[T1]"));
        assert!(summary.contains("Saw base"));
        assert!(summary.contains("[T2]"));
        assert!(summary.contains("Planning to move"));
    }

    #[test]
    fn long_term_memory_basic() {
        let mut mem = LongTermMemory::new();
        assert!(mem.is_empty());

        let id1 = mem.store("Important discovery", 10);
        let _id2 = mem.store("Another fact", 20);

        assert_eq!(mem.len(), 2);

        // Retrieve and check
        let entry = mem.get(&id1, 25).unwrap();
        assert_eq!(entry.content, "Important discovery");
        assert_eq!(entry.created_at, 10);
        assert_eq!(entry.last_accessed, 25);
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn long_term_memory_search_by_tag() {
        let mut mem = LongTermMemory::new();

        mem.store_with_tags("Location info", 1, vec!["location".to_string()]);
        mem.store_with_tags("Agent info", 2, vec!["agent".to_string()]);
        mem.store_with_tags("Both", 3, vec!["location".to_string(), "agent".to_string()]);

        let location_results = mem.search_by_tag("location");
        assert_eq!(location_results.len(), 2);

        let agent_results = mem.search_by_tag("agent");
        assert_eq!(agent_results.len(), 2);
    }

    #[test]
    fn long_term_memory_search_by_content() {
        let mut mem = LongTermMemory::new();

        mem.store("The base is at coordinates 0,0", 1);
        mem.store("Agent-1 is friendly", 2);
        mem.store("The outpost has resources", 3);

        let results = mem.search_by_content("base");
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("base"));

        let results_case = mem.search_by_content("AGENT");
        assert_eq!(results_case.len(), 1); // Case-insensitive
    }

    #[test]
    fn long_term_memory_capacity_eviction() {
        let mut mem = LongTermMemory::with_capacity(3);

        let id1 = mem.store("Low importance", 1);
        if let Some(e) = mem.entries.get_mut(&id1) {
            e.importance = 0.1;
        }

        let id2 = mem.store("High importance", 2);
        if let Some(e) = mem.entries.get_mut(&id2) {
            e.importance = 0.9;
        }

        let id3 = mem.store("Medium importance", 3);
        if let Some(e) = mem.entries.get_mut(&id3) {
            e.importance = 0.5;
        }

        assert_eq!(mem.len(), 3);

        // Adding a 4th should evict the lowest importance
        let id4 = mem.store("New entry", 4);
        if let Some(e) = mem.entries.get_mut(&id4) {
            e.importance = 0.6;
        }

        assert_eq!(mem.len(), 3);
        assert!(mem.entries.get(&id1).is_none()); // Low importance was evicted
        assert!(mem.entries.get(&id2).is_some()); // High importance kept
    }

    #[test]
    fn long_term_memory_top_by_importance() {
        let mut mem = LongTermMemory::new();

        let id1 = mem.store("Low", 1);
        mem.entries.get_mut(&id1).unwrap().importance = 0.2;

        let id2 = mem.store("High", 2);
        mem.entries.get_mut(&id2).unwrap().importance = 0.9;

        let id3 = mem.store("Medium", 3);
        mem.entries.get_mut(&id3).unwrap().importance = 0.5;

        let top = mem.top_by_importance(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].importance, 0.9);
        assert_eq!(top[1].importance, 0.5);
    }

    #[test]
    fn agent_memory_combined() {
        let mut memory = AgentMemory::new();

        // Record some short-term memories
        memory.record_observation(1, "Saw the base");
        memory.record_decision(2, AgentDecision::Act(Action::MoveAgent {
            agent_id: "test".to_string(),
            to: "base".to_string(),
        }));
        memory.record_action_result(
            3,
            Action::MoveAgent {
                agent_id: "test".to_string(),
                to: "base".to_string(),
            },
            true,
        );
        memory.record_event(4, "Arrived at destination");

        assert_eq!(memory.short_term.len(), 4);
        assert!(memory.long_term.is_empty());

        // Get context summary
        let context = memory.context_summary(3);
        assert!(context.contains("Arrived at destination"));
        assert!(context.contains("Action"));
    }

    #[test]
    fn agent_memory_consolidation() {
        let mut memory = AgentMemory::new();

        // Add memories with varying importance
        memory.short_term.add(MemoryEntry::observation(1, "Low").with_importance(0.3));
        memory.short_term.add(MemoryEntry::observation(2, "High").with_importance(0.9));
        memory.short_term.add(MemoryEntry::observation(3, "Medium").with_importance(0.5));

        // Consolidate high-importance memories
        memory.consolidate(10, 0.8);

        // High importance should be in long-term memory
        assert_eq!(memory.long_term.len(), 1);
        let results = memory.long_term.search_by_content("High");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn memory_entry_serialization() {
        let entry = MemoryEntry::observation(10, "Test observation").with_importance(0.7);

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Observation"));
        assert!(json.contains("Test observation"));

        let parsed: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.time, 10);
        assert_eq!(parsed.importance, 0.7);
    }
}
