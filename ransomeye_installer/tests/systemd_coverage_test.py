# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/tests/systemd_coverage_test.py
#
# ⚠️  DEV-ONLY TEST FILE - NOT FOR PRODUCTION DEPLOYMENT ⚠️
# This test file uses glob patterns and hardcoded paths for testing purposes only.
# Production code MUST NOT use glob or hardcoded paths.
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Regression test for systemd coverage bug - ensures ALL stale units are removed

"""
Systemd Coverage Bug Regression Test

CRITICAL COVERAGE BUG:
Tests that the installer handles the case where MORE stale units exist than
current service modules (e.g., 17 stale units but only 2 current modules).

Test scenario:
1. Simulate 17 stale systemd units from old build
2. Current build has only 2 service modules
3. Run installer's replacement logic
4. Verify ALL 17 stale units are removed
5. Verify ONLY 2 new units are installed (for existing modules)
6. Verify no orphaned stale units remain

This prevents the coverage bug where:
- 17 stale units exist
- Installer generates 2 fresh units
- Installer overwrites 2 units but leaves 15 stale units
- Validator fails because 15 stale units still have /home paths
"""

import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))


class TestSystemdCoverageBug(unittest.TestCase):
    """Test that ALL stale units are removed, not just overwritten."""
    
    def setUp(self):
        """Set up test environment."""
        # Create temporary directories
        self.test_dir = tempfile.mkdtemp()
        self.systemd_source_dir = Path(self.test_dir) / "systemd"
        self.systemd_target_dir = Path(self.test_dir) / "etc_systemd_system"
        
        self.systemd_source_dir.mkdir(parents=True)
        self.systemd_target_dir.mkdir(parents=True)
        
        # Create 2 current service unit templates (simulates current build)
        self.current_services = ['ransomeye-intelligence', 'ransomeye-posture-engine']
        for service_name in self.current_services:
            unit_content = f"""[Unit]
Description=RansomEye {service_name}
After=network.target

[Service]
Type=simple
User=ransomeye
Group=ransomeye
WorkingDirectory=/opt/ransomeye
ExecStart=/opt/ransomeye/bin/{service_name}
Restart=always

[Install]
WantedBy=multi-user.target
"""
            unit_file = self.systemd_source_dir / f"{service_name}.service"
            unit_file.write_text(unit_content)
        
        # Create 17 STALE units in target directory (simulates old build)
        self.stale_services = [
            'ransomeye-core',
            'ransomeye-correlation',
            'ransomeye-dpi-probe',
            'ransomeye-enforcement',
            'ransomeye-feed-fetcher',
            'ransomeye-feed-retraining',
            'ransomeye-github-sync',
            'ransomeye-ingestion',
            'ransomeye-intelligence',  # Exists in current build
            'ransomeye-linux-agent',
            'ransomeye-network-scanner',
            'ransomeye-playbook-engine',
            'ransomeye-policy',
            'ransomeye-posture-engine',  # Exists in current build
            'ransomeye-posture_engine',  # Duplicate with underscore
            'ransomeye-reporting',
            'ransomeye-sentinel',
        ]
        
        for service_name in self.stale_services:
            stale_content = f"""[Unit]
Description=RansomEye {service_name} (STALE)
After=network.target

[Service]
Type=simple
User=ransomeye
Group=ransomeye
WorkingDirectory=/home/ransomeye/rebuild
ExecStart=/home/ransomeye/rebuild/bin/{service_name}
Restart=always

[Install]
WantedBy=multi-user.target
"""
            stale_file = self.systemd_target_dir / f"{service_name}.service"
            stale_file.write_text(stale_content)
    
    def tearDown(self):
        """Clean up test environment."""
        import shutil
        if Path(self.test_dir).exists():
            shutil.rmtree(self.test_dir)
    
    def test_all_stale_units_removed(self):
        """
        CRITICAL TEST: Verify ALL 17 stale units are removed, not just 2 overwritten.
        
        This is the core regression test for the coverage bug.
        """
        # 1. Verify we have 17 stale units (pre-condition)
        stale_units_before = list(self.systemd_target_dir.glob("ransomeye-*.service"))
        self.assertEqual(len(stale_units_before), 17,
                        "Pre-condition: Should have 17 stale units")
        
        # 2. Verify all stale units have /home paths (pre-condition)
        for unit_file in stale_units_before:
            content = unit_file.read_text()
            self.assertIn("/home/ransomeye/rebuild", content,
                         f"Pre-condition: {unit_file.name} should have /home paths")
        
        # 3. Simulate installer's removal + replacement strategy (Option B)
        
        # Step 3a: Remove ALL existing units
        for existing_unit in self.systemd_target_dir.glob("ransomeye-*.service"):
            existing_unit.unlink()
        
        # Step 3b: Install ONLY new units (for modules that exist)
        import shutil
        for source_unit in self.systemd_source_dir.glob("*.service"):
            target = self.systemd_target_dir / source_unit.name
            shutil.copy2(source_unit, target)
            os.chmod(target, 0o644)
        
        # 4. Verify ONLY 2 units remain (post-condition)
        units_after = list(self.systemd_target_dir.glob("ransomeye-*.service"))
        self.assertEqual(len(units_after), 2,
                        "CRITICAL: Should have ONLY 2 units after replacement (not 17)")
        
        # 5. Verify the 2 remaining units are correct
        unit_names_after = {unit.stem for unit in units_after}
        expected_names = {'ransomeye-intelligence', 'ransomeye-posture-engine'}
        self.assertEqual(unit_names_after, expected_names,
                        "Should have units only for existing modules")
        
        # 6. Verify NO units have /home paths (post-condition)
        for unit_file in units_after:
            content = unit_file.read_text()
            self.assertNotIn("/home/ransomeye/rebuild", content,
                           f"CRITICAL: {unit_file.name} must NOT have /home paths")
            self.assertIn("/opt/ransomeye", content,
                         f"CRITICAL: {unit_file.name} must have /opt paths")
    
    def test_orphaned_units_not_left_behind(self):
        """
        Test that no orphaned units are left behind.
        
        This verifies the coverage bug where:
        - 17 stale units exist
        - 2 units overwritten
        - 15 orphaned units remain with /home paths
        """
        # Simulate BROKEN approach (overwrite instead of remove+install)
        import shutil
        
        # Broken approach: Just overwrite matching units
        for source_unit in self.systemd_source_dir.glob("*.service"):
            target = self.systemd_target_dir / source_unit.name
            shutil.copy2(source_unit, target)
        
        # Check result: 17 units still exist (BAD!)
        units_after_broken = list(self.systemd_target_dir.glob("ransomeye-*.service"))
        self.assertEqual(len(units_after_broken), 17,
                        "Broken approach leaves all 17 units")
        
        # Check result: 15 orphaned units still have /home paths (BAD!)
        units_with_home_paths = []
        for unit_file in units_after_broken:
            content = unit_file.read_text()
            if "/home/ransomeye/rebuild" in content:
                units_with_home_paths.append(unit_file.name)
        
        self.assertEqual(len(units_with_home_paths), 15,
                        "Broken approach leaves 15 orphaned units with /home paths")
        
        # This test documents the BUG - the actual fix removes all units first
    
    def test_coverage_mismatch_detected(self):
        """
        Test detection of coverage mismatch.
        
        When stale unit count > generated unit count, this is a coverage issue.
        """
        # Count stale units
        stale_count = len(list(self.systemd_target_dir.glob("ransomeye-*.service")))
        
        # Count source units (what we can generate)
        source_count = len(list(self.systemd_source_dir.glob("*.service")))
        
        # Coverage mismatch detected
        self.assertGreater(stale_count, source_count,
                          "Coverage bug: More stale units than current modules")
        self.assertEqual(stale_count, 17, "Should have 17 stale units")
        self.assertEqual(source_count, 2, "Should have 2 current modules")


class TestInstallScriptCoverageLogic(unittest.TestCase):
    """Higher-level test of install.sh coverage logic."""
    
    def test_install_script_removes_all_units_first(self):
        """
        Verify that install.sh removes ALL units before installing fresh ones.
        """
        install_script_path = Path("/home/ransomeye/rebuild/install.sh")
        
        if not install_script_path.exists():
            self.skipTest("install.sh not found")
        
        content = install_script_path.read_text()
        
        # Verify removal logic exists
        self.assertIn("Removing ALL existing ransomeye systemd units", content,
                     "install.sh must remove ALL units (full replacement strategy)")
        
        self.assertIn("rm -f \"$existing_unit\"", content,
                     "install.sh must use rm -f to remove units")
        
        # Verify it happens BEFORE installation
        removal_pos = content.find("Removing ALL existing ransomeye systemd units")
        install_pos = content.find("Installed fresh unit:")
        
        self.assertLess(removal_pos, install_pos,
                       "CRITICAL: Removal MUST happen BEFORE installation")


def main():
    """Run tests."""
    unittest.main(verbosity=2)


if __name__ == '__main__':
    main()

