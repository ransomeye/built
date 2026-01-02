// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/entity_state.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Bounded entity state management with TTL-based and LRU eviction

use crate::errors::CorrelationError;
use crate::kill_chain::stages::RansomwareStage;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

/// Entity state entry
#[derive(Debug, Clone)]
pub struct EntityStateEntry {
    /// Entity identifier
    pub entity_id: String,
    /// Current kill-chain stage
    pub current_stage: Option<RansomwareStage>,
    /// Current confidence score
    pub confidence: f64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Signal history (bounded)
    pub signal_history: VecDeque<SignalEntry>,
    /// Stage transition history (bounded)
    pub transition_history: VecDeque<StageTransition>,
}

/// Signal entry in history
#[derive(Debug, Clone)]
pub struct SignalEntry {
    pub signal_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
}

/// Stage transition record
#[derive(Debug, Clone)]
pub struct StageTransition {
    pub from_stage: Option<RansomwareStage>,
    pub to_stage: RansomwareStage,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
}

/// Entity state manager with bounded memory
pub struct EntityStateManager {
    /// Entity states (bounded by max_entities)
    states: Arc<RwLock<HashMap<String, EntityStateEntry>>>,
    /// Maximum number of entities
    max_entities: usize,
    /// Maximum signals per entity
    max_signals_per_entity: usize,
    /// Maximum transitions per entity
    max_transitions_per_entity: usize,
    /// TTL for entity state (seconds)
    entity_ttl_seconds: u64,
    /// LRU queue for eviction
    lru_queue: Arc<RwLock<VecDeque<String>>>,
}

impl EntityStateManager {
    /// Create new entity state manager with bounds
    pub fn new(
        max_entities: usize,
        max_signals_per_entity: usize,
        max_transitions_per_entity: usize,
        entity_ttl_seconds: u64,
    ) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            max_entities,
            max_signals_per_entity,
            max_transitions_per_entity,
            entity_ttl_seconds,
            lru_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Get or create entity state
    pub fn get_or_create_entity(
        &self,
        entity_id: &str,
    ) -> Result<Arc<RwLock<EntityStateEntry>>, CorrelationError> {
        // Check if we need to evict
        self.ensure_capacity()?;

        let mut states = self.states.write();
        let mut lru_queue = self.lru_queue.write();

        // Update LRU
        if let Some(pos) = lru_queue.iter().position(|id| id == entity_id) {
            lru_queue.remove(pos);
        }
        lru_queue.push_back(entity_id.to_string());

        // Get or create
        if !states.contains_key(entity_id) {
            if states.len() >= self.max_entities {
                // Evict LRU entity
                if let Some(evict_id) = lru_queue.pop_front() {
                    states.remove(&evict_id);
                }
            }

            let entry = EntityStateEntry {
                entity_id: entity_id.to_string(),
                current_stage: None,
                confidence: 0.0,
                last_updated: Utc::now(),
                signal_history: VecDeque::with_capacity(self.max_signals_per_entity),
                transition_history: VecDeque::with_capacity(self.max_transitions_per_entity),
            };

            states.insert(entity_id.to_string(), entry);
        }

        // Return reference (wrapped for external access)
        // Note: In real implementation, we'd use a different pattern for thread-safe access
        // For now, we'll return the entity ID and caller uses get_entity
        Ok(Arc::new(RwLock::new(
            states.get(entity_id).unwrap().clone(),
        )))
    }

    /// Get entity state
    pub fn get_entity(&self, entity_id: &str) -> Option<EntityStateEntry> {
        let states = self.states.read();
        states.get(entity_id).cloned()
    }

    /// Update entity state
    pub fn update_entity(
        &self,
        entity_id: &str,
        updater: impl FnOnce(&mut EntityStateEntry),
    ) -> Result<(), CorrelationError> {
        let mut states = self.states.write();
        let mut lru_queue = self.lru_queue.write();

        if let Some(entry) = states.get_mut(entity_id) {
            updater(entry);
            entry.last_updated = Utc::now();

            // Update LRU
            if let Some(pos) = lru_queue.iter().position(|id| id == entity_id) {
                lru_queue.remove(pos);
            }
            lru_queue.push_back(entity_id.to_string());

            // Enforce signal history bounds
            while entry.signal_history.len() > self.max_signals_per_entity {
                entry.signal_history.pop_front();
            }

            // Enforce transition history bounds
            while entry.transition_history.len() > self.max_transitions_per_entity {
                entry.transition_history.pop_front();
            }

            Ok(())
        } else {
            Err(CorrelationError::EntityNotFound(entity_id.to_string()))
        }
    }

