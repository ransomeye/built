# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/ai_registry/versioning.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: AI model versioning - manages model versions and compatibility

"""
AI Model Versioning: Manages model versions and compatibility.
Ensures version compatibility and prevents breaking changes.
"""

from typing import Dict, List, Optional, Tuple
from semver import VersionInfo


class AIModelVersioning:
    """Manages AI model versioning."""
    
    def __init__(self):
        self.compatibility_matrix = {
            '1.0.0': ['1.0.1', '1.0.2', '1.1.0'],
            '1.1.0': ['1.1.1', '1.2.0'],
            '2.0.0': ['2.0.1', '2.1.0']
        }
    
    def parse_version(self, version_str: str) -> VersionInfo:
        """Parse version string to VersionInfo."""
        try:
            return VersionInfo.parse(version_str)
        except Exception:
            raise ValueError(f"Invalid version format: {version_str}")
    
    def is_compatible(self, current_version: str, target_version: str) -> bool:
        """
        Check if versions are compatible.
        
        Args:
            current_version: Current version
            target_version: Target version
        
        Returns:
            True if compatible
        """
        current = self.parse_version(current_version)
        target = self.parse_version(target_version)
        
        # Same major version is compatible
        if current.major == target.major:
            return True
        
        # Check compatibility matrix
        compatible_versions = self.compatibility_matrix.get(current_version, [])
        return target_version in compatible_versions
    
    def get_latest_version(self, versions: List[str]) -> str:
        """Get latest version from list."""
        parsed_versions = [self.parse_version(v) for v in versions]
        latest = max(parsed_versions)
        return str(latest)
    
    def validate_version_compatibility(self, model_versions: Dict[str, str]) -> Tuple[bool, List[str]]:
        """
        Validate version compatibility across models.
        
        Args:
            model_versions: Dictionary of model_name -> version
        
        Returns:
            Tuple of (is_compatible: bool, errors: List[str])
        """
        errors = []
        
        # Check each model version is valid
        for model_name, version in model_versions.items():
            try:
                self.parse_version(version)
            except Exception as e:
                errors.append(f"Invalid version for {model_name}: {version}")
        
        return len(errors) == 0, errors


def main():
    """CLI entry point for versioning."""
    versioning = AIModelVersioning()
    
    # Example compatibility check
    is_compat = versioning.is_compatible('1.0.0', '1.0.1')
    print(f"Version compatibility: {is_compat}")


if __name__ == '__main__':
    main()

