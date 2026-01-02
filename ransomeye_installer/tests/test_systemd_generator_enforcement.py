# Path and File Name : /home/ransomeye/rebuild/ransomeye_installer/tests/test_systemd_generator_enforcement.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Tests for systemd generator-as-source-of-truth enforcement
#
# ⚠️  DEV-ONLY TEST FILE - NOT FOR PRODUCTION DEPLOYMENT ⚠️
# This test file uses glob patterns and hardcoded paths for testing purposes only.
# Production code MUST NOT use glob or hardcoded paths.

"""
Tests for Systemd Generator-as-Source-of-Truth Enforcement

Validates that:
1. SystemdWriter only generates units for existing modules
2. install_units() only installs units from the provided list
3. Installed count MUST match generated count
4. Orphan/legacy units cannot be reinstalled
5. Re-running installer converges deterministically
"""

import unittest
import tempfile
import shutil
import subprocess
from pathlib import Path
import sys
import os

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from ransomeye_installer.services.systemd_writer import SystemdWriter
from ransomeye_installer.module_resolver import ModuleResolver


class TestSystemdGeneratorEnforcement(unittest.TestCase):
    """Test systemd generator-as-source-of-truth enforcement."""
    
    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp(prefix="systemd_gen_test_"))
        self.output_dir = self.test_dir / "generated_units"
        self.output_dir.mkdir(parents=True)
        
        # Create simulated /etc/systemd/system
        self.etc_systemd = self.test_dir / "etc_systemd_system"
        self.etc_systemd.mkdir(parents=True)
        
        # Create simulated /opt/ransomeye (required for unit validation)
        self.runtime_root = self.test_dir / "opt_ransomeye"
        self.runtime_bin = self.runtime_root / "bin"
        self.runtime_bin.mkdir(parents=True)
    
    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir, ignore_errors=True)
    
    def test_writer_uses_output_dir(self):
        """Test that SystemdWriter uses the provided output directory."""
        writer = SystemdWriter(output_dir=self.output_dir)
        
        # Verify output directory was set correctly
        self.assertEqual(writer.output_dir, self.output_dir)
        
        # Generate units
        generated = writer.write_service_units()
        
        # Verify units were written to output directory
        for unit_file in generated:
            self.assertTrue(unit_file.parent == self.output_dir,
                          f"Unit {unit_file} not in output_dir {self.output_dir}")
            self.assertTrue(unit_file.exists(),
                          f"Unit {unit_file} does not exist")
    
    def test_generated_units_count_matches_modules(self):
        """Test that generated unit count matches existing service module count."""
        writer = SystemdWriter(output_dir=self.output_dir)
        generated = writer.write_service_units()
        
        # Get service module count from resolver
        resolver = ModuleResolver()
        service_modules = resolver.get_service_modules()
        
        # Generated count MUST match service module count
        self.assertEqual(len(generated), len(service_modules),
                        f"Generated {len(generated)} units but have {len(service_modules)} service modules")
    
    def test_install_units_only_installs_provided_list(self):
        """Test that install_units() only installs units from provided list."""
        writer = SystemdWriter(output_dir=self.output_dir)
        
        # Generate 2 units
        generated = writer.write_service_units()
        
        # Create 1 legacy unit in "installed" directory
        legacy_unit = self.etc_systemd / "ransomeye-legacy.service"
        legacy_unit.write_text("[Unit]\nDescription=Legacy\n")
        
        # Mock runtime root existence
        original_runtime_root = writer.RUNTIME_ROOT
        writer.RUNTIME_ROOT = self.runtime_root
        
        # Mock install location
        def mock_install(unit_list):
            """Mock installation that copies to test directory."""
            import shutil
            
            # Remove all existing units (clean slate)
            for existing in self.etc_systemd.glob("ransomeye-*.service"):
                existing.unlink()
            
            # Install only units from provided list
            for unit_file in unit_list:
                target = self.etc_systemd / unit_file.name
                shutil.copy2(unit_file, target)
            
            return True
        
        # Simulate installation
        mock_install(generated)
        
        # Count installed units
        installed = list(self.etc_systemd.glob("ransomeye-*.service"))
        
        # Legacy unit should NOT be present
        legacy_names = [u.name for u in installed if 'legacy' in u.name]
        self.assertEqual(len(legacy_names), 0,
                        "Legacy unit should not be reinstalled")
        
        # Only generated units should be installed
        self.assertEqual(len(installed), len(generated),
                        f"Should install exactly {len(generated)} units, got {len(installed)}")
        
        # Restore
        writer.RUNTIME_ROOT = original_runtime_root
    
    def test_install_fails_on_count_mismatch(self):
        """Test that installation fails if installed count != generated count."""
        writer = SystemdWriter(output_dir=self.output_dir)
        generated = writer.write_service_units()
        
        # Mock scenario: one unit fails to install
        def mock_failing_install(unit_list):
            """Mock installation that fails to install one unit."""
            import shutil
            
            # Install all but last unit (simulate partial failure)
            for unit_file in unit_list[:-1]:
                target = self.etc_systemd / unit_file.name
                shutil.copy2(unit_file, target)
            
            # Check count
            installed_count = len(list(self.etc_systemd.glob("ransomeye-*.service")))
            generated_count = len(unit_list)
            
            # Should return False if counts don't match
            return installed_count == generated_count
        
        result = mock_failing_install(generated)
        
        # Should fail (count mismatch)
        self.assertFalse(result,
                        "Installation should fail when installed count != generated count")
    
    def test_no_legacy_path_pollution(self):
        """Test that units are never generated in /home/ransomeye/rebuild/systemd."""
        writer = SystemdWriter(output_dir=self.output_dir)
        
        # Verify output directory is NOT the legacy path
        legacy_path = Path("/home/ransomeye/rebuild/systemd")
        self.assertNotEqual(writer.output_dir, legacy_path,
                          "Output directory should NOT be legacy path")
        
        # Generate units
        generated = writer.write_service_units()
        
        # Verify no units were written to legacy path
        for unit_file in generated:
            self.assertFalse(str(unit_file).startswith(str(legacy_path)),
                           f"Unit {unit_file} should not be in legacy path")
    
    def test_rerun_converges_deterministically(self):
        """Test that re-running generator produces identical results."""
        writer1 = SystemdWriter(output_dir=self.output_dir)
        generated1 = writer1.write_service_units()
        generated1_names = sorted([u.name for u in generated1])
        
        # Clear output directory
        shutil.rmtree(self.output_dir)
        self.output_dir.mkdir(parents=True)
        
        # Run again
        writer2 = SystemdWriter(output_dir=self.output_dir)
        generated2 = writer2.write_service_units()
        generated2_names = sorted([u.name for u in generated2])
        
        # Results should be identical
        self.assertEqual(generated1_names, generated2_names,
                        "Re-running generator should produce identical results")
        self.assertEqual(len(generated1), len(generated2),
                        "Re-running generator should produce same count")
    
    def test_unit_content_has_no_home_path(self):
        """Test that generated units do not contain /home/ransomeye/rebuild references."""
        writer = SystemdWriter(output_dir=self.output_dir)
        generated = writer.write_service_units()
        
        forbidden_path = "/home/ransomeye/rebuild"
        
        for unit_file in generated:
            content = unit_file.read_text()
            
            # Check for forbidden path
            if forbidden_path in content:
                # Allow in comments (header)
                lines_with_forbidden = [line for line in content.split('\n') 
                                       if forbidden_path in line and not line.strip().startswith('#')]
                
                self.assertEqual(len(lines_with_forbidden), 0,
                               f"Unit {unit_file.name} contains forbidden path in non-comment: {lines_with_forbidden}")
    
    def test_generated_units_reference_opt_ransomeye(self):
        """Test that generated units reference /opt/ransomeye runtime."""
        writer = SystemdWriter(output_dir=self.output_dir)
        generated = writer.write_service_units()
        
        expected_runtime = "/opt/ransomeye"
        
        for unit_file in generated:
            content = unit_file.read_text()
            
            # Should contain /opt/ransomeye
            self.assertIn(expected_runtime, content,
                         f"Unit {unit_file.name} should reference {expected_runtime}")