    /// Add signal to entity
    pub fn add_signal(
        &self,
        entity_id: &str,
        signal_type: String,
        confidence: f64,
        timestamp: chrono::DateTime<Utc>,
    ) -> Result<(), CorrelationError> {
        self.update_entity(entity_id, |entry| {
            entry.signal_history.push_back(SignalEntry {
                signal_type,
                timestamp,
                confidence,
            });
        })
    }

    /// Add stage transition
    pub fn add_transition(
        &self,
        entity_id: &str,
        from_stage: Option<RansomwareStage>,
        to_stage: RansomwareStage,
        confidence: f64,
    ) -> Result<(), CorrelationError> {
        self.update_entity(entity_id, |entry| {
            entry.current_stage = Some(to_stage);
            entry.transition_history.push_back(StageTransition {
                from_stage,
                to_stage,
                timestamp: Utc::now(),
                confidence,
            });
        })
    }

    /// Evict expired entities (TTL-based)
    pub fn evict_expired(&self) -> usize {
        let now = Utc::now();
        let ttl = Duration::from_secs(self.entity_ttl_seconds);
        let mut states = self.states.write();
        let mut lru_queue = self.lru_queue.write();

        let mut evicted = 0;
        let expired: Vec<String> = states
            .iter()
            .filter(|(_, entry)| {
                (now - entry.last_updated).to_std().unwrap_or_default() > ttl
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired {
            states.remove(&id);
            lru_queue.retain(|eid| eid != &id);
            evicted += 1;
        }

        evicted
    }

    /// Ensure capacity (evict if needed)
    fn ensure_capacity(&self) -> Result<(), CorrelationError> {
        let states = self.states.read();
        if states.len() >= self.max_entities {
            drop(states);
            // Evict expired first
            self.evict_expired();
            // Then evict LRU if still at capacity
            let mut states = self.states.write();
            let mut lru_queue = self.lru_queue.write();
            if states.len() >= self.max_entities {
                if let Some(evict_id) = lru_queue.pop_front() {
                    states.remove(&evict_id);
                }
            }
        }
        Ok(())
    }

    /// Get current entity count
    pub fn entity_count(&self) -> usize {
        self.states.read().len()
    }

    /// Get memory usage estimate (bytes)
    pub fn estimate_memory_usage(&self) -> usize {
        let states = self.states.read();
        let mut total = 0;
        for entry in states.values() {
            total += std::mem::size_of::<EntityStateEntry>();
            total += entry.signal_history.len() * std::mem::size_of::<SignalEntry>();
            total += entry.transition_history.len() * std::mem::size_of::<StageTransition>();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let manager = EntityStateManager::new(100, 50, 20, 3600);
        let _entity = manager.get_or_create_entity("test1").unwrap();
        assert_eq!(manager.entity_count(), 1);
    }

    #[test]
    fn test_eviction() {
        let manager = EntityStateManager::new(2, 10, 5, 3600);
        
        manager.get_or_create_entity("entity1").unwrap();
        manager.get_or_create_entity("entity2").unwrap();
        assert_eq!(manager.entity_count(), 2);
        
        // Adding third should evict one
        manager.get_or_create_entity("entity3").unwrap();
        assert_eq!(manager.entity_count(), 2);
    }

    #[test]
    fn test_signal_bounds() {
        let manager = EntityStateManager::new(10, 5, 5, 3600);
        manager.get_or_create_entity("test1").unwrap();
        
        // Add more signals than max
        for i in 0..10 {
            manager.add_signal("test1", format!("signal_{}", i), 0.5).unwrap();
        }
        
        let entry = manager.get_entity("test1").unwrap();
        assert!(entry.signal_history.len() <= 5);
    }
}

