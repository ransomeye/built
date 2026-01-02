// Path and File Name : /home/ransomeye/rebuild/ransomeye_core/correlation/output/confidence.rs
// Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
// Details: Confidence score representation and validation

use serde::{Deserialize, Serialize};

/// Confidence score (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
    /// Create new confidence score
    pub fn new(value: f64) -> Option<Self> {
        if value >= 0.0 && value <= 1.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Get as percentage
    pub fn as_percentage(&self) -> f64 {
        self.0 * 100.0
    }

    /// Check if meets threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

impl From<f64> for ConfidenceScore {
    fn from(value: f64) -> Self {
        Self(value.max(0.0).min(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_score_validation() {
        assert!(ConfidenceScore::new(0.5).is_some());
        assert!(ConfidenceScore::new(1.0).is_some());
        assert!(ConfidenceScore::new(0.0).is_some());
        assert!(ConfidenceScore::new(1.5).is_none());
        assert!(ConfidenceScore::new(-0.1).is_none());
    }

    #[test]
    fn test_confidence_threshold() {
        let conf = ConfidenceScore::new(0.7).unwrap();
        assert!(conf.meets_threshold(0.6));
        assert!(!conf.meets_threshold(0.8));
    }
}

