# Path and File Name : /home/ransomeye/rebuild/ui/dashboard_engine.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Dashboard engine that loads JSON definitions and renders panels via responsive grid

"""
RansomEye Dashboard Engine
- Loads dashboard definitions from JSON files (DB-backed later)
- Renders panels via 12-column responsive grid
- Supports panel positioning (x, y, w, h)
- Supports refresh intervals per panel
- No SQL in frontend templates
- No detection logic in UI
- No schema assumptions
"""

import json
import logging
import shutil
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
from overlay_manager import OverlayManager

logger = logging.getLogger(__name__)


class DashboardEngine:
    """Engine for loading and rendering dashboard definitions from JSON."""
    
    def __init__(self, dashboards_dir: Path):
        """
        Initialize dashboard engine.
        
        Args:
            dashboards_dir: Directory containing JSON dashboard definitions (system dashboards)
        """
        self.dashboards_dir = Path(dashboards_dir)
        self.dashboards_dir.mkdir(parents=True, exist_ok=True)
        self._cache: Dict[str, Dict] = {}
        self._cache_timestamps: Dict[str, float] = {}
        
        # Initialize overlay manager for user customizations
        self.overlay_manager = OverlayManager(self.dashboards_dir.parent)
    
    def load_dashboard(self, dashboard_name: str, include_overlay: bool = True) -> Optional[Dict[str, Any]]:
        """
        Load a dashboard definition with optional user overlay merge.
        
        Load order:
        1. Load system dashboard from dashboards_dir
        2. Load user overlay if exists and include_overlay=True
        3. Merge overlay onto system dashboard
        
        Args:
            dashboard_name: Name of dashboard (without .json extension)
            include_overlay: Whether to include user overlay (default: True)
            
        Returns:
            Dashboard definition dict or None if not found
        """
        dashboard_path = self.dashboards_dir / f"{dashboard_name}.json"
        
        if not dashboard_path.exists():
            logger.warning(f"System dashboard not found: {dashboard_path}")
            return None
        
        try:
            # Check cache timestamp (need to check both system and overlay)
            system_mtime = dashboard_path.stat().st_mtime
            cache_key = dashboard_name
            
            # Check overlay timestamp if exists
            overlay = None
            overlay_mtime = None
            if include_overlay:
                overlay = self.overlay_manager.load_overlay(dashboard_name)
                if overlay:
                    overlay_path = self.overlay_manager.get_overlay_path(dashboard_name)
                    if overlay_path.exists():
                        overlay_mtime = overlay_path.stat().st_mtime
            
            # Cache key includes overlay state
            cache_timestamp = max(system_mtime, overlay_mtime or 0)
            
            if cache_key in self._cache and self._cache_timestamps.get(cache_key) == cache_timestamp:
                return self._cache[cache_key]
            
            # Load system dashboard
            with open(dashboard_path, 'r', encoding='utf-8') as f:
                system_dashboard = json.load(f)
            
            # Auto-assign to General folder if folder_id is missing (fail-soft migration)
            if 'folder_id' not in system_dashboard:
                system_dashboard['folder_id'] = 'general'
                logger.info(f"Auto-assigned dashboard '{dashboard_name}' to 'general' folder")
                # Note: We don't auto-save system dashboards here to avoid overwriting
            
            # Merge with user overlay if present
            if include_overlay and overlay:
                dashboard = self.overlay_manager.merge_overlay(system_dashboard, overlay)
                logger.debug(f"Loaded dashboard '{dashboard_name}' with user overlay")
            else:
                dashboard = system_dashboard.copy()
            
            # Validate merged dashboard structure
            if not self._validate_dashboard(dashboard):
                logger.error(f"Invalid dashboard structure after merge: {dashboard_name}")
                # Fail-soft: return system dashboard if overlay is invalid
                if include_overlay and overlay:
                    logger.warning(f"Overlay invalid, falling back to system dashboard: {dashboard_name}")
                    dashboard = system_dashboard
                    if not self._validate_dashboard(dashboard):
                        return None
                else:
                    return None
            
            # Cache dashboard
            self._cache[cache_key] = dashboard
            self._cache_timestamps[cache_key] = cache_timestamp
            
            logger.info(f"Loaded dashboard: {dashboard_name} (with overlay: {overlay is not None})")
            return dashboard
            
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON in dashboard {dashboard_name}: {e}")
            return None
        except Exception as e:
            logger.error(f"Error loading dashboard {dashboard_name}: {e}", exc_info=True)
            return None
    
    def _validate_dashboard(self, dashboard: Dict) -> bool:
        """
        Validate dashboard structure.
        
        Args:
            dashboard: Dashboard definition dict
            
        Returns:
            True if valid, False otherwise
        """
        required_fields = ['name', 'panels']
        if not all(field in dashboard for field in required_fields):
            return False
        
        if not isinstance(dashboard['panels'], list):
            return False
        
        # Validate each panel
        for panel in dashboard['panels']:
            if not isinstance(panel, dict):
                return False
            
            # Required panel fields
            if 'id' not in panel or 'title' not in panel:
                return False
            
            # Validate grid positioning (x, y, w, h)
            grid_fields = ['x', 'y', 'w', 'h']
            for field in grid_fields:
                if field in panel:
                    if not isinstance(panel[field], int) or panel[field] < 0:
                        return False
                    if field in ['w', 'h'] and panel[field] > 12:
                        return False  # Max 12 columns
        
        return True
    
    def list_dashboards(self) -> List[str]:
        """
        List all available dashboard names.
        
        Returns:
            List of dashboard names (without .json extension)
        """
        dashboards = []
        for json_file in self.dashboards_dir.glob("*.json"):
            # Skip backup and temp files
            if json_file.suffix == '.json' and not json_file.name.endswith('.backup') and not json_file.name.endswith('.tmp'):
                dashboards.append(json_file.stem)
        return sorted(dashboards)
    
    def list_dashboards_with_folders(self) -> List[Dict[str, Any]]:
        """
        List all dashboards with their folder assignments.
        
        Returns:
            List of dicts with 'name' and 'folder_id' keys
        """
        dashboards = []
        for dashboard_name in self.list_dashboards():
            dashboard = self.load_dashboard(dashboard_name)
            if dashboard:
                dashboards.append({
                    'name': dashboard_name,
                    'folder_id': dashboard.get('folder_id', 'general')
                })
        return dashboards
    
    def render_dashboard_html(self, dashboard: Dict[str, Any], 
                            panel_data: Optional[Dict[str, Any]] = None) -> str:
        """
        Render dashboard HTML with panel grid.
        
        Args:
            dashboard: Dashboard definition dict
            panel_data: Optional dict mapping panel IDs to their data
            
        Returns:
            HTML string for dashboard
        """
        if not panel_data:
            panel_data = {}
        
        html_parts = []
        html_parts.append('<div class="dashboard-container">')
        html_parts.append(f'<div class="dashboard-title">{dashboard.get("title", dashboard["name"])}</div>')
        
        # Render panels in grid
        html_parts.append('<div class="dashboard-grid">')
        
        for panel in dashboard.get('panels', []):
            panel_id = panel['id']
            panel_html = self._render_panel(panel, panel_data.get(panel_id))
            html_parts.append(panel_html)
        
        html_parts.append('</div>')  # Close dashboard-grid
        html_parts.append('</div>')  # Close dashboard-container
        
        return '\n'.join(html_parts)
    
    def _render_panel(self, panel: Dict[str, Any], data: Optional[Any] = None) -> str:
        """
        Render a single panel HTML.
        
        Args:
            panel: Panel definition dict
            data: Optional panel data
            
        Returns:
            HTML string for panel
        """
        panel_id = panel['id']
        title = panel.get('title', 'Panel')
        
        # Grid positioning (default to full width if not specified)
        x = panel.get('x', 0)
        y = panel.get('y', 0)
        w = panel.get('w', 12)
        h = panel.get('h', 1)
        
        # Refresh interval (in seconds, optional)
        refresh_interval = panel.get('refresh_interval', None)
        
        # Status badge (healthy/warning/critical)
        status = panel.get('status', 'healthy')
        if data and isinstance(data, dict) and 'status' in data:
            status = data['status']
        
        # Last updated timestamp
        last_updated = datetime.now(timezone.utc).isoformat()
        if data and isinstance(data, dict) and 'last_updated' in data:
            last_updated = data['last_updated']
        
        # Panel content
        content = panel.get('content', '')
        if data and isinstance(data, dict) and 'content' in data:
            content = data['content']
        elif data and not isinstance(data, dict):
            # If data is a simple value, use it as content
            content = str(data) if data is not None else 'Metric unavailable'
        
        # Build panel HTML
        panel_html = f'''
        <div class="panel" 
             data-panel-id="{panel_id}"
             data-x="{x}" 
             data-y="{y}" 
             data-w="{w}" 
             data-h="{h}"
             {f'data-refresh-interval="{refresh_interval}"' if refresh_interval else ''}
             style="grid-column: span {w}; grid-row: span {h}">
            <div class="panel-header">
                <div class="panel-title">{title}</div>
                <div class="panel-status-group">
                    <span class="status-badge status-{status}">{status}</span>
                    <span class="panel-timestamp" data-timestamp="{last_updated}"></span>
                </div>
            </div>
            <div class="panel-body">
                {content if content else '<div class="panel-unavailable">Metric unavailable</div>'}
            </div>
        </div>
        '''
        
        return panel_html
    
    def get_panel_refresh_intervals(self, dashboard: Dict[str, Any]) -> Dict[str, int]:
        """
        Extract refresh intervals for all panels in a dashboard.
        
        Args:
            dashboard: Dashboard definition dict
            
        Returns:
            Dict mapping panel IDs to refresh intervals (seconds)
        """
        intervals = {}
        for panel in dashboard.get('panels', []):
            panel_id = panel['id']
            if 'refresh_interval' in panel:
                intervals[panel_id] = panel['refresh_interval']
        return intervals
    
    def get_dashboard_source(self, dashboard_name: str) -> Dict[str, Any]:
        """
        Get dashboard source information (system vs user overlay).
        
        Args:
            dashboard_name: Dashboard name
            
        Returns:
            Dict with 'source' ('system', 'user', or 'merged'), 
            'has_overlay' (bool), and 'user_id' (str or None)
        """
        system_path = self.dashboards_dir / f"{dashboard_name}.json"
        has_system = system_path.exists()
        
        user_id = self.overlay_manager.get_user_id()
        has_overlay = self.overlay_manager.has_overlay(dashboard_name, user_id)
        
        if has_system and has_overlay:
            source = 'merged'
        elif has_overlay:
            source = 'user'
        elif has_system:
            source = 'system'
        else:
            source = 'none'
        
        return {
            'source': source,
            'has_overlay': has_overlay,
            'has_system': has_system,
            'user_id': user_id if has_overlay else None
        }
    
    def clear_cache(self):
        """Clear dashboard cache."""
        self._cache.clear()
        self._cache_timestamps.clear()
        logger.info("Dashboard cache cleared")
    
    def save_dashboard(self, dashboard: Dict[str, Any], dashboard_name: Optional[str] = None, 
                      save_as_overlay: bool = True) -> bool:
        """
        Save dashboard definition.
        
        By default, saves as user overlay (never modifies system dashboards).
        Set save_as_overlay=False to save system dashboard (use with caution).
        
        Args:
            dashboard: Dashboard definition dict (must be validated)
            dashboard_name: Optional dashboard name (if not provided, uses dashboard['name'])
            save_as_overlay: If True, save as user overlay; if False, save as system dashboard
            
        Returns:
            True if saved successfully, False otherwise
        """
        # Get dashboard name
        if not dashboard_name:
            dashboard_name = dashboard.get('name')
        
        if not dashboard_name:
            logger.error("Dashboard name is required")
            return False
        
        # Validate dashboard structure strictly
        if not self._validate_dashboard_strict(dashboard):
            logger.error(f"Dashboard validation failed for {dashboard_name}")
            return False
        
        # Check if this is a system dashboard path
        system_dashboard_path = self.dashboards_dir / f"{dashboard_name}.json"
        is_system_dashboard = system_dashboard_path.exists()
        
        # Safety: Never allow overwriting system dashboards via save_as_overlay=False
        # unless explicitly intended (this should be a separate admin function)
        if not save_as_overlay and is_system_dashboard:
            logger.error(f"Attempted to overwrite system dashboard '{dashboard_name}'. Use overlay instead.")
            return False
        
        # Save as user overlay (default behavior)
        if save_as_overlay:
            # Extract only overlay changes (for now, save full dashboard as overlay)
            # In future, could optimize to save only differences
            overlay = dashboard.copy()
            
            success = self.overlay_manager.save_overlay(overlay, dashboard_name)
            
            if success:
                # Clear cache for this dashboard
                if dashboard_name in self._cache:
                    del self._cache[dashboard_name]
                if dashboard_name in self._cache_timestamps:
                    del self._cache_timestamps[dashboard_name]
            
            return success
        
        # Save as system dashboard (only if it doesn't exist)
        else:
            dashboard_path = self.dashboards_dir / f"{dashboard_name}.json"
            
            try:
                # Create backup if file exists
                backup_path = None
                if dashboard_path.exists():
                    backup_path = self.dashboards_dir / f"{dashboard_name}.json.backup"
                    shutil.copy2(dashboard_path, backup_path)
                    logger.info(f"Created backup: {backup_path}")
                
                # Preserve original file permissions if exists
                original_mode = None
                if dashboard_path.exists():
                    original_mode = dashboard_path.stat().st_mode
                
                # Atomic write: write to temp file first, then rename
                temp_path = self.dashboards_dir / f"{dashboard_name}.json.tmp"
                
                with open(temp_path, 'w', encoding='utf-8') as f:
                    json.dump(dashboard, f, indent=2, ensure_ascii=False)
                
                # Restore permissions if original existed
                if original_mode:
                    temp_path.chmod(original_mode)
                else:
                    # Default permissions: rw-r--r--
                    temp_path.chmod(0o644)
                
                # Atomic rename
                temp_path.replace(dashboard_path)
                
                # Clear cache for this dashboard
                if dashboard_name in self._cache:
                    del self._cache[dashboard_name]
                if dashboard_name in self._cache_timestamps:
                    del self._cache_timestamps[dashboard_name]
                
                logger.info(f"Saved system dashboard: {dashboard_name}")
                return True
                
            except Exception as e:
                logger.error(f"Error saving system dashboard {dashboard_name}: {e}", exc_info=True)
                # Restore from backup if write failed
                if backup_path and backup_path.exists() and not dashboard_path.exists():
                    try:
                        backup_path.replace(dashboard_path)
                        logger.info(f"Restored dashboard from backup: {dashboard_name}")
                    except Exception as restore_error:
                        logger.error(f"Failed to restore from backup: {restore_error}")
                return False
    
    def _validate_dashboard_strict(self, dashboard: Dict) -> bool:
        """
        Strictly validate dashboard structure - reject unknown fields.
        
        Args:
            dashboard: Dashboard definition dict
            
        Returns:
            True if valid, False otherwise
        """
        # Required top-level fields
        required_fields = ['name', 'panels']
        if not all(field in dashboard for field in required_fields):
            logger.error("Missing required fields: name, panels")
            return False
        
        # Allowed top-level fields (strict whitelist)
        allowed_fields = {
            'name', 'title', 'description', 'category', 'type', 'panels', 'folder_id'
        }
        for field in dashboard.keys():
            if field not in allowed_fields:
                logger.error(f"Unknown top-level field: {field}")
                return False
        
        # Validate name
        if not isinstance(dashboard['name'], str) or not dashboard['name']:
            logger.error("Dashboard name must be a non-empty string")
            return False
        
        # Validate panels
        if not isinstance(dashboard['panels'], list):
            logger.error("Panels must be a list")
            return False
        
        if len(dashboard['panels']) == 0:
            logger.error("Dashboard must have at least one panel")
            return False
        
        # Validate each panel
        allowed_panel_fields = {
            'id', 'title', 'x', 'y', 'w', 'h', 'refresh_interval', 
            'status', 'content', 'data_source'
        }
        
        for i, panel in enumerate(dashboard['panels']):
            if not isinstance(panel, dict):
                logger.error(f"Panel {i} must be a dict")
                return False
            
            # Required panel fields
            if 'id' not in panel or 'title' not in panel:
                logger.error(f"Panel {i} missing required fields: id, title")
                return False
            
            # Validate panel ID
            if not isinstance(panel['id'], str) or not panel['id']:
                logger.error(f"Panel {i} id must be a non-empty string")
                return False
            
            # Validate panel title
            if not isinstance(panel['title'], str):
                logger.error(f"Panel {i} title must be a string")
                return False
            
            # Check for unknown fields
            for field in panel.keys():
                if field not in allowed_panel_fields:
                    logger.error(f"Panel {i} has unknown field: {field}")
                    return False
            
            # Validate grid positioning (x, y, w, h) - all required
            grid_fields = ['x', 'y', 'w', 'h']
            for field in grid_fields:
                if field not in panel:
                    logger.error(f"Panel {i} missing required grid field: {field}")
                    return False
                
                if not isinstance(panel[field], int):
                    logger.error(f"Panel {i} {field} must be an integer")
                    return False
                
                if panel[field] < 0:
                    logger.error(f"Panel {i} {field} must be >= 0")
                    return False
                
                # Width constraint: max 12 columns
                if field == 'w' and panel[field] > 12:
                    logger.error(f"Panel {i} w must be <= 12")
                    return False
                
                # Width constraint: min 1
                if field == 'w' and panel[field] < 1:
                    logger.error(f"Panel {i} w must be >= 1")
                    return False
                
                # Height constraint: min 1
                if field == 'h' and panel[field] < 1:
                    logger.error(f"Panel {i} h must be >= 1")
                    return False
            
            # Validate refresh_interval if present
            if 'refresh_interval' in panel:
                if not isinstance(panel['refresh_interval'], int) or panel['refresh_interval'] < 0:
                    logger.error(f"Panel {i} refresh_interval must be a non-negative integer")
                    return False
            
            # Validate status if present
            if 'status' in panel:
                if panel['status'] not in ['healthy', 'warning', 'critical', 'info', 'degraded']:
                    logger.error(f"Panel {i} status must be one of: healthy, warning, critical, info, degraded")
                    return False
            
            # Validate data_source if present
            if 'data_source' in panel:
                if not isinstance(panel['data_source'], dict):
                    logger.error(f"Panel {i} data_source must be a dict")
                    return False
                
                # Validate data_source structure
                if 'endpoint' in panel['data_source']:
                    if not isinstance(panel['data_source']['endpoint'], str):
                        logger.error(f"Panel {i} data_source.endpoint must be a string")
                        return False
        
        return True

