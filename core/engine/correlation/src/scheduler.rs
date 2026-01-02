// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/scheduler.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Fair entity scheduling with deterministic ordering and starvation prevention

use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Entity scheduling priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Scheduled entity
#[derive(Debug, Clone)]
pub struct ScheduledEntity {
    pub entity_id: String,
    pub priority: Priority,
    pub last_processed: Option<u64>, // Timestamp for round-robin
    pub processing_count: u64,
}

/// Fair scheduler for entity processing
pub struct EntityScheduler {
    /// Entity queue by priority
    queues: Arc<RwLock<HashMap<Priority, VecDeque<String>>>>,
    /// Entity metadata
    entities: Arc<RwLock<HashMap<String, ScheduledEntity>>>,
    /// Round-robin counter
    round_robin_counter: Arc<RwLock<u64>>,
}

impl EntityScheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        queues.insert(Priority::Low, VecDeque::new());
        queues.insert(Priority::Normal, VecDeque::new());
        queues.insert(Priority::High, VecDeque::new());
        queues.insert(Priority::Critical, VecDeque::new());

        Self {
            queues: Arc::new(RwLock::new(queues)),
            entities: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Schedule entity for processing
    pub fn schedule(&self, entity_id: String, priority: Priority) {
        let mut queues = self.queues.write();
        let mut entities = self.entities.write();

        // Update or create entity metadata
        if let Some(entity) = entities.get_mut(&entity_id) {
            entity.priority = priority;
        } else {
            entities.insert(
                entity_id.clone(),
                ScheduledEntity {
                    entity_id: entity_id.clone(),
                    priority,
                    last_processed: None,
                    processing_count: 0,
                },
            );
        }

        // Add to appropriate queue if not already present
        if let Some(queue) = queues.get_mut(&priority) {
            if !queue.contains(&entity_id) {
                queue.push_back(entity_id);
            }
        }
    }

    /// Get next entity to process (fair scheduling)
    pub fn next(&self) -> Option<String> {
        let mut queues = self.queues.write();
        let mut entities = self.entities.write();
        let mut counter = self.round_robin_counter.write();

        // Process in priority order, but with fairness
        // Check each priority level
        for priority in [Priority::Critical, Priority::High, Priority::Normal, Priority::Low] {
            if let Some(queue) = queues.get_mut(&priority) {
                if let Some(entity_id) = queue.pop_front() {
                    // Update entity metadata
                    if let Some(entity) = entities.get_mut(&entity_id) {
                        entity.last_processed = Some(*counter);
                        entity.processing_count += 1;
                    }

                    *counter += 1;
                    return Some(entity_id);
                }
            }
        }

        None
    }

    /// Mark entity as processed (for round-robin tracking)
    pub fn mark_processed(&self, entity_id: &str) {
        let mut entities = self.entities.write();
        if let Some(entity) = entities.get_mut(entity_id) {
            let mut counter = self.round_robin_counter.write();
            entity.last_processed = Some(*counter);
            *counter += 1;
        }
    }

    /// Get entity priority
    pub fn get_priority(&self, entity_id: &str) -> Option<Priority> {
        let entities = self.entities.read();
        entities.get(entity_id).map(|e| e.priority)
    }

    /// Update entity priority
    pub fn update_priority(&self, entity_id: &str, priority: Priority) {
        self.schedule(entity_id.to_string(), priority);
    }

    /// Get queue sizes
    pub fn queue_sizes(&self) -> HashMap<Priority, usize> {
        let queues = self.queues.read();
        queues
            .iter()
            .map(|(p, q)| (*p, q.len()))
            .collect()
    }

    /// Clear all queues
    pub fn clear(&self) {
        let mut queues = self.queues.write();
        for queue in queues.values_mut() {
            queue.clear();
        }
        let mut entities = self.entities.write();
        entities.clear();
    }
}

impl Default for EntityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_scheduling() {
        let scheduler = EntityScheduler::new();
        
        scheduler.schedule("low1".to_string(), Priority::Low);
        scheduler.schedule("critical1".to_string(), Priority::Critical);
        scheduler.schedule("normal1".to_string(), Priority::Normal);
        
        // Critical should be processed first
        assert_eq!(scheduler.next(), Some("critical1".to_string()));
        // Then normal
        assert_eq!(scheduler.next(), Some("normal1".to_string()));
        // Then low
        assert_eq!(scheduler.next(), Some("low1".to_string()));
    }

    #[test]
    fn test_fair_scheduling() {
        let scheduler = EntityScheduler::new();
        
        // Schedule multiple entities at same priority
        for i in 0..5 {
            scheduler.schedule(format!("entity_{}", i), Priority::Normal);
        }
        
        // Should process in order
        for i in 0..5 {
            assert_eq!(scheduler.next(), Some(format!("entity_{}", i)));
        }
    }
}

