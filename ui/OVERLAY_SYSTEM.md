# Dashboard Overlay System

## Overview

The dashboard overlay system allows per-user dashboard customizations without modifying system dashboards. System dashboards remain immutable, while user-specific changes are stored as overlays and merged at load time.

## Architecture

### Components

1. **OverlayManager** (`overlay_manager.py`)
   - Manages user overlay storage
   - Handles overlay loading, saving, and deletion
   - Provides merge logic for combining system + user dashboards
   - Implements audit logging

2. **DashboardEngine** (modified `dashboard_engine.py`)
   - Loads system dashboards
   - Merges user overlays when present
   - Routes saves to overlay storage (never system)

3. **API Endpoints** (modified `server.py`)
   - `GET /api/dashboards/<name>` - Returns merged dashboard
   - `POST /api/dashboards/<name>/save` - Saves as user overlay
   - `GET /api/dashboards/<name>/source` - Returns source info

## User Identity

- User identity is determined via `RANSOMEYE_UI_USER_ID` environment variable
- Defaults to `"system"` if not set
- User ID is sanitized to prevent path traversal attacks

## Storage Structure

```
ui/
├── dashboards/              # System dashboards (immutable)
│   └── system_soc.json
└── user_overlays/           # User overlays
    ├── user1/
    │   └── system_soc.json  # User1's overlay for system_soc
    └── user2/
        └── system_soc.json  # User2's overlay for system_soc
```

## Load Order

1. Load system dashboard from `dashboards/<name>.json`
2. Load user overlay from `user_overlays/<user_id>/<name>.json` (if exists)
3. Merge overlay onto system dashboard
4. Return merged result

## Merge Strategy

### Top-Level Fields
- Overlay fields override system fields (except `name` which must match)
- Supported fields: `title`, `description`, `category`, `type`, `folder_id`
- Deep merge for nested dicts

### Panels
- Panels are matched by `id`
- Overlay panels override system panels with same `id`
- New overlay panels (not in system) are appended
- System panels not in overlay are preserved

## Save Behavior

- **Default**: All saves go to user overlay storage
- **System dashboards**: Never modified via save endpoint
- **Validation**: Strict validation before saving
- **Atomic writes**: Temp file + rename for safety
- **Backups**: Automatic backup before overwrite

## API Endpoints

### GET /api/dashboards/<dashboard_name>
Returns merged dashboard (system + user overlay if present).

**Response:**
```json
{
  "name": "system_soc",
  "title": "My Custom SOC Dashboard",
  "panels": [...]
}
```

### POST /api/dashboards/<dashboard_name>/save
Saves dashboard as user overlay (never modifies system).

**Request:**
```json
{
  "name": "system_soc",
  "title": "My Custom Title",
  "panels": [...]
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Dashboard 'system_soc' saved as user overlay",
  "dashboard": "system_soc",
  "source": "user_overlay"
}
```

### GET /api/dashboards/<dashboard_name>/source
Returns source information.

**Response:**
```json
{
  "source": "merged",      // "system" | "user" | "merged" | "none"
  "has_overlay": true,
  "has_system": true,
  "user_id": "user1"
}
```

## Safety Rules

1. **System dashboards are immutable** - Never overwritten via save endpoint
2. **Fail-closed on corruption** - Invalid overlays are rejected
3. **Fail-soft on missing overlay** - Falls back to system dashboard
4. **Audit logging** - All overlay operations logged to `overlay_audit.log`
5. **Atomic writes** - Temp file + rename prevents partial writes
6. **Automatic backups** - Backup created before overwrite

## Audit Logging

All overlay operations are logged to `ui/overlay_audit.log`:

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "action": "save_overlay",
  "user_id": "user1",
  "dashboard_name": "system_soc",
  "success": true
}
```

## Usage Example

```python
# Set user identity
os.environ['RANSOMEYE_UI_USER_ID'] = 'alice'

# Load dashboard (automatically merges overlay if exists)
dashboard = dashboard_engine.load_dashboard('system_soc')

# Save customizations (saves to overlay, never touches system)
dashboard['title'] = 'Alice\'s Custom Dashboard'
dashboard_engine.save_dashboard(dashboard, 'system_soc')
```

## Future Enhancements

- RBAC integration (currently single-user via env)
- Overlay diff/merge visualization
- Overlay export/import
- Overlay versioning
- Partial overlay storage (only changed fields)

