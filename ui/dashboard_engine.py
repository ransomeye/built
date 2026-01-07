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
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone

logger = logging.getLogger(__name__)


class DashboardEngine:
    """Engine for loading and rendering dashboard definitions from JSON."""
    
    def __init__(self, dashboards_dir: Path):
        """
        Initialize dashboard engine.
        
        Args:
            dashboards_dir: Directory containing JSON dashboard definitions
        """
        self.dashboards_dir = Path(dashboards_dir)
        self.dashboards_dir.mkdir(parents=True, exist_ok=True)
        self._cache: Dict[str, Dict] = {}
        self._cache_timestamps: Dict[str, float] = {}
    
    def load_dashboard(self, dashboard_name: str) -> Optional[Dict[str, Any]]:
        """
        Load a dashboard definition from JSON file.
        
        Args:
            dashboard_name: Name of dashboard (without .json extension)
            
        Returns:
            Dashboard definition dict or None if not found
        """
        dashboard_path = self.dashboards_dir / f"{dashboard_name}.json"
        
        if not dashboard_path.exists():
            logger.warning(f"Dashboard not found: {dashboard_path}")
            return None
        
        try:
            # Check cache timestamp
            mtime = dashboard_path.stat().st_mtime
            cache_key = dashboard_name
            
            if cache_key in self._cache and self._cache_timestamps.get(cache_key) == mtime:
                return self._cache[cache_key]
            
            # Load and parse JSON
            with open(dashboard_path, 'r', encoding='utf-8') as f:
                dashboard = json.load(f)
            
            # Validate dashboard structure
            if not self._validate_dashboard(dashboard):
                logger.error(f"Invalid dashboard structure: {dashboard_name}")
                return None
            
            # Cache dashboard
            self._cache[cache_key] = dashboard
            self._cache_timestamps[cache_key] = mtime
            
            logger.info(f"Loaded dashboard: {dashboard_name}")
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
            dashboards.append(json_file.stem)
        return sorted(dashboards)
    
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
    
    def clear_cache(self):
        """Clear dashboard cache."""
        self._cache.clear()
        self._cache_timestamps.clear()
        logger.info("Dashboard cache cleared")

