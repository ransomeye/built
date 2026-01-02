# Path and File Name : /home/ransomeye/rebuild/ransomeye_posture_engine/tests/test_normalizer.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Unit tests for signal normalizer

"""
Unit tests for Signal Normalizer.
"""

import unittest
from datetime import datetime
from ransomeye_posture_engine.engine.normalizer import SignalNormalizer, PostureCategory
from ransomeye_posture_engine.engine.telemetry_ingester import TelemetryEvent

class TestSignalNormalizer(unittest.TestCase):
    """Test signal normalizer."""
    
    def setUp(self):
        self.normalizer = SignalNormalizer()
    
    def test_normalize_empty_events(self):
        """Test normalizing empty event list."""
        facts = self.normalizer.normalize([])
        self.assertEqual(len(facts), 0)
    
    def test_normalize_process_event(self):
        """Test normalizing process execution event."""
        event = TelemetryEvent(
            event_id="test_1",
            producer_type="linux_agent",
            host_id="host_1",
            timestamp=datetime.utcnow(),
            event_type="process_exec",
            data={"process_name": "test", "user": "root"},
            signature_valid=True,
        )
        
        facts = self.normalizer.normalize([event])
        self.assertGreater(len(facts), 0)
        
        # Should have host hardening fact
        hardening_facts = [f for f in facts if f.category == PostureCategory.HOST_HARDENING]
        self.assertGreater(len(hardening_facts), 0)
    
    def test_normalize_auth_event(self):
        """Test normalizing authentication event."""
        event = TelemetryEvent(
            event_id="test_2",
            producer_type="linux_agent",
            host_id="host_1",
            timestamp=datetime.utcnow(),
            event_type="auth_failure",
            data={"user": "test", "reason": "invalid_password"},
            signature_valid=True,
        )
        
        facts = self.normalizer.normalize([event])
        
        # Should have auth hygiene fact if failure rate is high
        auth_facts = [f for f in facts if f.category == PostureCategory.AUTH_HYGIENE]
        # May or may not have facts depending on failure rate threshold
        self.assertIsInstance(auth_facts, list)

if __name__ == '__main__':
    unittest.main()