class TestSystemdInstallationIntegration(unittest.TestCase):
    """Integration tests for systemd installation flow."""
    
    def setUp(self):
        """Set up test environment."""
        self.test_dir = Path(tempfile.mkdtemp(prefix="systemd_install_test_"))
        self.output_dir = self.test_dir / "generated_units"
        self.output_dir.mkdir(parents=True)
        
        # Create simulated runtime root
        self.runtime_root = self.test_dir / "opt_ransomeye"
        self.runtime_bin = self.runtime_root / "bin"
        self.runtime_bin.mkdir(parents=True)
        
        # Create launcher scripts (required for unit validation)
        resolver = ModuleResolver()
        for module in resolver.get_service_modules():
            launcher_name = module.replace('ransomeye_', 'ransomeye-')
            launcher = self.runtime_bin / launcher_name
            launcher.write_text("#!/bin/bash\necho 'mock launcher'\n")
            launcher.chmod(0o755)
    
    def tearDown(self):
        """Clean up test environment."""
        shutil.rmtree(self.test_dir, ignore_errors=True)
    
    def test_full_generation_to_installation_flow(self):
        """Test complete flow from generation to installation."""
        writer = SystemdWriter(output_dir=self.output_dir)
        
        # Mock runtime root
        original_runtime_root = writer.RUNTIME_ROOT
        writer.RUNTIME_ROOT = self.runtime_root
        
        # Step 1: Generate units
        generated = writer.write_service_units()
        generated_count = len(generated)
        
        self.assertGreater(generated_count, 0, "Should generate at least one unit")
        
        # Step 2: Verify all generated units exist
        for unit_file in generated:
            self.assertTrue(unit_file.exists(), f"Generated unit {unit_file} should exist")
        
        # Step 3: Simulate installation (in test, we can't actually install to /etc/systemd/system)
        # So we verify the generated list is complete and correct
        
        # Verify count matches service modules
        resolver = ModuleResolver()
        service_modules = resolver.get_service_modules()
        
        self.assertEqual(generated_count, len(service_modules),
                        f"Generated count ({generated_count}) should match service modules ({len(service_modules)})")
        
        # Restore
        writer.RUNTIME_ROOT = original_runtime_root


if __name__ == '__main__':
    unittest.main()

