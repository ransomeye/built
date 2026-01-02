// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/src/temporal.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Temporal correlation logic - event time vs processing time semantics

use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;

/// Temporal window for correlation
#[derive(Debug, Clone)]
pub struct TemporalWindow {
    /// Window start (event time)
    pub start: DateTime<Utc>,
    /// Window end (event time)
    pub end: DateTime<Utc>,
    /// Window size in seconds
    pub size_seconds: u64,
}

impl TemporalWindow {
    /// Create new temporal window
    pub fn new(start: DateTime<Utc>, size_seconds: u64) -> Self {
        Self {
            start,
            end: start + Duration::seconds(size_seconds as i64),
            size_seconds,
        }
    }

    /// Check if timestamp is within window (event time)
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp < self.end
    }

    /// Slide window forward
    pub fn slide(&mut self, new_start: DateTime<Utc>) {
        self.start = new_start;
        self.end = self.start + Duration::seconds(self.size_seconds as i64);
    }
}

/// Temporal correlator for event time-based correlation
pub struct TemporalCorrelator {
    /// Current correlation window
    window: TemporalWindow,
    /// Events in window (bounded)
    events: VecDeque<TemporalEvent>,
    /// Maximum events per window
    max_events: usize,
}

/// Temporal event with event time
#[derive(Debug, Clone)]
pub struct TemporalEvent {
    pub event_id: String,
    pub entity_id: String,
    pub event_time: DateTime<Utc>,
    pub processing_time: DateTime<Utc>,
    pub signal_type: String,
}

impl TemporalCorrelator {
    /// Create new temporal correlator
    pub fn new(window_size_seconds: u64, max_events: usize) -> Self {
        Self {
            window: TemporalWindow::new(Utc::now(), window_size_seconds),
            events: VecDeque::with_capacity(max_events),
            max_events,
        }
    }

    /// Add event to correlation window (using event time)
    pub fn add_event(&mut self, event: TemporalEvent) -> bool {
        // Use event time for correlation, not processing time
        if self.window.contains(event.event_time) {
            // Check capacity
            if self.events.len() >= self.max_events {
                // Evict oldest event
                self.events.pop_front();
            }
            self.events.push_back(event);
            true
        } else {
            false
        }
    }

    /// Get events in current window (by event time)
    pub fn get_events_in_window(&self) -> Vec<&TemporalEvent> {
        self.events
            .iter()
            .filter(|e| self.window.contains(e.event_time))
            .collect()
    }

    /// Get events for entity in window
    pub fn get_entity_events(&self, entity_id: &str) -> Vec<&TemporalEvent> {
        self.events
            .iter()
            .filter(|e| e.entity_id == entity_id && self.window.contains(e.event_time))
            .collect()
    }

    /// Cleanup expired events (based on event time, not processing time)
    pub fn cleanup_expired(&mut self, _now: DateTime<Utc>) {
        // Remove events outside window (using event time)
        while let Some(front) = self.events.front() {
            if !self.window.contains(front.event_time) {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Slide window to new start time
    pub fn slide_window(&mut self, new_start: DateTime<Utc>) {
        self.window.slide(new_start);
        self.cleanup_expired(new_start);
    }

    /// Get window statistics
    pub fn get_stats(&self) -> TemporalStats {
        TemporalStats {
            window_start: self.window.start,
            window_end: self.window.end,
            event_count: self.events.len(),
            max_events: self.max_events,
        }
    }
}

/// Temporal correlation statistics
#[derive(Debug, Clone)]
pub struct TemporalStats {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub event_count: usize,
    pub max_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_window() {
        let start = Utc::now();
        let window = TemporalWindow::new(start, 300);
        
        assert!(window.contains(start));
        assert!(window.contains(start + Duration::seconds(100)));
        assert!(!window.contains(start + Duration::seconds(400)));
    }

    #[test]
    fn test_event_time_correlation() {
        let mut correlator = TemporalCorrelator::new(300, 100);
        let event_time = Utc::now() - Duration::seconds(10);
        
        let event = TemporalEvent {
            event_id: "e1".to_string(),
            entity_id: "entity1".to_string(),
            event_time,
            processing_time: Utc::now(),
            signal_type: "test".to_string(),
        };
        
        // Slide window to include event
        correlator.slide_window(event_time - Duration::seconds(5));
        
        assert!(correlator.add_event(event));
        assert_eq!(correlator.get_events_in_window().len(), 1);
    }
}

