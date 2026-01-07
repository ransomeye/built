# Path and File Name : /home/ransomeye/rebuild/ui/settings.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: UI Settings Framework - Backend validation, persistence, and defaults

"""
UI Settings Framework for RansomEye:
- Per-user appearance preferences (theme, density, font_size)
- Forward-compatible with RBAC and multi-user support
- Fail-safe defaults enforced
- Input validation mandatory
- Audit-logged settings changes
"""

import os
import logging
import json
from typing import Dict, Optional, Any
from enum import Enum
from datetime import datetime, timezone

logger = logging.getLogger(__name__)


# ============================================================================
# ENUMS - Validated settings values
# ============================================================================

class ThemeEnum(str, Enum):
    """UI theme options."""
    SOC_DARK = "soc_dark"
    HIGH_CONTRAST = "high_contrast"
    EXECUTIVE = "executive"


class DensityEnum(str, Enum):
    """UI density options."""
    COMPACT = "compact"
    COMFORTABLE = "comfortable"


class FontSizeEnum(str, Enum):
    """Font size options."""
    SMALL = "small"
    MEDIUM = "medium"
    LARGE = "large"


# ============================================================================
# DEFAULT VALUES (Env-configurable, fail-safe fallbacks)
# ============================================================================

# Default values from environment, with fail-safe fallbacks
DEFAULT_THEME = os.environ.get("RANSOMEYE_UI_DEFAULT_THEME", ThemeEnum.SOC_DARK.value)
DEFAULT_DENSITY = os.environ.get("RANSOMEYE_UI_DEFAULT_DENSITY", DensityEnum.COMFORTABLE.value)
DEFAULT_FONT_SIZE = os.environ.get("RANSOMEYE_UI_DEFAULT_FONT_SIZE", FontSizeEnum.MEDIUM.value)

# Validate defaults on import (fail-closed if invalid)
def _validate_default_enum(value: str, enum_class: type[Enum], default_fallback: str) -> str:
    """Validate and return enum value, or fallback if invalid."""
    try:
        enum_class(value)
        return value
    except ValueError:
        logger.warning(f"Invalid default value '{value}' for {enum_class.__name__}, using fallback '{default_fallback}'")
        return default_fallback

DEFAULT_THEME = _validate_default_enum(DEFAULT_THEME, ThemeEnum, ThemeEnum.SOC_DARK.value)
DEFAULT_DENSITY = _validate_default_enum(DEFAULT_DENSITY, DensityEnum, DensityEnum.COMFORTABLE.value)
DEFAULT_FONT_SIZE = _validate_default_enum(DEFAULT_FONT_SIZE, FontSizeEnum, FontSizeEnum.MEDIUM.value)


# ============================================================================
# SETTINGS VALIDATION
# ============================================================================

class SettingsValidationError(Exception):
    """Raised when settings validation fails."""
    pass


def validate_settings(data: Dict[str, Any]) -> Dict[str, str]:
    """
    Validate UI settings input.
    
    Args:
        data: Dictionary with settings keys (theme, density, font_size)
        
    Returns:
        Validated settings dictionary with only valid enum values
        
    Raises:
        SettingsValidationError: If any value is invalid
    """
    validated = {}
    errors = []
    
    # Validate theme
    if "theme" in data:
        theme = data["theme"]
        if not isinstance(theme, str):
            errors.append(f"theme must be a string, got {type(theme).__name__}")
        else:
            try:
                validated["theme"] = ThemeEnum(theme).value
            except ValueError:
                errors.append(f"Invalid theme: '{theme}'. Must be one of: {[e.value for e in ThemeEnum]}")
    
    # Validate density
    if "density" in data:
        density = data["density"]
        if not isinstance(density, str):
            errors.append(f"density must be a string, got {type(density).__name__}")
        else:
            try:
                validated["density"] = DensityEnum(density).value
            except ValueError:
                errors.append(f"Invalid density: '{density}'. Must be one of: {[e.value for e in DensityEnum]}")
    
    # Validate font_size
    if "font_size" in data:
        font_size = data["font_size"]
        if not isinstance(font_size, str):
            errors.append(f"font_size must be a string, got {type(font_size).__name__}")
        else:
            try:
                validated["font_size"] = FontSizeEnum(font_size).value
            except ValueError:
                errors.append(f"Invalid font_size: '{font_size}'. Must be one of: {[e.value for e in FontSizeEnum]}")
    
    if errors:
        raise SettingsValidationError(f"Settings validation failed: {'; '.join(errors)}")
    
    return validated


def get_default_settings() -> Dict[str, str]:
    """Get default settings (fail-safe)."""
    return {
        "theme": DEFAULT_THEME,
        "density": DEFAULT_DENSITY,
        "font_size": DEFAULT_FONT_SIZE
    }


def merge_with_defaults(settings: Dict[str, str]) -> Dict[str, str]:
    """Merge provided settings with defaults (fail-safe)."""
    defaults = get_default_settings()
    return {**defaults, **settings}


# ============================================================================
# USER IDENTITY RESOLUTION (Forward-compatible with RBAC)
# ============================================================================

def get_user_identity() -> str:
    """
    Get current user identity for settings association.
    
    Currently returns 'system-default' until user authentication is implemented.
    Forward-compatible: will be replaced with actual user ID when RBAC is added.
    
    Returns:
        User identifier string (currently 'system-default')
    """
    # Future: Replace with actual user ID from session/auth context
    # For now, use system-default for single-user deployments
    return os.environ.get("RANSOMEYE_UI_USER_ID", "system-default")

