# Path and File Name : /home/ransomeye/rebuild/ui/server.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: RansomEye UI Server - SOC-grade schema-safe dashboards

"""
RansomEye UI Server - SOC-Grade Production Dashboard:
- Network-agnostic binding (0.0.0.0 by default)
- Schema-aware queries with fail-soft error handling
- No hardcoded URLs, IPs, or secrets
- Air-gap compatible
- Works across dynamic IPs, DHCP, cloud, on-prem, and air-gapped systems
- SOC semantics: sensor coverage, event flow, security signals, data freshness, integrity
"""

import os
import sys
import json
import logging
import psycopg2
from pathlib import Path
from flask import Flask, jsonify, send_from_directory, render_template, request, Response, send_file
from io import BytesIO
from flask_cors import CORS
from datetime import datetime, timedelta, timezone
from typing import Optional, Dict, Tuple, Any
from schema_helper import SchemaAwareDB
from dashboard_engine import DashboardEngine
from folder_manager import FolderManager
from settings_manager import SettingsManager
from settings import SettingsValidationError
from version_manager import VersionManager
from share_manager import ShareManager
from rate_limiter import get_rate_limiter

# Configure logging - errors logged, not exposed to UI
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stderr)
    ]
)
logger = logging.getLogger(__name__)

# Resolve paths relative to this file location (runtime-safe, portable)
BASE_DIR = Path(__file__).parent.resolve()
TEMPLATE_DIR = BASE_DIR / 'templates'
STATIC_DIR = BASE_DIR / 'static'
DASHBOARDS_DIR = BASE_DIR / 'dashboards'
REBUILD_ROOT = BASE_DIR.parent.resolve()
LOGO_PATH = REBUILD_ROOT / 'core' / 'logo-removebg-preview.png'

# Initialize dashboard engine
dashboard_engine = DashboardEngine(DASHBOARDS_DIR)

# Initialize folder manager
FOLDERS_FILE = BASE_DIR / 'dashboard_folders.json'
folder_manager = FolderManager(FOLDERS_FILE)

# Configuration via environment variables
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = os.environ.get("DB_PORT", "5432")

# UI binding - safe defaults (127.0.0.1 for localhost-only, configurable for network access)
UI_BIND_ADDRESS = os.environ.get("RANSOMEYE_UI_BIND_ADDRESS", "127.0.0.1")
UI_BIND_PORT = int(os.environ.get("RANSOMEYE_UI_BIND_PORT", "8081"))

# Legacy support for old env var names
if "RANSOMEYE_UI_HOST" in os.environ:
    UI_BIND_ADDRESS = os.environ.get("RANSOMEYE_UI_HOST")
if "RANSOMEYE_UI_PORT" in os.environ:
    UI_BIND_PORT = int(os.environ.get("RANSOMEYE_UI_PORT"))

# CORS configuration - explicit origins only (no wildcard)
CORS_ALLOWED_ORIGINS = os.environ.get("RANSOMEYE_UI_ALLOWED_ORIGINS", "").strip()
if CORS_ALLOWED_ORIGINS:
    # Parse comma-separated origins
    CORS_ORIGINS_LIST = [origin.strip() for origin in CORS_ALLOWED_ORIGINS.split(",") if origin.strip()]
else:
    # Default: no CORS (same-origin only)
    CORS_ORIGINS_LIST = []

# CORS credentials (disabled by default for security)
CORS_CREDENTIALS = os.environ.get("RANSOMEYE_UI_CORS_CREDENTIALS", "false").lower() in ("true", "1", "yes", "on")

# Proxy trust configuration (disabled by default)
TRUST_PROXY = os.environ.get("RANSOMEYE_UI_TRUST_PROXY", "false").lower() in ("true", "1", "yes", "on")

# Air-gap mode detection (if no internet connectivity expected)
AIR_GAP_MODE = os.environ.get("RANSOMEYE_AIR_GAP", "false").lower() == "true"

# Flask app
app = Flask(__name__, static_folder=str(STATIC_DIR), template_folder=str(TEMPLATE_DIR))

# Configure CORS with explicit origins
if CORS_ORIGINS_LIST:
    CORS(app, origins=CORS_ORIGINS_LIST, supports_credentials=CORS_CREDENTIALS, methods=["GET", "HEAD"])
else:
    # No CORS if no origins specified (same-origin only)
    CORS(app, resources={r"/*": {"origins": []}}, supports_credentials=False, methods=["GET", "HEAD"])

# Configure proxy trust
app.config['PROXY_FIX'] = TRUST_PROXY


def validate_bind_address(address: str) -> bool:
    """
    Validate bind address for security.
    
    Allowed values:
    - 127.0.0.1 (localhost only - safest)
    - 0.0.0.0 (all interfaces - use with caution)
    - Specific interface IP (e.g., 192.168.1.100)
    
    Returns:
        True if valid, False otherwise
    """
    if address == "127.0.0.1" or address == "localhost":
        return True
    if address == "0.0.0.0":
        return True
    
    # Validate IP address format
    try:
        parts = address.split(".")
        if len(parts) != 4:
            return False
        for part in parts:
            num = int(part)
            if num < 0 or num > 255:
                return False
        return True
    except (ValueError, AttributeError):
        return False


def get_client_ip():
    """
    Get client IP address, handling forwarded headers if proxy trust is enabled.
    
    Returns:
        Client IP address string
    """
    if TRUST_PROXY:
        # Trust X-Forwarded-For header (use first IP in chain)
        forwarded_for = request.headers.get('X-Forwarded-For')
        if forwarded_for:
            # X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
            return forwarded_for.split(',')[0].strip()
    
    # Default: use direct connection IP
    return request.remote_addr or "unknown"


# Security headers middleware
@app.after_request
def set_security_headers(response):
    """Set security headers on all responses."""
    # X-Frame-Options: prevent clickjacking
    frame_options = os.environ.get("RANSOMEYE_UI_X_FRAME_OPTIONS", "DENY")
    if frame_options.upper() in ("DENY", "SAMEORIGIN"):
        response.headers['X-Frame-Options'] = frame_options.upper()
    
    # X-Content-Type-Options: prevent MIME sniffing
    response.headers['X-Content-Type-Options'] = 'nosniff'
    
    # Referrer-Policy: control referrer information
    referrer_policy = os.environ.get("RANSOMEYE_UI_REFERRER_POLICY", "strict-origin-when-cross-origin")
    response.headers['Referrer-Policy'] = referrer_policy
    
    # Content-Security-Policy: safe default that doesn't break existing UI
    csp = os.environ.get("RANSOMEYE_UI_CSP", 
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self';")
    response.headers['Content-Security-Policy'] = csp
    
    return response


# Method enforcement middleware
@app.before_request
def enforce_methods():
    """Enforce HTTP method restrictions for security."""
    # Log client IP for security monitoring
    client_ip = get_client_ip()
    
    # Reject unsupported methods on API endpoints
    if request.path.startswith('/api/') and request.method not in ['GET', 'HEAD', 'POST', 'DELETE']:
        logger.warning(f"Method {request.method} not allowed on {request.path} from {client_ip}")
        return jsonify({
            "error": "Method not allowed",
            "status": "error"
        }), 405
    
    # Log state-changing operations for audit (actual method enforcement handled by route decorators)
    state_changing_keywords = ['import', 'create', 'save', 'share', 'settings', 'revoke', 'cleanup', 'rotate', 'delete']
    if any(keyword in request.path for keyword in state_changing_keywords):
        if request.method == 'GET':
            logger.warning(f"GET method attempted on potentially state-changing endpoint {request.path} from {client_ip}")
        else:
            logger.info(f"State-changing operation: {request.method} {request.path} from {client_ip}")


# Error handlers for clean error pages
@app.errorhandler(404)
def not_found(error):
    """Handle 404 errors with clean error page."""
    if request.path.startswith('/api/'):
        return jsonify({
            "error": "Resource not found",
            "status": "error"
        }), 404
    return render_template('error.html', 
                         error_code=404, 
                         error_message="Page not found"), 404


@app.errorhandler(405)
def method_not_allowed(error):
    """Handle 405 errors."""
    if request.path.startswith('/api/'):
        return jsonify({
            "error": "Method not allowed",
            "status": "error"
        }), 405
    return render_template('error.html', 
                         error_code=405, 
                         error_message="Method not allowed"), 405


@app.errorhandler(500)
def internal_error(error):
    """Handle 500 errors with clean error page (no stack traces)."""
    logger.error(f"Internal server error: {error}", exc_info=True)
    if request.path.startswith('/api/'):
        return jsonify({
            "error": "Internal server error",
            "status": "error"
        }), 500
    return render_template('error.html', 
                         error_code=500, 
                         error_message="Internal server error"), 500


@app.errorhandler(400)
def bad_request(error):
    """Handle 400 errors."""
    if request.path.startswith('/api/'):
        return jsonify({
            "error": "Bad request",
            "status": "error"
        }), 400
    return render_template('error.html', 
                         error_code=400, 
                         error_message="Bad request"), 400


def get_db_connection():
    """Get database connection."""
    try:
        return psycopg2.connect(
            host=DB_HOST, port=DB_PORT, database=DB_NAME,
            user=DB_USER, password=DB_PASSWORD
        )
    except Exception as e:
        logger.error(f"Database connection error: {e}", exc_info=True)
        return None


def safe_get_metric(db: SchemaAwareDB, query: str, params=None, default="Metric unavailable"):
    """Safely execute a query and return metric or default."""
    try:
        result = db.safe_query(query, params, default=None)
        return result if result is not None else default
    except Exception as e:
        logger.error(f"Metric query failed: {e}", exc_info=True)
        return default


def get_share_policy():
    """
    Get share policy configuration from environment variables.
    
    Returns:
        Dict with:
        - enabled: bool (True if sharing is enabled)
        - default_expiry_days: int or None (default expiry in days)
        - max_expiry_days: int or None (maximum allowed expiry in days)
    """
    # Check if sharing is enabled (default: True if not set)
    share_enabled_str = os.environ.get('RANSOMEYE_SHARE_ENABLED', 'true').lower()
    share_enabled = share_enabled_str in ('true', '1', 'yes', 'on')
    
    # Get default expiry (default: None if not set)
    default_expiry_days = None
    default_expiry_str = os.environ.get('RANSOMEYE_SHARE_DEFAULT_EXPIRY_DAYS')
    if default_expiry_str:
        try:
            default_expiry_days = int(default_expiry_str)
            if default_expiry_days <= 0:
                default_expiry_days = None
        except (ValueError, TypeError):
            default_expiry_days = None
    
    # Get max expiry (default: None if not set, meaning no limit)
    max_expiry_days = None
    max_expiry_str = os.environ.get('RANSOMEYE_SHARE_MAX_EXPIRY_DAYS')
    if max_expiry_str:
        try:
            max_expiry_days = int(max_expiry_str)
            if max_expiry_days <= 0:
                max_expiry_days = None
        except (ValueError, TypeError):
            max_expiry_days = None
    
    return {
        'enabled': share_enabled,
        'default_expiry_days': default_expiry_days,
        'max_expiry_days': max_expiry_days
    }


def validate_expiry_days(expires_in_days: Optional[int], policy: Dict[str, Any]) -> Tuple[Optional[int], Optional[str]]:
    """
    Validate and enforce expiry days against policy.
    
    Args:
        expires_in_days: Requested expiry in days (None = use default)
        policy: Policy dict from get_share_policy()
        
    Returns:
        Tuple of (validated_expiry_days, error_message)
        - If error_message is not None, expiry is invalid
        - If expires_in_days is None, applies default if available
    """
    # If no expiry specified, use default
    if expires_in_days is None:
        expires_in_days = policy.get('default_expiry_days')
    
    # If still None, no expiry (allowed)
    if expires_in_days is None:
        return None, None
    
    # Validate max expiry limit
    max_expiry = policy.get('max_expiry_days')
    if max_expiry is not None and expires_in_days > max_expiry:
        return None, f"Expiry exceeds maximum allowed ({max_expiry} days)"
    
    # Must be positive
    if expires_in_days <= 0:
        return None, "Expiry must be a positive number of days"
    
    return expires_in_days, None


@app.route('/')
def index():
    """Serve main SOC dashboard page."""
    return render_template('index.html')


@app.route('/dashboard/<dashboard_name>')
def dashboard_view(dashboard_name: str):
    """Serve a dashboard by name (loads from JSON definition)."""
    dashboard = dashboard_engine.load_dashboard(dashboard_name)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    return render_template('dashboard.html', dashboard=dashboard)


@app.route('/api/dashboards')
def list_dashboards():
    """
    List all available dashboards with folder assignments.
    Filters out dashboards that should be hidden (e.g., no data available).
    
    Returns:
        JSON with dashboards array (names) and dashboards_with_folders array (name + folder_id)
    """
    dashboard_names = dashboard_engine.list_dashboards()
    dashboards_with_folders = dashboard_engine.list_dashboards_with_folders()
    
    # Check visibility for dashboards that require data
    visible_dashboards = []
    visible_dashboards_with_folders = []
    
    for dash in dashboards_with_folders:
        dashboard_name = dash.get('name')
        
        # Check visibility for Linux Agent Health dashboard
        if dashboard_name == 'linux_agent_health':
            if not _check_linux_agent_health_visibility():
                continue  # Skip this dashboard - no data available
        
        # Check visibility for Sensor Coverage dashboard
        if dashboard_name == 'sensor_coverage':
            if not _check_sensor_coverage_visibility():
                continue  # Skip this dashboard - no data available
        
        visible_dashboards_with_folders.append(dash)
    
    # Filter dashboard_names to match visible dashboards
    visible_dashboard_names = [d['name'] for d in visible_dashboards_with_folders]
    
    return jsonify({
        "dashboards": visible_dashboard_names,
        "dashboards_with_folders": visible_dashboards_with_folders,
        "count": len(visible_dashboard_names)
    })


def _check_linux_agent_health_visibility() -> bool:
    """
    Check if Linux Agent Health dashboard should be visible.
    Returns True if data exists, False otherwise.
    
    Uses EXACT same SQL predicate as main endpoint (no simplified COUNT query).
    """
    conn = get_db_connection()
    if not conn:
        return False
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Check if table exists
        if not db.table_exists("ransomeye", "linux_agent_telemetry"):
            cursor.close()
            conn.close()
            return False
        
        # Check if required columns exist
        required_cols = ["agent_id", "observed_at", "payload"]
        for col in required_cols:
            if not db.column_exists("ransomeye", "linux_agent_telemetry", col):
                cursor.close()
                conn.close()
                return False
        
        # Use EXACT same SQL predicate as main endpoint
        # Reuse the same WITH latest CTE and check if it returns any rows
        visibility_query = """
            WITH latest AS (
              SELECT DISTINCT ON (agent_id)
                agent_id,
                observed_at,
                payload->'system' AS system
              FROM linux_agent_telemetry
              WHERE payload ? 'system'
                AND payload->'system' != '{}'::jsonb
                AND observed_at > NOW() - INTERVAL '10 minutes'
              ORDER BY agent_id, observed_at DESC
            )
            SELECT COUNT(*)
            FROM latest
        """
        
        count = db.safe_query(visibility_query, default=0) or 0
        
        cursor.close()
        conn.close()
        
        return count > 0
    
    except Exception as e:
        logger.error(f"Error checking Linux Agent Health visibility: {e}", exc_info=True)
        if conn:
            try:
                cursor.close()
                conn.close()
            except:
                pass
        return False


def _check_sensor_coverage_visibility() -> bool:
    """
    Check if Sensor Coverage dashboard should be visible.
    Returns True if data exists, False otherwise.
    
    Uses EXACT same SQL predicate as main endpoint (no simplified COUNT query).
    """
    conn = get_db_connection()
    if not conn:
        return False
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Check if table exists
        if not db.table_exists("ransomeye", "linux_agent_telemetry"):
            cursor.close()
            conn.close()
            return False
        
        # Check if required columns exist
        required_cols = ["agent_id", "observed_at", "payload"]
        for col in required_cols:
            if not db.column_exists("ransomeye", "linux_agent_telemetry", col):
                cursor.close()
                conn.close()
                return False
        
        # Use EXACT same SQL predicate as main endpoint
        visibility_query = """
            WITH recent AS (
              SELECT
                agent_id,
                payload
              FROM linux_agent_telemetry
              WHERE observed_at > NOW() - INTERVAL '10 minutes'
            )
            SELECT COUNT(*)
            FROM (
              SELECT
                agent_id,
                BOOL_OR(payload->>'event_category' = 'process')        AS process_sensor,
                BOOL_OR(payload ? 'filesystem_data' AND payload->'filesystem_data' != 'null'::jsonb) AS filesystem_sensor,
                BOOL_OR(payload ? 'network_data' AND payload->'network_data' != 'null'::jsonb) AS network_sensor,
                BOOL_OR(payload ? 'system' AND payload->'system' != '{}'::jsonb) AS system_sensor,
                BOOL_OR(payload->>'event_category' = 'deception')      AS deception_sensor
              FROM recent
              GROUP BY agent_id
            ) AS coverage
        """
        
        count = db.safe_query(visibility_query, default=0) or 0
        
        cursor.close()
        conn.close()
        
        return count > 0
    
    except Exception as e:
        logger.error(f"Error checking Sensor Coverage visibility: {e}", exc_info=True)
        if conn:
            try:
                cursor.close()
                conn.close()
            except:
                pass
        return False


@app.route('/api/dashboard-folders', methods=['GET'])
def list_dashboard_folders():
    """
    List all dashboard folders (system-scoped).
    
    Returns:
        JSON array of folder objects with id, name, description, order
    """
    try:
        folders = folder_manager.list_folders()
        return jsonify(folders)
    except Exception as e:
        logger.error(f"Error listing folders: {e}", exc_info=True)
        return jsonify({"error": "Failed to list folders"}), 500


@app.route('/api/dashboard-folders', methods=['POST'])
def create_dashboard_folder():
    """
    Create a new dashboard folder (system-scoped, strict validation).
    
    Request body (JSON):
        {
            "id": "folder-id",  # Required, slug-safe
            "name": "Folder Name",  # Required
            "description": "Optional description",  # Optional
            "order": 10  # Optional, integer for sorting
        }
    
    Rules:
    - Strict validation (reject unknown fields)
    - Fail-closed on corruption
    - Audit-log all changes
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        folder_data = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in folder creation request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not folder_data:
        return jsonify({"error": "Folder data is required"}), 400
    
    # Reject unknown fields (strict whitelist)
    allowed_fields = {'id', 'name', 'description', 'order'}
    for field in folder_data.keys():
        if field not in allowed_fields:
            logger.warning(f"Rejecting unknown field in folder creation: {field}")
            return jsonify({"error": f"Unknown field: {field}"}), 400
    
    # Create folder
    success, error_msg = folder_manager.create_folder(folder_data)
    
    if success:
        folder = folder_manager.get_folder(folder_data['id'])
        return jsonify({
            "status": "success",
            "message": f"Folder '{folder_data['id']}' created successfully",
            "folder": folder
        }), 201
    else:
        return jsonify({"error": error_msg or "Failed to create folder"}), 400


@app.route('/api/dashboards/<dashboard_name>/move', methods=['POST'])
def move_dashboard(dashboard_name: str):
    """
    Move a dashboard to a different folder.
    
    Request body (JSON):
        {
            "folder_id": "target-folder-id"  # Required
        }
    
    Rules:
    - Validate folder exists
    - Fail-closed on invalid folder_id
    - Audit-log all moves
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        data = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in move request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not data or 'folder_id' not in data:
        return jsonify({"error": "folder_id is required"}), 400
    
    folder_id = data['folder_id']
    
    # Validate folder exists
    folder = folder_manager.get_folder(folder_id)
    if not folder:
        return jsonify({"error": f"Folder '{folder_id}' not found"}), 404
    
    # Load dashboard
    dashboard = dashboard_engine.load_dashboard(dashboard_name)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    # Update folder_id
    dashboard['folder_id'] = folder_id
    
    # Save dashboard
    if dashboard_engine.save_dashboard(dashboard, dashboard_name):
        logger.info(f"Moved dashboard '{dashboard_name}' to folder '{folder_id}'")
        return jsonify({
            "status": "success",
            "message": f"Dashboard '{dashboard_name}' moved to folder '{folder_id}'",
            "dashboard": dashboard_name,
            "folder_id": folder_id
        })
    else:
        return jsonify({"error": "Failed to save dashboard after move"}), 500


@app.route('/api/dashboards/<dashboard_name>')
def get_dashboard_definition(dashboard_name: str):
    """
    Get dashboard JSON definition (with user overlay merged if present).
    
    Returns:
        Dashboard JSON with user overlay applied (if exists)
    """
    dashboard = dashboard_engine.load_dashboard(dashboard_name, include_overlay=True)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    return jsonify(dashboard)


@app.route('/api/dashboards/<dashboard_name>/source', methods=['GET'])
def get_dashboard_source(dashboard_name: str):
    """
    Get dashboard source information (system vs user overlay).
    
    Returns:
        JSON with source information:
        {
            "source": "system" | "user" | "merged" | "none",
            "has_overlay": bool,
            "has_system": bool,
            "user_id": str | null
        }
    """
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    return jsonify(source_info)


@app.route('/api/dashboards/<dashboard_name>/export', methods=['GET'])
def export_dashboard(dashboard_name: str):
    """
    Export dashboard as validated JSON for backup, migration, and audit purposes.
    
    Rules:
    - Export allowed for system and personal dashboards
    - Personal dashboards export merged view (system + personal overlay)
    - System dashboards export system JSON only
    - Strict schema validation before export (fail-closed on corruption)
    - Audit-log export event
    - Returns JSON file download with proper filename
    
    Returns:
        JSON file download with dashboard definition including:
        - dashboard metadata (name, title, description, etc.)
        - panels array
        - layout (grid positioning)
        - folder_id
        - source info (system/personal/merged)
        - export metadata (timestamp, exported_by)
    """
    # Validate dashboard exists
    dashboard = dashboard_engine.load_dashboard(dashboard_name, include_overlay=True)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    # Get source information
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Get user ID for audit logging
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Strict schema validation (fail-closed on corruption)
    if not dashboard_engine._validate_dashboard_strict(dashboard):
        logger.error(f"Dashboard '{dashboard_name}' failed validation - cannot export corrupted dashboard")
        dashboard_engine.overlay_manager._audit_log(
            'export_dashboard', user_id, dashboard_name,
            success=False, error="validation_failed"
        )
        return jsonify({"error": "Dashboard validation failed - dashboard is corrupted"}), 400
    
    # Build export JSON with metadata
    export_data = {
        # Dashboard definition
        "name": dashboard.get('name'),
        "title": dashboard.get('title', ''),
        "description": dashboard.get('description', ''),
        "category": dashboard.get('category', ''),
        "type": dashboard.get('type', ''),
        "folder_id": dashboard.get('folder_id', 'general'),
        "panels": dashboard.get('panels', []),
        
        # Source information
        "export_metadata": {
            "source": source_info.get('source', 'unknown'),
            "has_overlay": source_info.get('has_overlay', False),
            "has_system": source_info.get('has_system', False),
            "user_id": source_info.get('user_id'),
            "exported_at": datetime.now(timezone.utc).isoformat(),
            "exported_by": user_id
        }
    }
    
    # Audit log export event
    dashboard_engine.overlay_manager._audit_log(
        'export_dashboard', user_id, dashboard_name,
        success=True,
        error=f"source:{source_info.get('source', 'unknown')}"
    )
    
    logger.info(f"Exported dashboard '{dashboard_name}' (source: {source_info.get('source')}, user: {user_id})")
    
    # Return JSON response with proper headers for file download
    response = Response(
        json.dumps(export_data, indent=2, ensure_ascii=False),
        mimetype='application/json',
        headers={
            'Content-Disposition': f'attachment; filename=ransomeye-dashboard-{dashboard_name}.json'
        }
    )
    return response


@app.route('/api/dashboards/import', methods=['POST'])
def import_dashboard():
    """
    Import dashboard from exported JSON file.
    
    Request (multipart/form-data):
        - file: JSON file (required)
        - new_name: Optional new dashboard name (slug-safe, defaults to imported name)
        - new_title: Optional new dashboard title (defaults to imported title)
    
    Rules:
    - Import creates a NEW personal dashboard overlay only
    - System dashboards are NEVER overwritten
    - Validate JSON strictly against dashboard schema
    - Reject unknown fields
    - Reject corrupted or non-RansomEye exports
    - Validate name uniqueness (system + user overlays)
    - Strip export_metadata and system-only fields
    - Fail-closed on any validation error
    - Audit-log import event with file hash
    """
    import hashlib
    
    # Get user ID for audit logging
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Check if file is present
    if 'file' not in request.files:
        logger.error("Import request missing file")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, 'unknown',
            success=False, error="missing_file"
        )
        return jsonify({"error": "File is required"}), 400
    
    file = request.files['file']
    
    # Validate file extension
    if not file.filename or not file.filename.lower().endswith('.json'):
        logger.error(f"Invalid file type: {file.filename}")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, 'unknown',
            success=False, error=f"invalid_file_type:{file.filename}"
        )
        return jsonify({"error": "File must be a JSON file (.json)"}), 400
    
    # Read and parse JSON
    try:
        file_content = file.read()
        file_hash = hashlib.sha256(file_content).hexdigest()
        
        # Parse JSON
        import_data = json.loads(file_content.decode('utf-8'))
    except json.JSONDecodeError as e:
        logger.error(f"Invalid JSON in import file: {e}")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, 'unknown',
            success=False, error=f"json_decode_error:{str(e)}"
        )
        return jsonify({"error": f"Invalid JSON: {str(e)}"}), 400
    except Exception as e:
        logger.error(f"Error reading import file: {e}", exc_info=True)
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, 'unknown',
            success=False, error=f"file_read_error:{str(e)}"
        )
        return jsonify({"error": f"Error reading file: {str(e)}"}), 400
    
    # Extract optional parameters
    new_name = request.form.get('new_name', '').strip()
    new_title = request.form.get('new_title', '').strip()
    
    # Validate and extract dashboard data
    # Strip export_metadata if present (it's not part of dashboard schema)
    if 'export_metadata' in import_data:
        export_metadata = import_data.pop('export_metadata')
        logger.debug(f"Stripped export_metadata: {export_metadata}")
    
    # Get dashboard name (from import or new_name)
    dashboard_name = new_name if new_name else import_data.get('name')
    if not dashboard_name:
        logger.error("Dashboard name is required (not found in import or new_name)")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, 'unknown',
            success=False, error="missing_dashboard_name"
        )
        return jsonify({"error": "Dashboard name is required"}), 400
    
    # Validate dashboard name (slug-safe)
    import re
    if not re.match(r'^[a-z0-9_-]+$', dashboard_name.lower()):
        logger.error(f"Invalid dashboard name format: {dashboard_name}")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, dashboard_name,
            success=False, error="invalid_name_format"
        )
        return jsonify({"error": "Dashboard name must be slug-safe (lowercase letters, numbers, hyphens, underscores)"}), 400
    
    # Check for system dashboard collision (fail-closed)
    system_dashboard_path = DASHBOARDS_DIR / f"{dashboard_name}.json"
    if system_dashboard_path.exists():
        logger.error(f"Dashboard name '{dashboard_name}' conflicts with a system dashboard")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, dashboard_name,
            success=False, error="system_dashboard_collision"
        )
        return jsonify({"error": f"Dashboard name '{dashboard_name}' conflicts with a system dashboard"}), 400
    
    # Check for existing user overlay collision (fail-closed)
    if dashboard_engine.overlay_manager.has_overlay(dashboard_name, user_id):
        logger.error(f"Dashboard '{dashboard_name}' already exists for user '{user_id}'")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, dashboard_name,
            success=False, error="user_overlay_collision"
        )
        return jsonify({"error": f"Dashboard '{dashboard_name}' already exists"}), 400
    
    # Update dashboard name and title
    import_data['name'] = dashboard_name
    if new_title:
        import_data['title'] = new_title
    elif 'title' not in import_data:
        import_data['title'] = dashboard_name.replace('-', ' ').replace('_', ' ').title()
    
    # Validate dashboard structure strictly (fail-closed on validation error)
    if not dashboard_engine._validate_dashboard_strict(import_data):
        logger.error(f"Imported dashboard '{dashboard_name}' failed strict validation")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, dashboard_name,
            success=False, error="validation_failed"
        )
        return jsonify({"error": "Dashboard validation failed - check schema compliance"}), 400
    
    # Save as personal dashboard overlay (atomic write with backup, version capture with 'import' action)
    success = dashboard_engine.overlay_manager.save_overlay(import_data, dashboard_name, user_id, version_action='import')
    
    if not success:
        logger.error(f"Failed to save imported dashboard '{dashboard_name}'")
        dashboard_engine.overlay_manager._audit_log(
            'import_dashboard', user_id, dashboard_name,
            success=False, error="save_failed"
        )
        return jsonify({"error": "Failed to save imported dashboard"}), 500
    
    # Clear cache for this dashboard
    dashboard_engine.clear_cache()
    
    # Audit log success
    dashboard_engine.overlay_manager._audit_log(
        'import_dashboard', user_id, dashboard_name,
        success=True,
        error=f"file_hash:{file_hash[:16]}"
    )
    
    logger.info(f"Imported dashboard '{dashboard_name}' (user: {user_id}, file_hash: {file_hash[:16]})")
    
    return jsonify({
        "success": True,
        "dashboard_name": dashboard_name,
        "title": import_data.get('title', ''),
        "message": f"Dashboard '{dashboard_name}' imported successfully"
    }), 200


@app.route('/api/dashboards/<dashboard_name>/duplicate', methods=['POST'])
def duplicate_dashboard(dashboard_name: str):
    """
    Duplicate a personal dashboard into a new personal dashboard.
    
    Request body (JSON):
        {
            "new_name": "dashboard-name-copy",  # Required, slug-safe
            "new_title": "Dashboard Title (Copy)"  # Required, 1-200 characters
        }
    
    Rules:
    - Only personal dashboards can be duplicated
    - System dashboards cannot be duplicated (fail-closed)
    - Source dashboard remains unchanged
    - New dashboard must have unique name (system + user overlays)
    - Fail-closed on collisions or invalid input
    - Audit-log duplication event
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        data = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in duplicate request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not data:
        return jsonify({"error": "Request body is required"}), 400
    
    if 'new_name' not in data or not data['new_name']:
        return jsonify({"error": "new_name is required"}), 400
    
    if 'new_title' not in data or not data['new_title']:
        return jsonify({"error": "new_title is required"}), 400
    
    new_name = data['new_name'].strip()
    new_title = data['new_title'].strip()
    
    # Validate name (slug-safe: alphanumeric, hyphens, underscores only)
    import re
    if not re.match(r'^[a-zA-Z0-9_-]+$', new_name):
        return jsonify({
            "error": "Dashboard name must contain only letters, numbers, hyphens, and underscores"
        }), 400
    
    if len(new_name) < 1 or len(new_name) > 100:
        return jsonify({"error": "Dashboard name must be between 1 and 100 characters"}), 400
    
    if len(new_title) < 1 or len(new_title) > 200:
        return jsonify({"error": "Dashboard title must be between 1 and 200 characters"}), 400
    
    # Check if source dashboard is a personal dashboard (has overlay)
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can be duplicated
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot be duplicated"
        }), 403
    
    # Check for collision with system dashboard
    system_dashboard_path = DASHBOARDS_DIR / f"{new_name}.json"
    if system_dashboard_path.exists():
        return jsonify({
            "error": f"Dashboard name '{new_name}' conflicts with a system dashboard"
        }), 409
    
    # Check for collision with existing user overlay
    if dashboard_engine.overlay_manager.has_overlay(new_name, user_id):
        return jsonify({
            "error": f"Dashboard '{new_name}' already exists"
        }), 409
    
    # Duplicate dashboard
    success = dashboard_engine.overlay_manager.duplicate_dashboard(
        source_dashboard_name=dashboard_name,
        new_dashboard_name=new_name,
        new_title=new_title,
        user_id=user_id
    )
    
    if not success:
        return jsonify({
            "error": "Failed to duplicate dashboard (internal error)"
        }), 500
    
    logger.info(f"Duplicated personal dashboard '{dashboard_name}' to '{new_name}' (user: {user_id})")
    
    return jsonify({
        "status": "success",
        "message": f"Dashboard '{dashboard_name}' duplicated to '{new_name}' successfully",
        "dashboard": new_name,
        "source": "user_overlay"
    }), 201


@app.route('/api/dashboards/<dashboard_name>/rename', methods=['POST'])
def rename_dashboard(dashboard_name: str):
    """
    Rename a personal dashboard (title only, slug remains unchanged).
    
    Request body (JSON):
        {
            "new_title": "New Dashboard Title"  # Required, 1-200 characters
        }
    
    Rules:
    - Only personal dashboards can be renamed
    - System dashboards cannot be renamed (fail-closed)
    - Dashboard name (slug) remains immutable
    - Only title is updated
    - Audit-log rename event
    - Fail-closed on invalid input
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        data = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in rename request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not data or 'new_title' not in data:
        return jsonify({"error": "new_title is required"}), 400
    
    new_title = data['new_title'].strip()
    
    # Validate title length
    if len(new_title) < 1 or len(new_title) > 200:
        return jsonify({"error": "Title must be between 1 and 200 characters"}), 400
    
    # Check if dashboard is a personal dashboard (has overlay)
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can be renamed
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot be renamed"
        }), 403
    
    # Load current dashboard (will include overlay)
    dashboard = dashboard_engine.load_dashboard(dashboard_name, include_overlay=True)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    # Update title only
    dashboard['title'] = new_title
    
    # Save as overlay (only updates overlay, never system) with 'rename' action for version capture
    if dashboard_engine.save_dashboard(dashboard, dashboard_name, save_as_overlay=True, version_action='rename'):
        logger.info(f"Renamed personal dashboard '{dashboard_name}' to '{new_title}' (user: {user_id})")
        
        # Audit log via overlay manager
        dashboard_engine.overlay_manager._audit_log(
            'rename_dashboard', user_id, dashboard_name, 
            success=True, error=f"new_title:{new_title}"
        )
        
        return jsonify({
            "status": "success",
            "message": f"Dashboard '{dashboard_name}' renamed successfully",
            "dashboard": dashboard_name,
            "new_title": new_title
        })
    else:
        return jsonify({"error": "Failed to save dashboard after rename"}), 500


@app.route('/api/dashboards/create', methods=['POST'])
def create_dashboard():
    """
    Create a new personal dashboard.
    
    Request body (JSON):
        {
            "name": "dashboard-name",  # Required, slug-safe
            "title": "Dashboard Title",  # Required
            "source_dashboard": "system_soc"  # Optional, system dashboard to use as template
        }
    
    Rules:
    - Validate name (slug-safe, non-empty)
    - Check for collisions (fail-closed)
    - Create from template or blank
    - Audit-log creation
    - Fail-closed on invalid input
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        data = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in create request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not data:
        return jsonify({"error": "Request body is required"}), 400
    
    # Validate required fields
    if 'name' not in data or not data['name']:
        return jsonify({"error": "Dashboard name is required"}), 400
    
    if 'title' not in data or not data['title']:
        return jsonify({"error": "Dashboard title is required"}), 400
    
    dashboard_name = data['name'].strip()
    title = data['title'].strip()
    source_dashboard = data.get('source_dashboard', '').strip() or None
    
    # Validate name (slug-safe: alphanumeric, hyphens, underscores only)
    import re
    if not re.match(r'^[a-zA-Z0-9_-]+$', dashboard_name):
        return jsonify({
            "error": "Dashboard name must contain only letters, numbers, hyphens, and underscores"
        }), 400
    
    if len(dashboard_name) < 1 or len(dashboard_name) > 100:
        return jsonify({"error": "Dashboard name must be between 1 and 100 characters"}), 400
    
    if len(title) < 1 or len(title) > 200:
        return jsonify({"error": "Dashboard title must be between 1 and 200 characters"}), 400
    
    # Check for collision with system dashboard
    system_dashboard_path = DASHBOARDS_DIR / f"{dashboard_name}.json"
    if system_dashboard_path.exists():
        return jsonify({
            "error": f"Dashboard name '{dashboard_name}' conflicts with a system dashboard"
        }), 409
    
    # Check for collision with existing user overlay
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    if dashboard_engine.overlay_manager.has_overlay(dashboard_name, user_id):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' already exists"
        }), 409
    
    # Validate source dashboard if provided
    if source_dashboard:
        source_path = DASHBOARDS_DIR / f"{source_dashboard}.json"
        if not source_path.exists():
            return jsonify({
                "error": f"Source dashboard '{source_dashboard}' not found"
            }), 404
    
    # Create dashboard
    success = dashboard_engine.overlay_manager.create_personal_dashboard(
        dashboard_name=dashboard_name,
        title=title,
        source_dashboard=source_dashboard,
        user_id=user_id
    )
    
    if not success:
        return jsonify({
            "error": "Failed to create dashboard (internal error)"
        }), 500
    
    logger.info(f"Created personal dashboard '{dashboard_name}' (user: {user_id}, source: {source_dashboard or 'blank'})")
    
    return jsonify({
        "status": "success",
        "message": f"Dashboard '{dashboard_name}' created successfully",
        "dashboard": dashboard_name,
        "source": "user_overlay"
    }), 201


@app.route('/api/dashboards/<dashboard_name>', methods=['DELETE'])
def delete_dashboard(dashboard_name: str):
    """
    Delete a personal dashboard.
    
    Rules:
    - Only personal dashboards can be deleted (must have overlay)
    - System dashboards cannot be deleted (fail-closed, 403)
    - Dashboard must exist and be personal
    - Atomic delete with backup (.deleted backup)
    - Audit-log deletion event
    - Fail-closed on any validation error
    """
    # Validate dashboard exists
    dashboard = dashboard_engine.load_dashboard(dashboard_name)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    # Get dashboard source information
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can be deleted
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot be deleted"
        }), 403
    
    # Validate dashboard is personal-only (not merged with system)
    # For merged dashboards, we delete the overlay (restores to system)
    # For user-only dashboards, we delete the entire dashboard
    if source_info.get('source') == 'user':
        # Personal-only dashboard - delete overlay
        success = dashboard_engine.overlay_manager.delete_overlay(dashboard_name, user_id)
        
        if not success:
            return jsonify({
                "error": "Failed to delete dashboard (internal error)"
            }), 500
        
        logger.info(f"Deleted personal dashboard '{dashboard_name}' (user: {user_id})")
        
        return jsonify({
            "status": "success",
            "message": f"Dashboard '{dashboard_name}' deleted successfully",
            "dashboard": dashboard_name
        })
    elif source_info.get('source') == 'merged':
        # Merged dashboard - delete overlay (restores to system)
        success = dashboard_engine.overlay_manager.delete_overlay(dashboard_name, user_id)
        
        if not success:
            return jsonify({
                "error": "Failed to delete dashboard overlay (internal error)"
            }), 500
        
        logger.info(f"Deleted overlay for merged dashboard '{dashboard_name}' (user: {user_id}) - restored to system")
        
        return jsonify({
            "status": "success",
            "message": f"Personal customizations for '{dashboard_name}' deleted - restored to system dashboard",
            "dashboard": dashboard_name,
            "restored_to_system": True
        })
    else:
        # Unexpected state - fail-closed
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is in an unexpected state and cannot be deleted"
        }), 500


@app.route('/api/dashboards/<dashboard_name>/panels')
def get_dashboard_panels(dashboard_name: str):
    """Get dashboard panels with refresh intervals."""
    dashboard = dashboard_engine.load_dashboard(dashboard_name)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    refresh_intervals = dashboard_engine.get_panel_refresh_intervals(dashboard)
    panels_info = []
    
    for panel in dashboard.get('panels', []):
        panels_info.append({
            "id": panel['id'],
            "title": panel.get('title', ''),
            "refresh_interval": panel.get('refresh_interval'),
            "x": panel.get('x', 0),
            "y": panel.get('y', 0),
            "w": panel.get('w', 12),
            "h": panel.get('h', 1)
        })
    
    return jsonify({
        "dashboard": dashboard_name,
        "panels": panels_info,
        "refresh_intervals": refresh_intervals
    })


@app.route('/api/dashboards/<dashboard_name>/save', methods=['POST'])
def save_dashboard(dashboard_name: str):
    """
    Save dashboard definition as user overlay (never modifies system dashboards).
    
    Rules:
    - Always saves as user overlay (never overwrites system dashboards)
    - Validate incoming JSON strictly
    - Reject unknown fields
    - Reject invalid grid values
    - Fail-closed on any schema violation
    - Write atomically with backup
    - Full audit logging
    """
    if not request.is_json:
        return jsonify({"error": "Content-Type must be application/json"}), 400
    
    try:
        dashboard = request.get_json()
    except Exception as e:
        logger.error(f"Invalid JSON in save request: {e}")
        return jsonify({"error": "Invalid JSON"}), 400
    
    if not dashboard:
        return jsonify({"error": "Dashboard definition is required"}), 400
    
    # Ensure dashboard name matches URL parameter
    if dashboard.get('name') != dashboard_name:
        logger.warning(f"Dashboard name mismatch: URL={dashboard_name}, JSON={dashboard.get('name')}")
        dashboard['name'] = dashboard_name
    
    # Ensure folder_id exists (default to 'general' if missing)
    if 'folder_id' not in dashboard:
        dashboard['folder_id'] = 'general'
    
    # Validate folder_id exists
    folder = folder_manager.get_folder(dashboard['folder_id'])
    if not folder:
        logger.warning(f"Dashboard '{dashboard_name}' references unknown folder '{dashboard['folder_id']}', defaulting to 'general'")
        dashboard['folder_id'] = 'general'
    
    # Get user ID for audit logging
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Safety check: Never allow saving to system dashboard path
    system_dashboard_path = DASHBOARDS_DIR / f"{dashboard_name}.json"
    if system_dashboard_path.exists():
        logger.info(f"Saving dashboard '{dashboard_name}' as user overlay (user: {user_id}) - system dashboard preserved")
    
    # Validate and save as user overlay (default behavior)
    if dashboard_engine.save_dashboard(dashboard, dashboard_name, save_as_overlay=True):
        logger.info(f"Saved dashboard '{dashboard_name}' as user overlay (user: {user_id})")
        return jsonify({
            "status": "success",
            "message": f"Dashboard '{dashboard_name}' saved as user overlay",
            "dashboard": dashboard_name,
            "source": "user_overlay"
        })
    else:
        return jsonify({"error": "Failed to save dashboard (validation or write error)"}), 400


@app.route('/api/dashboards/<dashboard_name>/versions', methods=['GET'])
def get_dashboard_versions(dashboard_name: str):
    """
    Get version history for a personal dashboard.
    
    Rules:
    - Only personal dashboards have version history
    - System dashboards return empty list
    - Returns metadata only (no full JSON by default)
    - Sorted by timestamp (newest first)
    - Fail-soft on errors (returns empty list)
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Check if dashboard is personal (has overlay)
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # If not a personal dashboard, return empty list
    if not source_info.get('has_overlay'):
        return jsonify({
            "versions": [],
            "dashboard_name": dashboard_name,
            "is_personal": False,
            "message": "System dashboards do not have version history"
        })
    
    try:
        # Get versions (metadata only, no full JSON)
        versions = version_manager.list_versions(dashboard_name, user_id, include_json=False)
        
        return jsonify({
            "versions": versions,
            "dashboard_name": dashboard_name,
            "is_personal": True,
            "count": len(versions)
        })
        
    except Exception as e:
        logger.error(f"Error listing versions for dashboard '{dashboard_name}': {e}", exc_info=True)
        # Fail-soft: return empty list on error
        return jsonify({
            "versions": [],
            "dashboard_name": dashboard_name,
            "is_personal": True,
            "error": "Failed to load version history"
        }), 500


@app.route('/api/dashboards/<dashboard_name>/versions/<version_id>', methods=['GET'])
def get_dashboard_version(dashboard_name: str, version_id: str):
    """
    Get a specific version of a personal dashboard.
    
    Rules:
    - Only personal dashboards have versions
    - Returns full dashboard JSON for the specified version
    - Fail-closed if version not found or corrupted
    - Validates dashboard exists and is personal
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Check if dashboard is personal (has overlay)
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards have versions
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and does not have version history"
        }), 403
    
    try:
        # Get version by ID
        version_data = version_manager.get_version(dashboard_name, version_id, user_id)
        
        if not version_data:
            return jsonify({
                "error": f"Version '{version_id}' not found for dashboard '{dashboard_name}'"
            }), 404
        
        # Validate version data structure
        if 'dashboard' not in version_data:
            logger.error(f"Version '{version_id}' for dashboard '{dashboard_name}' is corrupted (missing dashboard data)")
            return jsonify({
                "error": "Version data is corrupted"
            }), 500
        
        # Return version metadata and dashboard JSON
        return jsonify({
            "version_id": version_data.get('version_id'),
            "timestamp": version_data.get('timestamp'),
            "user_id": version_data.get('user_id'),
            "dashboard_name": version_data.get('dashboard_name'),
            "action": version_data.get('action'),
            "json_hash": version_data.get('json_hash'),
            "dashboard": version_data.get('dashboard')
        })
        
    except Exception as e:
        logger.error(f"Error retrieving version '{version_id}' for dashboard '{dashboard_name}': {e}", exc_info=True)
        return jsonify({
            "error": f"Failed to retrieve version: {str(e)}"
        }), 500


@app.route('/api/dashboards/<dashboard_name>/versions/<version_id>/restore', methods=['POST'])
def restore_dashboard_version(dashboard_name: str, version_id: str):
    """
    Restore a personal dashboard to a previous version.
    
    Rules:
    - Only personal dashboards can be restored
    - System dashboards are never restorable (fail-closed)
    - Validates version exists and is valid
    - Backs up current overlay before restore
    - Creates new version snapshot with action='restore'
    - Audit-logs restore event
    - Fail-closed on any validation error
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Check if dashboard is personal (has overlay)
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can be restored
    if not source_info.get('has_overlay'):
        logger.warning(f"Attempt to restore system dashboard '{dashboard_name}' (user: {user_id})")
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot be restored"
        }), 403
    
    # Validate version exists
    try:
        version_data = version_manager.get_version(dashboard_name, version_id, user_id)
        
        if not version_data:
            logger.warning(f"Version '{version_id}' not found for dashboard '{dashboard_name}' (user: {user_id})")
            return jsonify({
                "error": f"Version '{version_id}' not found for dashboard '{dashboard_name}'"
            }), 404
        
        # Validate version data structure
        if 'dashboard' not in version_data:
            logger.error(f"Version '{version_id}' for dashboard '{dashboard_name}' is corrupted (missing dashboard data)")
            return jsonify({
                "error": "Version data is corrupted"
            }), 500
        
    except Exception as e:
        logger.error(f"Error validating version '{version_id}' for dashboard '{dashboard_name}': {e}", exc_info=True)
        return jsonify({
            "error": f"Failed to validate version: {str(e)}"
        }), 500
    
    # Restore version
    try:
        success = version_manager.restore_version(
            dashboard_name=dashboard_name,
            version_id=version_id,
            user_id=user_id,
            overlay_manager=dashboard_engine.overlay_manager
        )
        
        if not success:
            logger.error(f"Failed to restore dashboard '{dashboard_name}' to version '{version_id}'")
            return jsonify({
                "error": "Failed to restore version (internal error)"
            }), 500
        
        # Audit log restore event (also logged by version_manager, but add API-level log)
        dashboard_engine.overlay_manager._audit_log(
            'restore_dashboard_version', user_id, dashboard_name,
            success=True,
            error=f"version_id:{version_id}"
        )
        
        logger.info(f"Restored dashboard '{dashboard_name}' to version '{version_id}' (user: {user_id})")
        
        return jsonify({
            "success": True,
            "dashboard_name": dashboard_name,
            "version_id": version_id,
            "message": "Dashboard restored successfully"
        })
        
    except Exception as e:
        logger.error(f"Error restoring dashboard '{dashboard_name}' to version '{version_id}': {e}", exc_info=True)
        dashboard_engine.overlay_manager._audit_log(
            'restore_dashboard_version', user_id, dashboard_name,
            success=False,
            error=str(e)
        )
        return jsonify({
            "error": f"Failed to restore version: {str(e)}"
        }), 500


@app.route('/api/health')
def health():
    """System health endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "status": "degraded",
            "message": "Database unavailable",
            "timestamp": datetime.now(timezone.utc).isoformat()
        }), 503
    
    try:
        db = SchemaAwareDB(conn)
        db.safe_query("SELECT 1")
        conn.close()
        return jsonify({
            "status": "healthy",
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Health check failed: {e}", exc_info=True)
        conn.close()
        return jsonify({
            "status": "degraded",
            "message": "Health check failed",
            "timestamp": datetime.now(timezone.utc).isoformat()
        }), 503


@app.route('/api/dashboards/soc')
def dashboard_soc():
    """
    Unified SOC-Grade Dashboard Endpoint.
    Returns all 5 required panels with schema-aware fail-soft queries.
    """
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "degraded"
        }), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # ====================================================================
        # GLOBAL HEADER
        # ====================================================================
        system_health = "healthy"
        last_event_ts = None
        mode = "AIR-GAPPED" if AIR_GAP_MODE else "LIVE"
        
        # Get last event timestamp (check multiple possible columns)
        for table, time_col in [
            ("raw_events", "created_at"),
            ("normalized_events", "created_at"),
            ("linux_agent_telemetry", "received_at"),
            ("linux_agent_telemetry", "observed_at")
        ]:
            if db.table_exists("ransomeye", table) and db.column_exists("ransomeye", table, time_col):
                ts = db.safe_query(
                    f"SELECT MAX({time_col}) FROM ransomeye.{table}",
                    default=None
                )
                if ts:
                    last_event_ts = ts.isoformat() if hasattr(ts, 'isoformat') else str(ts)
                    break
        
        # System health determination
        if not last_event_ts:
            system_health = "degraded"
        else:
            # Check if last event is recent (within 5 minutes)
            try:
                last_dt = datetime.fromisoformat(last_event_ts.replace('Z', '+00:00'))
                if (datetime.now(timezone.utc) - last_dt) > timedelta(minutes=5):
                    system_health = "degraded"
            except:
                pass
        
        # ====================================================================
        # PANEL 1: SENSOR COVERAGE
        # ====================================================================
        sensor_coverage = {
            "linux_agents": {
                "active": 0,
                "stale": 0,
                "total": 0
            },
            "dpi_probes": {
                "active": 0,
                "pps": "Metric unavailable"
            }
        }
        
        # Linux Agents - check for received_at or observed_at
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
            
            if db.column_exists("ransomeye", "linux_agent_telemetry", "agent_id"):
                # Active agents (last 5 minutes)
                active_query = f"""
                    SELECT COUNT(DISTINCT agent_id)
                    FROM ransomeye.linux_agent_telemetry
                    WHERE {time_col} > now() - interval '5 minutes'
                """
                sensor_coverage["linux_agents"]["active"] = db.safe_query(active_query, default=0) or 0
                
                # Stale agents (older than 5 minutes but within 24 hours)
                stale_query = f"""
                    SELECT COUNT(DISTINCT agent_id)
                    FROM ransomeye.linux_agent_telemetry
                    WHERE {time_col} > now() - interval '24 hours'
                      AND {time_col} <= now() - interval '5 minutes'
                """
                sensor_coverage["linux_agents"]["stale"] = db.safe_query(stale_query, default=0) or 0
                
                # Total unique agents
                total_query = "SELECT COUNT(DISTINCT agent_id) FROM ransomeye.linux_agent_telemetry"
                sensor_coverage["linux_agents"]["total"] = db.safe_query(total_query, default=0) or 0
        
        # DPI Probes
        if db.table_exists("ransomeye", "dpi_probe_telemetry"):
            if db.column_exists("ransomeye", "dpi_probe_telemetry", "probe_id"):
                # Active probes (last 5 minutes)
                active_dpi_query = """
                    SELECT COUNT(DISTINCT probe_id)
                    FROM ransomeye.dpi_probe_telemetry
                    WHERE received_at > now() - interval '5 minutes'
                """
                sensor_coverage["dpi_probes"]["active"] = db.safe_query(active_dpi_query, default=0) or 0
                
                # Packets per second (if packets column exists)
                if db.column_exists("ransomeye", "dpi_probe_telemetry", "packet_count"):
                    pps_query = """
                        SELECT SUM(packet_count) / 60.0
                        FROM ransomeye.dpi_probe_telemetry
                        WHERE received_at > now() - interval '1 minute'
                    """
                    pps = db.safe_query(pps_query, default=None)
                    sensor_coverage["dpi_probes"]["pps"] = f"{pps:.2f}" if pps else "Metric unavailable"
        
        # ====================================================================
        # PANEL 2: EVENT PIPELINE
        # ====================================================================
        event_pipeline = {
            "events_per_second": "Metric unavailable",
            "raw_count": 0,
            "normalized_count": 0,
            "time_window_minutes": 1
        }
        
        # Events per second (last minute)
        if db.table_exists("ransomeye", "raw_events"):
            events_last_min = db.safe_query(
                "SELECT COUNT(*) FROM ransomeye.raw_events WHERE created_at > now() - interval '1 minute'",
                default=0
            ) or 0
            event_pipeline["events_per_second"] = round(events_last_min / 60.0, 2)
            event_pipeline["raw_count"] = db.safe_count("ransomeye", "raw_events")
        
        if db.table_exists("ransomeye", "normalized_events"):
            event_pipeline["normalized_count"] = db.safe_count("ransomeye", "normalized_events")
        
        # ====================================================================
        # PANEL 3: SECURITY SIGNALS
        # ====================================================================
        security_signals = {
            "suspicious_activity_count": 0,
            "ioc_hits": {
                "domain": 0,
                "hash": 0,
                "total": 0
            },
            "confidence_distribution": "Metric unavailable"
        }
        
        # Suspicious activity (check severity column if it exists)
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            if db.column_exists("ransomeye", "linux_agent_telemetry", "severity"):
                # Check what severity values exist first
                time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
                suspicious_query = f"""
                    SELECT COUNT(*)
                    FROM ransomeye.linux_agent_telemetry
                    WHERE severity IN ('warning', 'error', 'critical')
                      AND {time_col} > now() - interval '24 hours'
                """
                security_signals["suspicious_activity_count"] = db.safe_query(suspicious_query, default=0) or 0
        
        # IOC Hits
        if db.table_exists("ransomeye", "threat_intel"):
            if db.column_exists("ransomeye", "threat_intel", "ioc_type"):
                # Domain hits
                domain_query = """
                    SELECT COUNT(*)
                    FROM ransomeye.threat_intel
                    WHERE ioc_type = 'domain'
                """
                security_signals["ioc_hits"]["domain"] = db.safe_query(domain_query, default=0) or 0
                
                # Hash hits
                hash_query = """
                    SELECT COUNT(*)
                    FROM ransomeye.threat_intel
                    WHERE ioc_type IN ('hash', 'sha256', 'md5')
                """
                security_signals["ioc_hits"]["hash"] = db.safe_query(hash_query, default=0) or 0
                
                # Total IOCs
                security_signals["ioc_hits"]["total"] = db.safe_count("ransomeye", "threat_intel")
            
            # Confidence distribution (only if column exists)
            if db.column_exists("ransomeye", "threat_intel", "confidence"):
                conf_query = """
                    SELECT 
                        CASE 
                            WHEN confidence >= 0.8 THEN 'high'
                            WHEN confidence >= 0.5 THEN 'medium'
                            ELSE 'low'
                        END as level,
                        COUNT(*) as count
                    FROM ransomeye.threat_intel
                    GROUP BY level
                """
                conf_rows = db.safe_query_all(conf_query, default=[])
                if conf_rows:
                    security_signals["confidence_distribution"] = {
                        row[0]: row[1] for row in conf_rows
                    }
        
        # ====================================================================
        # PANEL 4: RECENT ACTIVITY FEED
        # ====================================================================
        recent_activity = []
        
        # Get recent security events (human-readable)
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            # Build query based on available columns
            select_cols = ["event_name", "event_category", "severity", "observed_at", "process_name"]
            available_cols = [col for col in select_cols if db.column_exists("ransomeye", "linux_agent_telemetry", col)]
            
            if available_cols:
                time_col = "observed_at" if "observed_at" in available_cols else "received_at"
                query = f"""
                    SELECT {', '.join(available_cols)}, {time_col}
                    FROM ransomeye.linux_agent_telemetry
                    ORDER BY {time_col} DESC
                    LIMIT 20
                """
                rows = db.safe_query_all(query, default=[])
                
                for row in rows:
                    event = {}
                    for i, col in enumerate(available_cols):
                        val = row[i]
                        if hasattr(val, 'isoformat'):
                            val = val.isoformat()
                        event[col] = val
                    
                    # Add human-readable description
                    event_name = event.get("event_name", "Unknown")
                    process = event.get("process_name", "Unknown process")
                    severity = event.get("severity", "info")
                    event["description"] = f"{severity.upper()}: {event_name} from {process}"
                    
                    recent_activity.append(event)
        
        # ====================================================================
        # RANSOMWARE RISK SNAPSHOT
        # ====================================================================
        ransomware_risk = {
            "active_signals": 0,
            "lateral_movement": 0,
            "high_risk_hosts": 0,
            "detection_confidence": 0.0
        }
        
        # Active ransomware signals (critical severity events in last 24h)
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
            if db.column_exists("ransomeye", "linux_agent_telemetry", "severity"):
                ransomware_query = f"""
                    SELECT COUNT(*)
                    FROM ransomeye.linux_agent_telemetry
                    WHERE severity = 'critical'
                      AND {time_col} > now() - interval '24 hours'
                """
                ransomware_risk["active_signals"] = db.safe_query(ransomware_query, default=0) or 0
        
        # Lateral movement (check detection_results for MITRE technique T1021)
        if db.table_exists("ransomeye", "detection_results"):
            if db.column_exists("ransomeye", "detection_results", "mitre_technique"):
                lateral_query = """
                    SELECT COUNT(*)
                    FROM ransomeye.detection_results
                    WHERE mitre_technique LIKE '%T1021%'
                      AND created_at > now() - interval '24 hours'
                """
                ransomware_risk["lateral_movement"] = db.safe_query(lateral_query, default=0) or 0
        
        # High-risk hosts (hosts with multiple critical events)
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
            if db.column_exists("ransomeye", "linux_agent_telemetry", "agent_id") and db.column_exists("ransomeye", "linux_agent_telemetry", "severity"):
                high_risk_query = f"""
                    SELECT COUNT(DISTINCT agent_id)
                    FROM ransomeye.linux_agent_telemetry
                    WHERE severity IN ('critical', 'error')
                      AND {time_col} > now() - interval '24 hours'
                    GROUP BY agent_id
                    HAVING COUNT(*) >= 3
                """
                high_risk_rows = db.safe_query_all(high_risk_query, default=[])
                ransomware_risk["high_risk_hosts"] = len(high_risk_rows) if high_risk_rows else 0
        
        # Detection confidence (average from detection_results)
        if db.table_exists("ransomeye", "detection_results"):
            if db.column_exists("ransomeye", "detection_results", "confidence"):
                conf_query = """
                    SELECT AVG(confidence)
                    FROM ransomeye.detection_results
                    WHERE created_at > now() - interval '24 hours'
                """
                avg_conf = db.safe_query(conf_query, default=None)
                ransomware_risk["detection_confidence"] = round(float(avg_conf), 2) if avg_conf else 0.0
        
        # ====================================================================
        # ACTIVITY TIMELINE DATA (last 24 hours, hourly buckets)
        # ====================================================================
        activity_timeline = {
            "linux": [],
            "network": [],
            "correlated": []
        }
        
        # Linux events timeline
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
            timeline_query = f"""
                SELECT 
                    DATE_TRUNC('hour', {time_col}) as hour,
                    COUNT(*) as count
                FROM ransomeye.linux_agent_telemetry
                WHERE {time_col} > now() - interval '24 hours'
                GROUP BY hour
                ORDER BY hour
            """
            linux_rows = db.safe_query_all(timeline_query, default=[])
            for row in linux_rows:
                hour_ts = row[0].isoformat() if hasattr(row[0], 'isoformat') else str(row[0])
                activity_timeline["linux"].append({"time": hour_ts, "count": row[1]})
        
        # Network events timeline (DPI probe)
        if db.table_exists("ransomeye", "dpi_probe_telemetry"):
            if db.column_exists("ransomeye", "dpi_probe_telemetry", "received_at"):
                network_query = """
                    SELECT 
                        DATE_TRUNC('hour', received_at) as hour,
                        COUNT(*) as count
                    FROM ransomeye.dpi_probe_telemetry
                    WHERE received_at > now() - interval '24 hours'
                    GROUP BY hour
                    ORDER BY hour
                """
                network_rows = db.safe_query_all(network_query, default=[])
                for row in network_rows:
                    hour_ts = row[0].isoformat() if hasattr(row[0], 'isoformat') else str(row[0])
                    activity_timeline["network"].append({"time": hour_ts, "count": row[1]})
        
        # Correlated events timeline (detection_results)
        if db.table_exists("ransomeye", "detection_results"):
            if db.column_exists("ransomeye", "detection_results", "created_at"):
                correlated_query = """
                    SELECT 
                        DATE_TRUNC('hour', created_at) as hour,
                        COUNT(*) as count
                    FROM ransomeye.detection_results
                    WHERE created_at > now() - interval '24 hours'
                    GROUP BY hour
                    ORDER BY hour
                """
                correlated_rows = db.safe_query_all(correlated_query, default=[])
                for row in correlated_rows:
                    hour_ts = row[0].isoformat() if hasattr(row[0], 'isoformat') else str(row[0])
                    activity_timeline["correlated"].append({"time": hour_ts, "count": row[1]})
        
        # ====================================================================
        # ENHANCED RECENT SECURITY EVENTS (better human-readable descriptions)
        # ====================================================================
        # Re-fetch with better descriptions
        recent_activity = []
        
        # Get from detection_results first (more structured)
        if db.table_exists("ransomeye", "detection_results"):
            det_cols = ["detection_name", "detection_category", "severity", "created_at", "confidence", "mitre_tactic", "mitre_technique"]
            available_det_cols = [col for col in det_cols if db.column_exists("ransomeye", "detection_results", col)]
            
            if available_det_cols:
                det_query = f"""
                    SELECT {', '.join(available_det_cols)}
                    FROM ransomeye.detection_results
                    ORDER BY created_at DESC
                    LIMIT 15
                """
                det_rows = db.safe_query_all(det_query, default=[])
                
                for row in det_rows:
                    event = {}
                    for i, col in enumerate(available_det_cols):
                        val = row[i]
                        if hasattr(val, 'isoformat'):
                            val = val.isoformat()
                        event[col] = val
                    
                    # Human-readable description
                    name = event.get("detection_name", "Security Detection")
                    category = event.get("detection_category", "")
                    severity = event.get("severity", "info")
                    mitre = event.get("mitre_technique", "")
                    event["description"] = f"{name}" + (f" ({category})" if category else "") + (f" - {mitre}" if mitre else "")
                    event["source"] = "Correlation Engine"
                    recent_activity.append(event)
        
        # Also get from linux_agent_telemetry
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            select_cols = ["event_name", "event_category", "severity", "observed_at", "process_name", "agent_id"]
            available_cols = [col for col in select_cols if db.column_exists("ransomeye", "linux_agent_telemetry", col)]
            
            if available_cols:
                time_col = "observed_at" if "observed_at" in available_cols else "received_at"
                query = f"""
                    SELECT {', '.join(available_cols)}, {time_col}
                    FROM ransomeye.linux_agent_telemetry
                    WHERE severity IN ('warning', 'error', 'critical')
                    ORDER BY {time_col} DESC
                    LIMIT 10
                """
                rows = db.safe_query_all(query, default=[])
                
                for row in rows:
                    event = {}
                    for i, col in enumerate(available_cols):
                        val = row[i]
                        if hasattr(val, 'isoformat'):
                            val = val.isoformat()
                        event[col] = val
                    
                    # Human-readable description
                    event_name = event.get("event_name", "Security Event")
                    process = event.get("process_name", "")
                    severity = event.get("severity", "info")
                    agent_id = event.get("agent_id", "")
                    event["description"] = f"{event_name}" + (f" from {process}" if process else "")
                    event["source"] = f"Linux Agent {agent_id[:8] if agent_id else 'Unknown'}"
                    recent_activity.append(event)
        
        # Sort by timestamp and limit to 20 most recent
        recent_activity.sort(key=lambda x: x.get("created_at") or x.get("observed_at") or "", reverse=True)
        recent_activity = recent_activity[:20]
        
        # ====================================================================
        # PANEL 5: INTEGRITY
        # ====================================================================
        integrity = {
            "tamper_protection": "Metric unavailable",
            "audit_chain_status": "Metric unavailable",
            "drift_detection": "Metric unavailable"
        }
        
        # Tamper protection (check for immutable audit trigger)
        tamper_query = """
            SELECT EXISTS (
                SELECT 1 FROM pg_trigger 
                WHERE tgname = 'trg_immutable_audit_no_update'
            )
        """
        tamper_exists = db.safe_query(tamper_query, default=False)
        integrity["tamper_protection"] = "Enabled" if tamper_exists else "Disabled"
        
        # Audit chain status
        if db.table_exists("ransomeye", "immutable_audit_log"):
            audit_count = db.safe_count("ransomeye", "immutable_audit_log")
            if db.column_exists("ransomeye", "immutable_audit_log", "created_at"):
                recent_audit = db.safe_count(
                    "ransomeye", "immutable_audit_log",
                    "created_at > now() - interval '24 hours'"
                )
                integrity["audit_chain_status"] = f"Active ({recent_audit} entries in 24h, {audit_count} total)"
            else:
                integrity["audit_chain_status"] = f"Active ({audit_count} entries)"
        
        # Drift detection (check for baseline tables)
        if db.table_exists("ransomeye", "golden_baseline") or db.table_exists("ransomeye", "baseline_snapshots"):
            integrity["drift_detection"] = "Available"
        else:
            integrity["drift_detection"] = "Not configured"
        
        cursor.close()
        conn.close()
        
        # Determine overall ransomware status
        ransomware_status = "Protected"
        if ransomware_risk["active_signals"] > 0 or ransomware_risk["lateral_movement"] > 0:
            ransomware_status = "Active Threat"
        elif ransomware_risk["high_risk_hosts"] > 0:
            ransomware_status = "At Risk"
        
        return jsonify({
            "header": {
                "ransomware_status": ransomware_status,
                "system_health": system_health,
                "last_event_timestamp": last_event_ts,
                "mode": mode,
                "sensor_coverage": {
                    "linux_agents": sensor_coverage["linux_agents"]["active"],
                    "dpi_probes": sensor_coverage["dpi_probes"]["active"]
                },
                "timestamp": datetime.now(timezone.utc).isoformat()
            },
            "ransomware_risk": ransomware_risk,
            "activity_timeline": activity_timeline,
            "recent_events": recent_activity,
            "sensor_health": {
                "linux_agents": sensor_coverage["linux_agents"],
                "dpi_probes": sensor_coverage["dpi_probes"],
                "integrity": integrity
            }
        })
    
    except Exception as e:
        logger.error(f"SOC dashboard error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({
            "error": "Dashboard unavailable",
            "status": "degraded",
            "message": "Metric unavailable"
        }), 500


# Legacy endpoints - kept for backward compatibility, now fail-soft
@app.route('/api/dashboards/system-health')
def dashboard_system_health():
    """Legacy system health endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        ingestion_status = "inactive"
        if db.table_exists("ransomeye", "components"):
            if db.column_exists("ransomeye", "components", "component_type"):
                count = db.safe_query(
                    "SELECT COUNT(*) FROM ransomeye.components WHERE component_type = 'core_engine'",
                    default=0
                ) or 0
                ingestion_status = "active" if count > 0 else "inactive"
        
        active_agents = 0
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            time_col = "received_at" if db.column_exists("ransomeye", "linux_agent_telemetry", "received_at") else "observed_at"
            if db.column_exists("ransomeye", "linux_agent_telemetry", "agent_id"):
                active_agents = db.safe_query(
                    f"SELECT COUNT(DISTINCT agent_id) FROM ransomeye.linux_agent_telemetry WHERE {time_col} > now() - interval '5 minutes'",
                    default=0
                ) or 0
        
        audit_count = db.safe_count("ransomeye", "immutable_audit_log") if db.table_exists("ransomeye", "immutable_audit_log") else 0
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "ingestion_status": ingestion_status,
            "normalization_status": "active",
            "active_agents": active_agents,
            "db_connectivity": "connected",
            "audit_log_entries": audit_count,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"System health error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/telemetry')
def dashboard_telemetry():
    """Legacy telemetry endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        events_per_sec = "Metric unavailable"
        if db.table_exists("ransomeye", "raw_events"):
            events_last_min = db.safe_query(
                "SELECT COUNT(*) FROM ransomeye.raw_events WHERE created_at > now() - interval '1 minute'",
                default=0
            ) or 0
            events_per_sec = round(events_last_min / 60.0, 2)
        
        agent_count = db.safe_count("ransomeye", "linux_agent_telemetry") if db.table_exists("ransomeye", "linux_agent_telemetry") else 0
        raw_count = db.safe_count("ransomeye", "raw_events") if db.table_exists("ransomeye", "raw_events") else 0
        normalized_count = db.safe_count("ransomeye", "normalized_events") if db.table_exists("ransomeye", "normalized_events") else 0
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "events_per_second": events_per_sec,
            "agent_telemetry_count": agent_count,
            "raw_events_count": raw_count,
            "normalized_events_count": normalized_count,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Telemetry error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/threat-intel')
def dashboard_threat_intel():
    """Legacy threat intel endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        total_iocs = db.safe_count("ransomeye", "threat_intel") if db.table_exists("ransomeye", "threat_intel") else 0
        
        ioc_by_type = {}
        ioc_by_source = {}
        confidence_dist = "Metric unavailable"
        
        if db.table_exists("ransomeye", "threat_intel"):
            if db.column_exists("ransomeye", "threat_intel", "ioc_type"):
                rows = db.safe_query_all(
                    "SELECT ioc_type, COUNT(*) FROM ransomeye.threat_intel GROUP BY ioc_type",
                    default=[]
                )
                ioc_by_type = {row[0]: row[1] for row in rows}
            
            if db.column_exists("ransomeye", "threat_intel", "source"):
                rows = db.safe_query_all(
                    "SELECT source, COUNT(*) FROM ransomeye.threat_intel GROUP BY source",
                    default=[]
                )
                ioc_by_source = {row[0]: row[1] for row in rows}
            
            if db.column_exists("ransomeye", "threat_intel", "confidence"):
                rows = db.safe_query_all("""
                    SELECT 
                        CASE 
                            WHEN confidence >= 0.8 THEN 'high'
                            WHEN confidence >= 0.5 THEN 'medium'
                            ELSE 'low'
                        END as level,
                        COUNT(*) as count
                    FROM ransomeye.threat_intel
                    GROUP BY level
                """, default=[])
                if rows:
                    confidence_dist = {row[0]: row[1] for row in rows}
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "total_iocs": total_iocs,
            "ioc_by_type": ioc_by_type,
            "ioc_by_source": ioc_by_source,
            "confidence_distribution": confidence_dist,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Threat intel error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/detections')
def dashboard_detections():
    """Legacy detections endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        risk_dist = {}
        model_version = "unknown"
        shap_available = False
        shap_count = 0
        
        # Risk distribution (only if table and column exist)
        if db.table_exists("ransomeye", "detection_results"):
            if db.column_exists("ransomeye", "detection_results", "risk_score"):
                rows = db.safe_query_all("""
                    SELECT 
                        CASE 
                            WHEN risk_score >= 0.8 THEN 'critical'
                            WHEN risk_score >= 0.6 THEN 'high'
                            WHEN risk_score >= 0.4 THEN 'medium'
                            ELSE 'low'
                        END as level,
                        COUNT(*) as count
                    FROM ransomeye.detection_results
                    GROUP BY level
                """, default=[])
                risk_dist = {row[0]: row[1] for row in rows}
        
        # Model version
        if db.table_exists("ransomeye", "inference_results"):
            if db.column_exists("ransomeye", "inference_results", "model_version"):
                model_version = db.safe_query(
                    "SELECT DISTINCT model_version FROM ransomeye.inference_results LIMIT 1",
                    default="unknown"
                ) or "unknown"
        
        # SHAP availability
        if db.table_exists("ransomeye", "shap_explanations"):
            shap_count = db.safe_count("ransomeye", "shap_explanations")
            shap_available = shap_count > 0
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "risk_distribution": risk_dist,
            "model_version": model_version,
            "shap_available": shap_available,
            "shap_explanation_count": shap_count,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Detections error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/audit')
def dashboard_audit():
    """Legacy audit endpoint - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        audit_growth_24h = 0
        action_breakdown = {}
        total_audit_entries = 0
        tamper_proof_enabled = False
        
        if db.table_exists("ransomeye", "immutable_audit_log"):
            if db.column_exists("ransomeye", "immutable_audit_log", "created_at"):
                audit_growth_24h = db.safe_count(
                    "ransomeye", "immutable_audit_log",
                    "created_at > now() - interval '24 hours'"
                )
            
            if db.column_exists("ransomeye", "immutable_audit_log", "action"):
                rows = db.safe_query_all(
                    "SELECT action, COUNT(*) FROM ransomeye.immutable_audit_log GROUP BY action ORDER BY COUNT(*) DESC LIMIT 10",
                    default=[]
                )
                action_breakdown = {row[0]: row[1] for row in rows}
            
            total_audit_entries = db.safe_count("ransomeye", "immutable_audit_log")
        
        # Tamper protection
        tamper_proof_enabled = db.safe_query(
            "SELECT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_immutable_audit_no_update')",
            default=False
        )
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "audit_growth_24h": audit_growth_24h,
            "action_breakdown": action_breakdown,
            "total_audit_entries": total_audit_entries,
            "tamper_proof_enabled": tamper_proof_enabled,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Audit error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/system/instances')
def system_instances():
    """Instance discovery endpoint - returns available Core, DPI, and DB instances."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        core_instances = []
        dpi_instances = []
        db_instances = []
        
        # Discover Core instances from components table
        if db.table_exists("ransomeye", "components"):
            if db.column_exists("ransomeye", "components", "component_type"):
                core_rows = db.safe_query_all(
                    """SELECT component_id, component_name, instance_id, last_heartbeat_at
                       FROM ransomeye.components
                       WHERE component_type = 'core_engine'
                       ORDER BY component_name, instance_id NULLS LAST""",
                    default=[]
                )
                
                for row in core_rows:
                    component_id = str(row[0]) if row[0] else None
                    component_name = row[1] or "unknown"
                    instance_id = row[2] or component_id or "default"
                    last_heartbeat = row[3].isoformat() if row[3] and hasattr(row[3], 'isoformat') else None
                    
                    # Try to get hostname/IP from linux_agent_telemetry if linked
                    hostname = None
                    ip = None
                    if db.table_exists("ransomeye", "linux_agent_telemetry"):
                        if db.column_exists("ransomeye", "linux_agent_telemetry", "source_host_id"):
                            if component_id:
                                host_row = db.safe_query(
                                    """SELECT DISTINCT source_host_id 
                                       FROM ransomeye.linux_agent_telemetry
                                       WHERE source_component_identity = %s
                                       ORDER BY observed_at DESC LIMIT 1""",
                                    (component_id,),
                                    default=None
                                )
                                if host_row:
                                    hostname = str(host_row) if host_row else None
                    
                    # Determine role (primary if first, replica if others exist)
                    role = "primary" if len(core_instances) == 0 else "replica"
                    
                    core_instances.append({
                        "id": instance_id,
                        "component_id": component_id,
                        "hostname": hostname or component_name,
                        "ip": ip,
                        "role": role,
                        "last_heartbeat": last_heartbeat
                    })
        
        # Discover DPI Probe instances from dpi_probe_telemetry
        if db.table_exists("ransomeye", "dpi_probe_telemetry"):
            if db.column_exists("ransomeye", "dpi_probe_telemetry", "agent_id"):
                dpi_rows = db.safe_query_all(
                    """SELECT DISTINCT agent_id, source_component_identity, iface_name, MAX(observed_at) as last_seen
                       FROM ransomeye.dpi_probe_telemetry
                       WHERE agent_id IS NOT NULL
                       GROUP BY agent_id, source_component_identity, iface_name
                       ORDER BY last_seen DESC""",
                    default=[]
                )
                
                for row in dpi_rows:
                    agent_id = str(row[0]) if row[0] else None
                    component_identity = row[1] or agent_id or "unknown"
                    iface = row[2] or "unknown"
                    last_seen = row[3].isoformat() if row[3] and hasattr(row[3], 'isoformat') else None
                    
                    # Use component_identity as stable ID, fallback to agent_id
                    probe_id = component_identity if component_identity != "unknown" else agent_id
                    
                    dpi_instances.append({
                        "id": probe_id,
                        "agent_id": agent_id,
                        "hostname": component_identity,
                        "iface": iface,
                        "last_seen": last_seen
                    })
        
        # Discover DB instances from components table (if DB components registered)
        # For now, we'll use a single DB instance identifier
        # In HA setups, this would query pg_stat_replication or similar
        if db.table_exists("ransomeye", "components"):
            db_rows = db.safe_query_all(
                """SELECT component_id, component_name, instance_id, last_heartbeat_at
                   FROM ransomeye.components
                   WHERE component_type IN ('db_core', 'database', 'postgres')
                   ORDER BY component_name, instance_id NULLS LAST""",
                default=[]
            )
            
            for row in db_rows:
                component_id = str(row[0]) if row[0] else None
                component_name = row[1] or "unknown"
                instance_id = row[2] or component_id or "db-01"
                last_heartbeat = row[3].isoformat() if row[3] and hasattr(row[3], 'isoformat') else None
                
                # Determine role (primary if first, replica if others exist)
                role = "primary" if len(db_instances) == 0 else "replica"
                
                db_instances.append({
                    "id": instance_id,
                    "component_id": component_id,
                    "role": role,
                    "last_heartbeat": last_heartbeat
                })
        
        # If no DB instances found in components, create a default one
        if not db_instances:
            db_instances.append({
                "id": "db-01",
                "component_id": None,
                "role": "primary",
                "last_heartbeat": None
            })
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "core": core_instances,
            "dpi": dpi_instances,
            "db": db_instances,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Instance discovery error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({
            "core": [],
            "dpi": [],
            "db": [],
            "error": "Instance discovery unavailable"
        }), 500


@app.route('/api/dashboards/core-system-health')
def dashboard_core_system_health():
    """Core system health metrics from Linux Agent telemetry - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Get instance_id from query parameter
        instance_id = request.args.get('instance_id', None)
        
        # Get latest system metrics from Linux Agent telemetry payload
        # Extract CPU, memory, disk, network, filesystem, and system state metrics
        cpu_data = {"utilization": "Metric unavailable", "load_avg_1m": "Metric unavailable", 
                   "load_avg_5m": "Metric unavailable", "load_avg_15m": "Metric unavailable",
                   "core_count": "Metric unavailable", "context_switches": "Metric unavailable"}
        memory_data = {"total": "Metric unavailable", "used": "Metric unavailable",
                      "free": "Metric unavailable", "swap_used": "Metric unavailable"}
        disk_io_data = {"read_iops": "Metric unavailable", "write_iops": "Metric unavailable",
                       "read_throughput": "Metric unavailable", "write_throughput": "Metric unavailable",
                       "utilization": "Metric unavailable"}
        filesystem_data = {"root_usage": "Metric unavailable", "critical_mounts": "Metric unavailable",
                          "inode_usage": "Metric unavailable"}
        network_data = {"bytes_in": "Metric unavailable", "bytes_out": "Metric unavailable",
                       "packets_in": "Metric unavailable", "packets_out": "Metric unavailable",
                       "errors": "Metric unavailable", "drops": "Metric unavailable"}
        system_state_data = {"uptime": "Metric unavailable", "process_count": "Metric unavailable",
                            "zombie_count": "Metric unavailable"}
        
        # Build query with instance filter if provided
        query = """SELECT payload, observed_at, agent_id, source_component_identity 
                   FROM ransomeye.linux_agent_telemetry 
                   WHERE payload IS NOT NULL AND payload::text LIKE '%system%'"""
        params = []
        
        if instance_id:
            # Filter by component identity or instance_id
            if db.column_exists("ransomeye", "linux_agent_telemetry", "source_component_identity"):
                query += " AND (source_component_identity = %s OR agent_id::text = %s)"
                params.extend([instance_id, instance_id])
            elif db.column_exists("ransomeye", "linux_agent_telemetry", "agent_id"):
                query += " AND agent_id::text = %s"
                params.append(instance_id)
        
        query += " ORDER BY observed_at DESC LIMIT 1"
        
        # Query latest telemetry with system metrics in payload
        if db.table_exists("ransomeye", "linux_agent_telemetry"):
            if db.column_exists("ransomeye", "linux_agent_telemetry", "payload"):
                if params:
                    latest_telemetry = db.safe_query_all(query, tuple(params), default=[])
                else:
                    latest_telemetry = db.safe_query_all(query, default=[])
                
                # If instance_id specified but no results, return 404
                if instance_id and not latest_telemetry:
                    cursor.close()
                    conn.close()
                    return jsonify({"error": f"Instance '{instance_id}' not found or offline"}), 404
                
                if latest_telemetry:
                    import json as json_lib
                    payload = latest_telemetry[0][0]
                    if payload and isinstance(payload, dict):
                        # Extract system metrics from payload (fail-soft if structure differs)
                        if 'cpu' in payload:
                            cpu_payload = payload['cpu']
                            cpu_data = {
                                "utilization": cpu_payload.get('utilization', cpu_data['utilization']),
                                "load_avg_1m": cpu_payload.get('load_avg_1m', cpu_data['load_avg_1m']),
                                "load_avg_5m": cpu_payload.get('load_avg_5m', cpu_data['load_avg_5m']),
                                "load_avg_15m": cpu_payload.get('load_avg_15m', cpu_data['load_avg_15m']),
                                "core_count": cpu_payload.get('core_count', cpu_data['core_count']),
                                "context_switches": cpu_payload.get('context_switches', cpu_data['context_switches'])
                            }
                        if 'memory' in payload:
                            mem_payload = payload['memory']
                            memory_data = {
                                "total": mem_payload.get('total', memory_data['total']),
                                "used": mem_payload.get('used', memory_data['used']),
                                "free": mem_payload.get('free', memory_data['free']),
                                "swap_used": mem_payload.get('swap_used', memory_data['swap_used'])
                            }
                        if 'disk' in payload:
                            disk_payload = payload['disk']
                            disk_io_data = {
                                "read_iops": disk_payload.get('read_iops', disk_io_data['read_iops']),
                                "write_iops": disk_payload.get('write_iops', disk_io_data['write_iops']),
                                "read_throughput": disk_payload.get('read_throughput', disk_io_data['read_throughput']),
                                "write_throughput": disk_payload.get('write_throughput', disk_io_data['write_throughput']),
                                "utilization": disk_payload.get('utilization', disk_io_data['utilization'])
                            }
                        if 'filesystem' in payload:
                            fs_payload = payload['filesystem']
                            filesystem_data = {
                                "root_usage": fs_payload.get('root_usage', filesystem_data['root_usage']),
                                "critical_mounts": fs_payload.get('critical_mounts', filesystem_data['critical_mounts']),
                                "inode_usage": fs_payload.get('inode_usage', filesystem_data['inode_usage'])
                            }
                        if 'network' in payload:
                            net_payload = payload['network']
                            network_data = {
                                "bytes_in": net_payload.get('bytes_in', network_data['bytes_in']),
                                "bytes_out": net_payload.get('bytes_out', network_data['bytes_out']),
                                "packets_in": net_payload.get('packets_in', network_data['packets_in']),
                                "packets_out": net_payload.get('packets_out', network_data['packets_out']),
                                "errors": net_payload.get('errors', network_data['errors']),
                                "drops": net_payload.get('drops', network_data['drops'])
                            }
                        if 'system' in payload:
                            sys_payload = payload['system']
                            system_state_data = {
                                "uptime": sys_payload.get('uptime', system_state_data['uptime']),
                                "process_count": sys_payload.get('process_count', system_state_data['process_count']),
                                "zombie_count": sys_payload.get('zombie_count', system_state_data['zombie_count'])
                            }
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "cpu": cpu_data,
            "memory": memory_data,
            "disk_io": disk_io_data,
            "filesystem": filesystem_data,
            "network": network_data,
            "system_state": system_state_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Core system health error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/dpi-probe-health')
def dashboard_dpi_probe_health():
    """DPI Probe health metrics from telemetry payload - fail-soft."""
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Get probe_id from query parameter
        probe_id = request.args.get('probe_id', None)
        
        # Get latest system metrics from DPI Probe telemetry payload
        cpu_load_data = {"cpu_utilization": "Metric unavailable", "load_avg_1m": "Metric unavailable",
                        "load_avg_5m": "Metric unavailable", "load_avg_15m": "Metric unavailable",
                        "core_count": "Metric unavailable", "dpi_process_cpu": "Metric unavailable"}
        memory_data = {"total": "Metric unavailable", "used": "Metric unavailable",
                      "free": "Metric unavailable", "swap_used": "Metric unavailable",
                      "dpi_process_rss": "Metric unavailable"}
        disk_io_data = {"read_throughput": "Metric unavailable", "write_throughput": "Metric unavailable",
                       "utilization": "Metric unavailable"}
        network_throughput_data = {"packets_per_sec": "Metric unavailable", "bytes_per_sec": "Metric unavailable",
                                  "drops": "Metric unavailable", "errors": "Metric unavailable",
                                  "ring_buffer_drops": "Metric unavailable"}
        processing_health_data = {"packet_processing_rate": "Metric unavailable", "packet_drops": "Metric unavailable",
                                 "probe_uptime": "Metric unavailable", "probe_status": "Metric unavailable"}
        system_state_data = {"host_uptime": "Metric unavailable", "process_count": "Metric unavailable",
                            "dpi_process_status": "Metric unavailable"}
        
        # Build query with probe filter if provided
        query = """SELECT payload, observed_at, agent_id, source_component_identity 
                   FROM ransomeye.dpi_probe_telemetry 
                   WHERE payload IS NOT NULL AND payload::text LIKE '%system%'"""
        params = []
        
        if probe_id:
            # Filter by component identity or agent_id
            if db.column_exists("ransomeye", "dpi_probe_telemetry", "source_component_identity"):
                query += " AND (source_component_identity = %s OR agent_id::text = %s)"
                params.extend([probe_id, probe_id])
            elif db.column_exists("ransomeye", "dpi_probe_telemetry", "agent_id"):
                query += " AND agent_id::text = %s"
                params.append(probe_id)
        
        query += " ORDER BY observed_at DESC LIMIT 1"
        
        # Query latest DPI Probe telemetry with system metrics in payload
        if db.table_exists("ransomeye", "dpi_probe_telemetry"):
            if db.column_exists("ransomeye", "dpi_probe_telemetry", "payload"):
                if params:
                    latest_telemetry = db.safe_query_all(query, tuple(params), default=[])
                else:
                    latest_telemetry = db.safe_query_all(query, default=[])
                
                # If probe_id specified but no results, return 404
                if probe_id and not latest_telemetry:
                    cursor.close()
                    conn.close()
                    return jsonify({"error": f"Probe '{probe_id}' not found or offline"}), 404
                
                if latest_telemetry:
                    import json as json_lib
                    payload = latest_telemetry[0][0]
                    if payload and isinstance(payload, dict):
                        # Extract system metrics from payload (fail-soft if structure differs)
                        if 'system' in payload:
                            sys_payload = payload['system']
                            # CPU & Load
                            if 'cpu' in sys_payload:
                                cpu_payload = sys_payload['cpu']
                                cpu_load_data = {
                                    "cpu_utilization": cpu_payload.get('utilization', cpu_load_data['cpu_utilization']),
                                    "load_avg_1m": cpu_payload.get('load_avg_1m', cpu_load_data['load_avg_1m']),
                                    "load_avg_5m": cpu_payload.get('load_avg_5m', cpu_load_data['load_avg_5m']),
                                    "load_avg_15m": cpu_payload.get('load_avg_15m', cpu_load_data['load_avg_15m']),
                                    "core_count": cpu_payload.get('core_count', cpu_load_data['core_count']),
                                    "dpi_process_cpu": cpu_payload.get('dpi_process_cpu', cpu_load_data['dpi_process_cpu'])
                                }
                            # Memory
                            if 'memory' in sys_payload:
                                mem_payload = sys_payload['memory']
                                memory_data = {
                                    "total": mem_payload.get('total', memory_data['total']),
                                    "used": mem_payload.get('used', memory_data['used']),
                                    "free": mem_payload.get('free', memory_data['free']),
                                    "swap_used": mem_payload.get('swap_used', memory_data['swap_used']),
                                    "dpi_process_rss": mem_payload.get('dpi_process_rss', memory_data['dpi_process_rss'])
                                }
                            # Disk I/O
                            if 'disk' in sys_payload:
                                disk_payload = sys_payload['disk']
                                disk_io_data = {
                                    "read_throughput": disk_payload.get('read_throughput', disk_io_data['read_throughput']),
                                    "write_throughput": disk_payload.get('write_throughput', disk_io_data['write_throughput']),
                                    "utilization": disk_payload.get('utilization', disk_io_data['utilization'])
                                }
                            # Network Throughput
                            if 'network' in sys_payload:
                                net_payload = sys_payload['network']
                                network_throughput_data = {
                                    "packets_per_sec": net_payload.get('packets_per_sec', network_throughput_data['packets_per_sec']),
                                    "bytes_per_sec": net_payload.get('bytes_per_sec', network_throughput_data['bytes_per_sec']),
                                    "drops": net_payload.get('drops', network_throughput_data['drops']),
                                    "errors": net_payload.get('errors', network_throughput_data['errors']),
                                    "ring_buffer_drops": net_payload.get('ring_buffer_drops', network_throughput_data['ring_buffer_drops'])
                                }
                            # Processing Health
                            if 'processing' in sys_payload:
                                proc_payload = sys_payload['processing']
                                processing_health_data = {
                                    "packet_processing_rate": proc_payload.get('packet_processing_rate', processing_health_data['packet_processing_rate']),
                                    "packet_drops": proc_payload.get('packet_drops', processing_health_data['packet_drops']),
                                    "probe_uptime": proc_payload.get('probe_uptime', processing_health_data['probe_uptime']),
                                    "probe_status": proc_payload.get('probe_status', processing_health_data['probe_status'])
                                }
                            # System State
                            if 'system_state' in sys_payload:
                                state_payload = sys_payload['system_state']
                                system_state_data = {
                                    "host_uptime": state_payload.get('host_uptime', system_state_data['host_uptime']),
                                    "process_count": state_payload.get('process_count', system_state_data['process_count']),
                                    "dpi_process_status": state_payload.get('dpi_process_status', system_state_data['dpi_process_status'])
                                }
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "cpu_load": cpu_load_data,
            "memory": memory_data,
            "disk_io": disk_io_data,
            "network_throughput": network_throughput_data,
            "processing_health": processing_health_data,
            "system_state": system_state_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"DPI Probe health error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/linux-agent-health')
def dashboard_linux_agent_health():
    """
    Linux Agent Health Dashboard Endpoint.
    Returns operational health and liveness metrics for Linux agents.
    Returns 204 No Content if no data exists.
    
    SQL Query (EXACT as per spec):
    WITH latest AS (
      SELECT DISTINCT ON (agent_id)
        agent_id,
        observed_at,
        payload->'system' AS system
      FROM linux_agent_telemetry
      WHERE payload ? 'system'
        AND payload->'system' != '{}'::jsonb
        AND observed_at > NOW() - INTERVAL '10 minutes'
      ORDER BY agent_id, observed_at DESC
    )
    SELECT
      agent_id,
      observed_at,
      system->'cpu' AS cpu,
      system->'memory' AS memory,
      system->'disk' AS disk,
      system->'network' AS network,
      system->'system_state' AS system_state
    FROM latest;
    """
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Get query parameters
        agent_id = request.args.get('agent_id', None)
        since_minutes = int(request.args.get('since_minutes', 10))
        
        # Check if table exists
        if not db.table_exists("ransomeye", "linux_agent_telemetry"):
            cursor.close()
            conn.close()
            return Response(status=204)
        
        # Check if required columns exist
        required_cols = ["agent_id", "observed_at", "payload"]
        for col in required_cols:
            if not db.column_exists("ransomeye", "linux_agent_telemetry", col):
                cursor.close()
                conn.close()
                return Response(status=204)
        
        # Build exact SQL query as per spec (no Python-side filtering)
        # Agent_id filter is handled in SQL WHERE clause if provided
        if agent_id:
            # When agent_id is provided, add it to WHERE clause in SQL
            base_query = """
                WITH latest AS (
                  SELECT DISTINCT ON (agent_id)
                    agent_id,
                    observed_at,
                    payload->'system' AS system
                  FROM linux_agent_telemetry
                  WHERE payload ? 'system'
                    AND payload->'system' != '{}'::jsonb
                    AND observed_at > NOW() - INTERVAL '%s minutes'
                    AND agent_id::text = %s
                  ORDER BY agent_id, observed_at DESC
                )
                SELECT
                  agent_id,
                  observed_at,
                  system->'cpu' AS cpu,
                  system->'memory' AS memory,
                  system->'disk' AS disk,
                  system->'network' AS network,
                  system->'system_state' AS system_state
                FROM latest
            """ % (since_minutes, agent_id)
            params = ()
        else:
            # Exact SQL as per spec (no agent_id filter)
            base_query = """
                WITH latest AS (
                  SELECT DISTINCT ON (agent_id)
                    agent_id,
                    observed_at,
                    payload->'system' AS system
                  FROM linux_agent_telemetry
                  WHERE payload ? 'system'
                    AND payload->'system' != '{}'::jsonb
                    AND observed_at > NOW() - INTERVAL '%s minutes'
                  ORDER BY agent_id, observed_at DESC
                )
                SELECT
                  agent_id,
                  observed_at,
                  system->'cpu' AS cpu,
                  system->'memory' AS memory,
                  system->'disk' AS disk,
                  system->'network' AS network,
                  system->'system_state' AS system_state
                FROM latest
            """ % since_minutes
            params = ()
        
        # Execute query
        cursor.execute(base_query, params)
        rows = cursor.fetchall()
        
        # If zero rows, return 204 No Content
        if not rows or len(rows) == 0:
            cursor.close()
            conn.close()
            return Response(status=204)
        
        # Process results
        import json as json_lib
        agents_data = []
        current_time = datetime.now(timezone.utc)
        
        for row in rows:
            agent_id_val = str(row[0]) if row[0] else None
            observed_at_val = row[1]
            cpu_json = row[2]
            memory_json = row[3]
            disk_json = row[4]
            network_json = row[5]
            system_state_json = row[6]
            
            # Calculate status based on time since last observation
            if observed_at_val:
                time_diff = (current_time - observed_at_val).total_seconds() / 60.0
                if time_diff < 2:
                    status = "ONLINE"
                elif time_diff < 5:
                    status = "DEGRADED"
                else:
                    status = "OFFLINE"
            else:
                status = "OFFLINE"
            
            # Parse JSONB fields (they come as dict or None)
            cpu_data = cpu_json if isinstance(cpu_json, dict) else {}
            memory_data = memory_json if isinstance(memory_json, dict) else {}
            disk_data = disk_json if isinstance(disk_json, dict) else {}
            network_data = network_json if isinstance(network_json, dict) else {}
            system_state_data = system_state_json if isinstance(system_state_json, dict) else {}
            
            agent_entry = {
                "agent_id": agent_id_val,
                "observed_at": observed_at_val.isoformat() if hasattr(observed_at_val, 'isoformat') else str(observed_at_val),
                "status": status,
                "cpu": {
                    "utilization": cpu_data.get("utilization"),
                    "core_count": cpu_data.get("core_count"),
                    "agent_process_cpu": cpu_data.get("agent_process_cpu")
                },
                "memory": {
                    "total": memory_data.get("total"),
                    "used": memory_data.get("used"),
                    "free": memory_data.get("free"),
                    "agent_rss": memory_data.get("agent_process_rss")
                },
                "disk": {
                    "read_bytes": disk_data.get("read_bytes"),
                    "write_bytes": disk_data.get("write_bytes"),
                    "read_iops": disk_data.get("read_iops"),
                    "write_iops": disk_data.get("write_iops")
                },
                "network": {
                    "bytes_in": network_data.get("bytes_in"),
                    "bytes_out": network_data.get("bytes_out"),
                    "drops": network_data.get("drops"),
                    "errors": network_data.get("errors")
                },
                "system_state": {
                    "host_uptime": system_state_data.get("host_uptime"),
                    "process_count": system_state_data.get("process_count"),
                    "agent_process_status": system_state_data.get("agent_process_status")
                }
            }
            agents_data.append(agent_entry)
        
        cursor.close()
        conn.close()
        
        # Structure response for panels
        return jsonify({
            "agent_liveness": agents_data,
            "cpu_health": agents_data,
            "memory_health": agents_data,
            "disk_io_health": agents_data,
            "network_health": agents_data,
            "system_state": agents_data,
            "timestamp": current_time.isoformat()
        })

    except Exception as e:
        logger.error(f"Linux Agent Health dashboard error: {e}", exc_info=True)
        if conn:
            try:
                cursor.close()
                conn.close()
            except:
                pass
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/api/dashboards/sensor-coverage')
def dashboard_sensor_coverage():
    """
    Sensor Coverage Dashboard Endpoint.
    Returns which telemetry sensors are ACTIVE per Linux agent.
    Returns 204 No Content if no data exists.

    SQL Query (EXACT as per spec):
    WITH recent AS (
      SELECT
        agent_id,
        payload
      FROM linux_agent_telemetry
      WHERE observed_at > NOW() - INTERVAL '10 minutes'
    )
    SELECT
      agent_id,
      BOOL_OR(payload->>'event_category' = 'process')        AS process_sensor,
      BOOL_OR(payload ? 'filesystem_data')                   AS filesystem_sensor,
      BOOL_OR(payload ? 'network_data')                      AS network_sensor,
      BOOL_OR(payload ? 'system' AND payload->'system' != '{}'::jsonb) AS system_sensor,
      BOOL_OR(payload->>'event_category' = 'deception')      AS deception_sensor
    FROM recent
    GROUP BY agent_id;
    """
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503

    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")

        # Get query parameters
        since_minutes = int(request.args.get('since_minutes', 10))

        # Check if table exists
        if not db.table_exists("ransomeye", "linux_agent_telemetry"):
            cursor.close()
            conn.close()
            return Response(status=204)

        # Check if required columns exist
        required_cols = ["agent_id", "observed_at", "payload"]
        for col in required_cols:
            if not db.column_exists("ransomeye", "linux_agent_telemetry", col):
                cursor.close()
                conn.close()
                return Response(status=204)

        # Build exact SQL query as per spec (no Python-side filtering)
        base_query = """
            WITH recent AS (
              SELECT
                agent_id,
                payload
              FROM linux_agent_telemetry
              WHERE observed_at > NOW() - INTERVAL '%s minutes'
            )
            SELECT
              agent_id,
              BOOL_OR(payload->>'event_category' = 'process')        AS process_sensor,
              BOOL_OR(payload ? 'filesystem_data' AND payload->'filesystem_data' != 'null'::jsonb) AS filesystem_sensor,
              BOOL_OR(payload ? 'network_data' AND payload->'network_data' != 'null'::jsonb) AS network_sensor,
              BOOL_OR(payload ? 'system' AND payload->'system' != '{}'::jsonb) AS system_sensor,
              BOOL_OR(payload->>'event_category' = 'deception')      AS deception_sensor
            FROM recent
            GROUP BY agent_id
        """ % since_minutes

        # Execute query
        cursor.execute(base_query)
        rows = cursor.fetchall()

        # If zero rows, return 204 No Content
        if not rows or len(rows) == 0:
            cursor.close()
            conn.close()
            return Response(status=204)

        # Process results
        sensor_coverage_data = []
        for row in rows:
            agent_id_val = str(row[0]) if row[0] else None
            process_sensor = bool(row[1]) if row[1] is not None else False
            filesystem_sensor = bool(row[2]) if row[2] is not None else False
            network_sensor = bool(row[3]) if row[3] is not None else False
            system_sensor = bool(row[4]) if row[4] is not None else False
            deception_sensor = bool(row[5]) if row[5] is not None else False

            agent_entry = {
                "agent_id": agent_id_val,
                "process_sensor": process_sensor,
                "filesystem_sensor": filesystem_sensor,
                "network_sensor": network_sensor,
                "system_sensor": system_sensor,
                "deception_sensor": deception_sensor
            }
            sensor_coverage_data.append(agent_entry)

        # Calculate coverage summary
        total_agents = len(sensor_coverage_data)
        if total_agents == 0:
            cursor.close()
            conn.close()
            return Response(status=204)

        # Count agents with full coverage (all 5 sensors active)
        full_coverage_count = 0
        missing_sensors_count = 0

        for agent in sensor_coverage_data:
            sensors_active = sum([
                agent["process_sensor"],
                agent["filesystem_sensor"],
                agent["network_sensor"],
                agent["system_sensor"],
                agent["deception_sensor"]
            ])
            if sensors_active == 5:
                full_coverage_count += 1
            if sensors_active < 5:
                missing_sensors_count += 1

        full_coverage_percent = (full_coverage_count / total_agents * 100) if total_agents > 0 else 0
        missing_sensors_percent = (missing_sensors_count / total_agents * 100) if total_agents > 0 else 0

        cursor.close()
        conn.close()

        current_time = datetime.now(timezone.utc)

        # Structure response for panels
        return jsonify({
            "sensor_coverage_matrix": sensor_coverage_data,
            "coverage_summary": {
                "total_agents": total_agents,
                "full_coverage_count": full_coverage_count,
                "full_coverage_percent": round(full_coverage_percent, 2),
                "missing_sensors_count": missing_sensors_count,
                "missing_sensors_percent": round(missing_sensors_percent, 2)
            },
            "timestamp": current_time.isoformat()
        })

    except Exception as e:
        logger.error(f"Error in sensor coverage endpoint: {e}", exc_info=True)
        if conn:
            try:
                cursor.close()
                conn.close()
            except:
                pass
        return jsonify({"error": "Internal server error"}), 500


@app.route('/api/dashboards/db-health')
def dashboard_db_health():
    """PostgreSQL database health metrics - fail-soft."""
    # Get db_instance_id from query parameter (for future HA support)
    db_instance_id = request.args.get('db_instance_id', None)
    
    # For now, all DB health queries target the current connection
    # In HA setups, this would route to specific DB instance
    # If instance_id specified but not found, we'll return 404
    if db_instance_id and db_instance_id != "db-01":
        # In future, validate instance_id against known DB instances
        # For now, only db-01 is supported
        return jsonify({"error": f"DB instance '{db_instance_id}' not found"}), 404
    
    conn = get_db_connection()
    if not conn:
        return jsonify({"error": "Database unavailable"}), 503
    
    try:
        db = SchemaAwareDB(conn)
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # Connection health
        connections_data = {"active": "Metric unavailable", "idle": "Metric unavailable",
                           "max": "Metric unavailable", "utilization": "Metric unavailable"}
        
        # Query performance
        query_perf_data = {"transactions_per_sec": "Metric unavailable", 
                          "long_running_queries": "Metric unavailable",
                          "slow_queries": "Metric unavailable", "deadlocks": "Metric unavailable"}
        
        # Database I/O
        db_io_data = {"blocks_read": "Metric unavailable", "blocks_hit": "Metric unavailable",
                     "cache_hit_ratio": "Metric unavailable"}
        
        # Replication
        replication_data = {"lag": "Metric unavailable", "replica_state": "Metric unavailable",
                           "configured": False}
        
        # Storage
        storage_data = {"database_size": "Metric unavailable", "table_bloat": "Metric unavailable",
                       "index_bloat": "Metric unavailable"}
        
        # Reliability
        reliability_data = {"checkpoint_frequency": "Metric unavailable", 
                           "wal_generation_rate": "Metric unavailable",
                           "last_vacuum": "Metric unavailable", "last_autovacuum": "Metric unavailable"}
        
        # Get connection stats from pg_stat_activity
        try:
            cursor.execute("""
                SELECT 
                    COUNT(*) FILTER (WHERE state = 'active') as active,
                    COUNT(*) FILTER (WHERE state = 'idle') as idle,
                    (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_conns
                FROM pg_stat_activity
                WHERE datname = current_database()
            """)
            conn_row = cursor.fetchone()
            if conn_row:
                active = conn_row[0] or 0
                idle = conn_row[1] or 0
                max_conns = conn_row[2] or 100
                utilization = round((active + idle) / max_conns * 100, 2) if max_conns > 0 else 0
                connections_data = {
                    "active": active,
                    "idle": idle,
                    "max": max_conns,
                    "utilization": utilization
                }
        except Exception as e:
            logger.debug(f"Connection stats query failed: {e}")
        
        # Get query performance from pg_stat_database
        try:
            cursor.execute("""
                SELECT 
                    xact_commit + xact_rollback as total_transactions,
                    deadlocks
                FROM pg_stat_database
                WHERE datname = current_database()
            """)
            db_row = cursor.fetchone()
            if db_row:
                # Calculate transactions per second (approximate from last stats reset)
                total_xacts = db_row[0] or 0
                deadlocks = db_row[1] or 0
                query_perf_data["deadlocks"] = deadlocks
                # Note: transactions_per_sec would require time-based calculation
        except Exception as e:
            logger.debug(f"Query performance stats failed: {e}")
        
        # Get long-running queries
        try:
            cursor.execute("""
                SELECT COUNT(*) FROM pg_stat_activity
                WHERE datname = current_database()
                AND state = 'active'
                AND now() - query_start > interval '5 seconds'
            """)
            long_running = cursor.fetchone()[0] or 0
            query_perf_data["long_running_queries"] = long_running
        except Exception as e:
            logger.debug(f"Long-running queries query failed: {e}")
        
        # Get database I/O from pg_stat_database
        try:
            cursor.execute("""
                SELECT 
                    blks_read,
                    blks_hit,
                    CASE 
                        WHEN (blks_hit + blks_read) > 0 
                        THEN round(blks_hit::numeric / (blks_hit + blks_read) * 100, 2)
                        ELSE 0
                    END as cache_hit_ratio
                FROM pg_stat_database
                WHERE datname = current_database()
            """)
            io_row = cursor.fetchone()
            if io_row:
                db_io_data = {
                    "blocks_read": io_row[0] or 0,
                    "blocks_hit": io_row[1] or 0,
                    "cache_hit_ratio": float(io_row[2]) if io_row[2] is not None else 0
                }
        except Exception as e:
            logger.debug(f"Database I/O stats failed: {e}")
        
        # Check replication (fail-soft if not configured)
        try:
            cursor.execute("""
                SELECT 
                    CASE WHEN pg_is_in_recovery() THEN 'replica' ELSE 'primary' END as role,
                    pg_last_wal_replay_lsn() as replay_lsn
            """)
            repl_row = cursor.fetchone()
            if repl_row:
                replication_data["configured"] = True
                replication_data["replica_state"] = repl_row[0] if repl_row[0] else "primary"
                # Replication lag calculation would require primary LSN comparison
                replication_data["lag"] = "Not applicable" if repl_row[0] == "primary" else "Metric unavailable"
        except Exception as e:
            logger.debug(f"Replication check failed (may not be configured): {e}")
            replication_data["configured"] = False
            replication_data["replica_state"] = "Not configured"
        
        # Get storage size
        try:
            cursor.execute("""
                SELECT pg_size_pretty(pg_database_size(current_database())) as db_size
            """)
            size_row = cursor.fetchone()
            if size_row:
                storage_data["database_size"] = size_row[0] if size_row[0] else "Metric unavailable"
        except Exception as e:
            logger.debug(f"Storage size query failed: {e}")
        
        # Get reliability metrics from pg_stat_bgwriter
        try:
            cursor.execute("""
                SELECT 
                    checkpoints_timed + checkpoints_req as total_checkpoints,
                    checkpoint_write_time
                FROM pg_stat_bgwriter
            """)
            bgwriter_row = cursor.fetchone()
            if bgwriter_row:
                total_checkpoints = bgwriter_row[0] or 0
                reliability_data["checkpoint_frequency"] = total_checkpoints
        except Exception as e:
            logger.debug(f"Background writer stats failed: {e}")
        
        # Get WAL generation (approximate)
        try:
            cursor.execute("""
                SELECT pg_current_wal_lsn() as current_lsn
            """)
            wal_row = cursor.fetchone()
            if wal_row:
                # WAL generation rate would require time-based calculation
                reliability_data["wal_generation_rate"] = "Metric unavailable"
        except Exception as e:
            logger.debug(f"WAL stats failed: {e}")
        
        # Get last vacuum/autovacuum from pg_stat_user_tables (sample from ransomeye schema)
        try:
            cursor.execute("""
                SELECT 
                    MAX(last_vacuum) as last_vacuum,
                    MAX(last_autovacuum) as last_autovacuum
                FROM pg_stat_user_tables
                WHERE schemaname = 'ransomeye'
            """)
            vacuum_row = cursor.fetchone()
            if vacuum_row:
                reliability_data["last_vacuum"] = vacuum_row[0].isoformat() if vacuum_row[0] else "Never"
                reliability_data["last_autovacuum"] = vacuum_row[1].isoformat() if vacuum_row[1] else "Never"
        except Exception as e:
            logger.debug(f"Vacuum stats failed: {e}")
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "connections": connections_data,
            "query_performance": query_perf_data,
            "database_io": db_io_data,
            "replication": replication_data,
            "storage": storage_data,
            "reliability": reliability_data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })
    except Exception as e:
        logger.error(f"Database health error: {e}", exc_info=True)
        if conn:
            conn.close()
        return jsonify({"error": "Metric unavailable"}), 500


@app.route('/logo.png')
def logo():
    """Serve RansomEye logo."""
    if LOGO_PATH.exists():
        return send_from_directory(LOGO_PATH.parent, LOGO_PATH.name)
    return jsonify({"error": "Logo not found"}), 404


@app.route('/static/<path:filename>')
def serve_static(filename):
    """Serve static files (CSS, JS, etc.)."""
    return send_from_directory(STATIC_DIR, filename)


# ============================================================================
# UI SETTINGS API ENDPOINTS
# ============================================================================

@app.route('/api/ui/settings', methods=['GET'])
def get_ui_settings():
    """
    Get UI settings for current user.
    
    Returns:
        JSON with theme, density, font_size (defaults applied if missing)
    """
    conn = get_db_connection()
    if not conn:
        # Fail-soft: return defaults if DB unavailable
        from settings import get_default_settings
        return jsonify(get_default_settings())
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        settings_manager = SettingsManager(conn)
        settings = settings_manager.get_settings()
        
        cursor.close()
        conn.close()
        
        return jsonify(settings)
    
    except Exception as e:
        logger.error(f"Error getting UI settings: {e}", exc_info=True)
        if conn:
            try:
                conn.close()
            except:
                pass
        # Fail-soft: return defaults on error
        from settings import get_default_settings
        return jsonify(get_default_settings())


@app.route('/api/ui/share-policy', methods=['GET'])
def get_share_policy_info():
    """
    Get share policy configuration for UI display.
    
    Returns:
        JSON with policy settings:
        - enabled: bool (whether sharing is enabled)
        - default_expiry_days: int or null (default expiry in days)
        - max_expiry_days: int or null (maximum allowed expiry in days)
    """
    policy = get_share_policy()
    return jsonify({
        "enabled": policy['enabled'],
        "default_expiry_days": policy['default_expiry_days'],
        "max_expiry_days": policy['max_expiry_days']
    })


@app.route('/api/dashboards/<dashboard_name>/share', methods=['POST'])
def create_dashboard_share(dashboard_name: str):
    """
    Create a read-only share link for a personal dashboard.
    
    Request body (JSON, optional):
        {
            "expires_in_days": 30  # Optional, number of days until expiration (None = no expiration)
        }
    
    Rules:
    - Only personal dashboards can be shared (fail-closed if system dashboard)
    - Generates cryptographically strong token
    - Optional expiration (configurable via env or request)
    - Policy enforcement: sharing enabled/disabled, default/max expiry limits
    - Audit-log share creation and policy denials
    - Returns share link and token info
    
    Returns:
        JSON with share token, share link URL, and metadata
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Get share policy
    policy = get_share_policy()
    
    # Fail-closed: Check if sharing is enabled
    if not policy['enabled']:
        # Audit log policy denial
        conn = get_db_connection()
        if conn:
            try:
                cursor = conn.cursor()
                cursor.execute("SET search_path = ransomeye, public;")
                share_manager = ShareManager(conn)
                share_manager._audit_log('create_share', user_id, dashboard_name, 
                                       success=False, error="policy_denied:sharing_disabled")
                cursor.close()
                conn.close()
            except Exception as e:
                logger.error(f"Error audit logging policy denial: {e}", exc_info=True)
                if conn:
                    try:
                        conn.rollback()
                        conn.close()
                    except:
                        pass
        
        return jsonify({
            "error": "Dashboard sharing is disabled by policy",
            "status": "failed"
        }), 403
    
    # Check if dashboard is personal (has overlay)
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can be shared
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot be shared"
        }), 403
    
    # Validate dashboard exists
    dashboard = dashboard_engine.load_dashboard(dashboard_name)
    if not dashboard:
        return jsonify({"error": f"Dashboard '{dashboard_name}' not found"}), 404
    
    # Parse optional expiration
    expires_in_days = None
    if request.is_json:
        data = request.get_json()
        if data and 'expires_in_days' in data:
            expires_in_days = data.get('expires_in_days')
            if expires_in_days is not None:
                try:
                    expires_in_days = int(expires_in_days)
                    if expires_in_days <= 0:
                        expires_in_days = None
                except (ValueError, TypeError):
                    expires_in_days = None
    
    # Validate expiry against policy (enforces default and max limits)
    expires_in_days, expiry_error = validate_expiry_days(expires_in_days, policy)
    if expiry_error:
        # Audit log policy denial
        conn = get_db_connection()
        if conn:
            try:
                cursor = conn.cursor()
                cursor.execute("SET search_path = ransomeye, public;")
                share_manager = ShareManager(conn)
                share_manager._audit_log('create_share', user_id, dashboard_name, 
                                       success=False, error=f"policy_denied:{expiry_error}")
                cursor.close()
                conn.close()
            except Exception as e:
                logger.error(f"Error audit logging policy denial: {e}", exc_info=True)
                if conn:
                    try:
                        conn.rollback()
                        conn.close()
                    except:
                        pass
        
        return jsonify({
            "error": expiry_error,
            "status": "failed"
        }), 400
    
    # Create share token
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        share_info = share_manager.create_share(
            dashboard_name=dashboard_name,
            owner_user_id=user_id,
            expires_in_days=expires_in_days
        )
        
        cursor.close()
        conn.close()
        
        if not share_info:
            return jsonify({
                "error": "Failed to create share token",
                "status": "failed"
            }), 500
        
        # Build share link URL
        share_link = f"/share/{share_info['token']}"
        
        return jsonify({
            "status": "success",
            "share": {
                "token": share_info['token'],
                "share_link": share_link,
                "dashboard_name": share_info['dashboard_name'],
                "permissions": share_info['permissions'],
                "expires_at": share_info['expires_at'],
                "created_at": share_info['created_at'],
                "access_count": share_info['access_count']
            }
        }), 201
        
    except Exception as e:
        logger.error(f"Error creating share: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/share/<token>')
def share_dashboard_view(token: str):
    """
    Serve read-only dashboard view via share token.
    
    Rules:
    - Rate limiting applied per token (configurable via env)
    - Validates token (expiry, revocation)
    - Emergency kill-switch: returns 410 Gone if RANSOMEYE_SHARE_EMERGENCY_DISABLE=true
    - Fail-closed on invalid/expired/revoked tokens
    - Fail-closed on rate limit abuse (429)
    - Captures access metadata (IP, User-Agent) best-effort
    - Logs all access attempts to share_access_logs (append-only)
    - Serves read-only template (no edit/save capabilities)
    - Tracks access count and timestamp
    - Audit-log access event with rate-limited flag and metadata presence
    """
    # Check emergency kill-switch first (before any DB access)
    conn = get_db_connection()
    if conn:
        try:
            cursor = conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            share_manager = ShareManager(conn)
            
            if share_manager.is_emergency_disabled():
                # Try to get token info for audit logging
                try:
                    query = """
                        SELECT token_id, dashboard_name, owner_user_id
                        FROM dashboard_share_tokens
                        WHERE token = %s
                    """
                    cursor.execute(query, (token,))
                    row = cursor.fetchone()
                    
                    if row:
                        share_manager._audit_log('access_share', row[2], row[1], 
                                              success=False, error="emergency_disabled:true")
                except:
                    pass
                
                cursor.close()
                conn.close()
                logger.warning(f"Share access denied due to emergency disable: {token[:16]}...")
                return render_template('share_error.html', 
                                     error="This share link has been disabled due to an emergency."), 410
        except Exception as e:
            logger.error(f"Error checking emergency disable: {e}", exc_info=True)
            if conn:
                try:
                    conn.close()
                except:
                    pass
    
    # Get rate limiter
    rate_limiter = get_rate_limiter()
    
    # Check rate limit (before database access for efficiency)
    is_allowed, rate_limit_reason = rate_limiter.is_allowed(token)
    
    # Capture access metadata (best-effort)
    ip_address = None
    user_agent = None
    
    # Get IP address (best-effort, may be None behind proxies)
    if request.headers.get('X-Forwarded-For'):
        # Use first IP in X-Forwarded-For chain
        ip_address = request.headers.get('X-Forwarded-For').split(',')[0].strip()
    elif request.headers.get('X-Real-Ip'):
        ip_address = request.headers.get('X-Real-Ip')
    else:
        ip_address = request.remote_addr
    
    # Get User-Agent (best-effort)
    user_agent = request.headers.get('User-Agent')
    
    # Validate token
    conn = get_db_connection()
    if not conn:
        return render_template('share_error.html', error="Database unavailable"), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        
        # Check token status first to determine if expired (for 410 response)
        # Query token to check expiry status
        from psycopg2.extras import RealDictCursor
        status_cursor = conn.cursor(cursor_factory=RealDictCursor)
        status_cursor.execute("SET search_path = ransomeye, public;")
        status_query = """
            SELECT expires_at, revoked_at
            FROM dashboard_share_tokens
            WHERE token = %s
        """
        status_cursor.execute(status_query, (token,))
        status_row = status_cursor.fetchone()
        status_cursor.close()
        
        # If token exists and is expired (not revoked), return 410 Gone
        if status_row and status_row['revoked_at'] is None:
            if status_row['expires_at']:
                if status_row['expires_at'] < datetime.now(timezone.utc):
                    # Token is expired - return 410 Gone
                    # Still log the access attempt via validate_token
                    share_manager.validate_token(
                        token=token,
                        ip_address=ip_address,
                        user_agent=user_agent,
                        rate_limited=not is_allowed
                    )
                    conn.close()
                    logger.warning(f"Expired share token access attempt: {token[:16]}... (IP: {ip_address})")
                    return render_template('share_error.html', 
                                         error="This share link has expired."), 410
        
        # Validate token with metadata and rate-limited flag
        share_info = share_manager.validate_token(
            token=token,
            ip_address=ip_address,
            user_agent=user_agent,
            rate_limited=not is_allowed
        )
        
        cursor.close()
        conn.close()
        
        # If rate-limited, return 429
        if not is_allowed:
            logger.warning(f"Rate limit exceeded for share token: {token[:16]}... (IP: {ip_address})")
            return render_template('share_error.html', 
                                 error="Rate limit exceeded. Please try again later."), 429
        
        if not share_info:
            return render_template('share_error.html', error="Invalid share link"), 404
        
        # Load dashboard
        dashboard = dashboard_engine.load_dashboard(share_info['dashboard_name'], include_overlay=True)
        if not dashboard:
            return render_template('share_error.html', error="Dashboard not found"), 404
        
        # Render read-only template
        return render_template('dashboard_share.html', 
                             dashboard=dashboard, 
                             share_info=share_info)
        
    except Exception as e:
        logger.error(f"Error serving share view: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return render_template('share_error.html', error="Internal server error"), 500


@app.route('/api/shares/<token>/rotate', methods=['POST'])
def rotate_share(token: str):
    """
    Rotate a share token: revoke old token and create new one.
    
    Request body (JSON, optional):
        {
            "expires_in_days": 30  # Optional, new expiration in days (None = preserve old expiry)
        }
    
    Rules:
    - Only token owner can rotate (fail-closed on unauthorized)
    - Old token immediately revoked
    - New token created with same permissions
    - Expiry preserved unless overridden
    - Policy enforcement: sharing enabled/disabled, default/max expiry limits
    - Audit-log rotation (old_token_id → new_token_id) and policy denials
    - Fail-closed on validation errors
    
    Returns:
        JSON with new share token info
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Get share policy
    policy = get_share_policy()
    
    # Fail-closed: Check if sharing is enabled
    if not policy['enabled']:
        # Audit log policy denial
        conn = get_db_connection()
        if conn:
            try:
                cursor = conn.cursor()
                cursor.execute("SET search_path = ransomeye, public;")
                share_manager = ShareManager(conn)
                share_manager._audit_log('rotate_share', user_id, 'unknown', 
                                       success=False, error="policy_denied:sharing_disabled")
                cursor.close()
                conn.close()
            except Exception as e:
                logger.error(f"Error audit logging policy denial: {e}", exc_info=True)
                if conn:
                    try:
                        conn.rollback()
                        conn.close()
                    except:
                        pass
        
        return jsonify({
            "error": "Dashboard sharing is disabled by policy",
            "status": "failed"
        }), 403
    
    # Parse optional new expiration
    expires_in_days = None
    if request.is_json:
        data = request.get_json()
        if data and 'expires_in_days' in data:
            expires_in_days = data.get('expires_in_days')
            if expires_in_days is not None:
                try:
                    expires_in_days = int(expires_in_days)
                    if expires_in_days <= 0:
                        expires_in_days = None
                except (ValueError, TypeError):
                    expires_in_days = None
    
    # If new expiry specified, validate against policy
    if expires_in_days is not None:
        expires_in_days, expiry_error = validate_expiry_days(expires_in_days, policy)
        if expiry_error:
            # Audit log policy denial
            conn = get_db_connection()
            if conn:
                try:
                    cursor = conn.cursor()
                    cursor.execute("SET search_path = ransomeye, public;")
                    share_manager = ShareManager(conn)
                    share_manager._audit_log('rotate_share', user_id, 'unknown', 
                                           success=False, error=f"policy_denied:{expiry_error}")
                    cursor.close()
                    conn.close()
                except Exception as e:
                    logger.error(f"Error audit logging policy denial: {e}", exc_info=True)
                    if conn:
                        try:
                            conn.rollback()
                            conn.close()
                        except:
                            pass
            
            return jsonify({
                "error": expiry_error,
                "status": "failed"
            }), 400
    
    # Rotate token
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        new_share_info = share_manager.rotate_token(token, user_id, expires_in_days)
        
        cursor.close()
        conn.close()
        
        if not new_share_info:
            return jsonify({
                "error": "Token not found or unauthorized",
                "status": "failed"
            }), 404
        
        # Build share link URL
        share_link = f"/share/{new_share_info['token']}"
        
        return jsonify({
            "status": "success",
            "message": "Share token rotated successfully",
            "share": {
                "token": new_share_info['token'],
                "share_link": share_link,
                "dashboard_name": new_share_info['dashboard_name'],
                "permissions": new_share_info['permissions'],
                "expires_at": new_share_info['expires_at'],
                "created_at": new_share_info['created_at'],
                "access_count": new_share_info['access_count']
            }
        })
        
    except Exception as e:
        logger.error(f"Error rotating share: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/shares/<token>', methods=['DELETE'])
def revoke_share(token: str):
    """
    Revoke a share token.
    
    Rules:
    - Only token owner can revoke
    - Soft delete (sets revoked_at timestamp)
    - Audit-log revocation
    - Fail-closed on unauthorized access
    
    Returns:
        JSON with success status
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Revoke token
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        success = share_manager.revoke_token(token, user_id)
        
        cursor.close()
        conn.close()
        
        if not success:
            return jsonify({
                "error": "Token not found or unauthorized",
                "status": "failed"
            }), 404
        
        return jsonify({
            "status": "success",
            "message": "Share token revoked successfully"
        })
        
    except Exception as e:
        logger.error(f"Error revoking share: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/dashboards/<dashboard_name>/shares', methods=['GET'])
def list_dashboard_shares(dashboard_name: str):
    """
    List all active shares for a dashboard.
    
    Rules:
    - Only dashboard owner can list shares
    - Returns active (non-revoked) shares only
    - Includes access counts and timestamps
    
    Returns:
        JSON array of share info
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    # Check if dashboard is personal (has overlay)
    source_info = dashboard_engine.get_dashboard_source(dashboard_name)
    
    # Fail-closed: Only personal dashboards can have shares
    if not source_info.get('has_overlay'):
        return jsonify({
            "error": f"Dashboard '{dashboard_name}' is a system dashboard and cannot have shares"
        }), 403
    
    # List shares
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        shares = share_manager.list_shares(dashboard_name, user_id)
        
        cursor.close()
        conn.close()
        
        # Build share links
        for share in shares:
            share['share_link'] = f"/share/{share['token']}"
        
        return jsonify({
            "status": "success",
            "shares": shares,
            "count": len(shares)
        })
        
    except Exception as e:
        logger.error(f"Error listing shares: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/shares/activity')
def share_activity_view():
    """Serve Share Activity & Audit Dashboard page (read-only)."""
    return render_template('share_activity.html')


@app.route('/shares/incident-report')
def incident_report_view():
    """Serve Share Incident Report page (forensic, read-only)."""
    return render_template('incident_report.html')


@app.route('/api/shares/activity', methods=['GET'])
def get_share_activity():
    """
    Get all dashboard share activity (read-only audit view).
    
    Returns:
        JSON array of share tokens with:
        - dashboard_name
        - status (active / expired / revoked)
        - created_at
        - expires_at
        - access_count
        - last_accessed_at
        - owner_user_id
    - Sorted by last_accessed_at DESC (or created_at if never accessed)
    - Fail-soft if no shares exist (returns empty array)
    """
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        shares = share_manager.get_all_share_activity()
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "status": "success",
            "shares": shares,
            "count": len(shares)
        })
        
    except Exception as e:
        logger.error(f"Error getting share activity: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/shares/revoke_all', methods=['POST'])
def revoke_all_shares():
    """
    Revoke all active share tokens owned by the current user (mass revocation).
    
    Rules:
    - Only revokes tokens owned by current user (owner-scoped)
    - Atomic operation (all or nothing)
    - Idempotent (safe to re-run)
    - Emergency disable check (fail-closed if emergency active)
    - Audit-log mass revocation
    - Returns summary with revoked_count and already_revoked_count
    
    Returns:
        JSON with revocation summary
    """
    # Get user ID
    user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
    
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        result = share_manager.revoke_all_shares(user_id)
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "status": "success",
            "revoked_count": result['revoked_count'],
            "already_revoked_count": result['already_revoked_count'],
            "message": f"Revoked {result['revoked_count']} share token(s). "
                      f"{result['already_revoked_count']} token(s) were already revoked."
        })
        
    except Exception as e:
        logger.error(f"Error in mass revocation: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/shares/cleanup', methods=['POST'])
def cleanup_expired_shares():
    """
    Background-safe cleanup helper: check expired tokens (env-gated).
    
    Rules:
    - Only enabled if RANSOMEYE_SHARE_CLEANUP_ENABLED=true
    - Read-only operation (no deletion, status computed dynamically)
    - Returns counts of expired tokens
    
    Returns:
        JSON with cleanup statistics
    """
    # Check if cleanup is enabled via env
    cleanup_enabled = os.environ.get('RANSOMEYE_SHARE_CLEANUP_ENABLED', 'false').lower() == 'true'
    if not cleanup_enabled:
        return jsonify({
            "error": "Share cleanup is disabled",
            "status": "disabled"
        }), 403
    
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        cleanup_stats = share_manager.cleanup_expired_tokens()
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "status": "success",
            "cleanup": cleanup_stats
        })
        
    except Exception as e:
        logger.error(f"Error in cleanup_expired_shares: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/shares/incident-report', methods=['GET'])
def get_incident_report():
    """
    Generate a forensic, read-only incident report for share activity in a time window.
    
    Query Params:
        from: ISO timestamp (required, e.g., "2024-01-01T00:00:00Z")
        to: ISO timestamp (required, e.g., "2024-01-31T23:59:59Z")
    
    Returns:
        JSON with:
        - summary: Dict with total_shares_created, total_shares_revoked, total_access_attempts,
                   total_rate_limited, total_expired
        - timeline: List of ordered events (create/access/rotate/revoke/deny) with timestamp,
                    dashboard_name, token_id (masked), outcome
        - top_dashboards: List of dashboards sorted by access count in window
    
    Rules:
        - Validate time range (fail-closed on invalid params)
        - Fail-soft if no data (returns empty structure)
        - Audit-log report generation
        - No mutations (read-only)
    """
    # Get query params
    from_str = request.args.get('from')
    to_str = request.args.get('to')
    
    # Fail-closed: Validate required params
    if not from_str or not to_str:
        return jsonify({
            "error": "Missing required query parameters: 'from' and 'to' (ISO timestamps)",
            "status": "failed"
        }), 400
    
    # Parse timestamps (fail-closed on invalid format)
    try:
        from_timestamp = datetime.fromisoformat(from_str.replace('Z', '+00:00'))
        to_timestamp = datetime.fromisoformat(to_str.replace('Z', '+00:00'))
        
        # Ensure timezone-aware
        if from_timestamp.tzinfo is None:
            from_timestamp = from_timestamp.replace(tzinfo=timezone.utc)
        if to_timestamp.tzinfo is None:
            to_timestamp = to_timestamp.replace(tzinfo=timezone.utc)
    except (ValueError, AttributeError) as e:
        return jsonify({
            "error": f"Invalid timestamp format: {str(e)}. Expected ISO format (e.g., '2024-01-01T00:00:00Z')",
            "status": "failed"
        }), 400
    
    # Validate time range (fail-closed: from must be before to)
    if from_timestamp >= to_timestamp:
        return jsonify({
            "error": "Invalid time range: 'from' must be before 'to'",
            "status": "failed"
        }), 400
    
    # Validate reasonable time range (fail-closed: max 1 year)
    max_range = timedelta(days=365)
    if (to_timestamp - from_timestamp) > max_range:
        return jsonify({
            "error": "Time range exceeds maximum allowed (365 days)",
            "status": "failed"
        }), 400
    
    # Get database connection
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        report = share_manager.get_incident_report(from_timestamp, to_timestamp)
        
        # Audit log report generation
        user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
        share_manager._audit_log('incident_report', user_id, 'all', 
                               success=True, 
                               error=f"from:{from_timestamp.isoformat()},to:{to_timestamp.isoformat()}")
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "status": "success",
            "report": report,
            "time_range": {
                "from": from_timestamp.isoformat(),
                "to": to_timestamp.isoformat()
            }
        })
        
    except Exception as e:
        logger.error(f"Error generating incident report: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        
        # Audit log failure (if connection still available)
        if conn:
            try:
                user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
                cursor = conn.cursor()
                cursor.execute("SET search_path = ransomeye, public;")
                share_manager = ShareManager(conn)
                share_manager._audit_log('incident_report', user_id, 'all', 
                                       success=False, error=str(e))
                cursor.close()
            except:
                pass
        
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/shares/evidence-pack', methods=['POST'])
def generate_evidence_pack():
    """
    Generate a cryptographically signed evidence pack for share activity in a time window.
    
    Request body (JSON):
        {
            "from": "2024-01-01T00:00:00Z",  # ISO timestamp (required)
            "to": "2024-01-31T23:59:59Z"     # ISO timestamp (required)
        }
    
    Returns:
        ZIP archive containing:
        - incident_summary.json: Summary statistics
        - incident_timeline.csv: Chronological event timeline
        - top_dashboards.csv: Top dashboards by access count
        - manifest.json: File hashes and cryptographic signature
    
    Rules:
        - Validate time range (fail-closed on invalid params)
        - Fail-closed on any generation or signing error
        - Audit-log evidence generation
        - No persistent storage (temp files only)
    """
    # Get request body
    if not request.is_json:
        return jsonify({
            "error": "Request must be JSON",
            "status": "failed"
        }), 400
    
    data = request.get_json()
    if not isinstance(data, dict):
        return jsonify({
            "error": "Request body must be a JSON object",
            "status": "failed"
        }), 400
    
    from_str = data.get('from')
    to_str = data.get('to')
    
    # Fail-closed: Validate required params
    if not from_str or not to_str:
        return jsonify({
            "error": "Missing required fields: 'from' and 'to' (ISO timestamps)",
            "status": "failed"
        }), 400
    
    # Parse timestamps (fail-closed on invalid format)
    try:
        from_timestamp = datetime.fromisoformat(from_str.replace('Z', '+00:00'))
        to_timestamp = datetime.fromisoformat(to_str.replace('Z', '+00:00'))
        
        # Ensure timezone-aware
        if from_timestamp.tzinfo is None:
            from_timestamp = from_timestamp.replace(tzinfo=timezone.utc)
        if to_timestamp.tzinfo is None:
            to_timestamp = to_timestamp.replace(tzinfo=timezone.utc)
    except (ValueError, AttributeError) as e:
        return jsonify({
            "error": f"Invalid timestamp format: {str(e)}. Expected ISO format (e.g., '2024-01-01T00:00:00Z')",
            "status": "failed"
        }), 400
    
    # Validate time range (fail-closed: from must be before to)
    if from_timestamp >= to_timestamp:
        return jsonify({
            "error": "Invalid time range: 'from' must be before 'to'",
            "status": "failed"
        }), 400
    
    # Validate reasonable time range (fail-closed: max 1 year)
    max_range = timedelta(days=365)
    if (to_timestamp - from_timestamp) > max_range:
        return jsonify({
            "error": "Time range exceeds maximum allowed (365 days)",
            "status": "failed"
        }), 400
    
    # Get database connection
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        share_manager = ShareManager(conn)
        zip_bytes, error_message = share_manager.generate_evidence_pack(from_timestamp, to_timestamp)
        
        if error_message:
            cursor.close()
            conn.close()
            return jsonify({
                "error": f"Failed to generate evidence pack: {error_message}",
                "status": "failed"
            }), 500
        
        # Audit log evidence generation
        user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
        share_manager._audit_log('evidence_pack', user_id, 'all', 
                               success=True, 
                               error=f"from:{from_timestamp.isoformat()},to:{to_timestamp.isoformat()}")
        
        cursor.close()
        conn.close()
        
        # Generate filename with timestamp range
        from_date = from_timestamp.strftime('%Y%m%d_%H%M%S')
        to_date = to_timestamp.strftime('%Y%m%d_%H%M%S')
        filename = f"ransomeye_evidence_pack_{from_date}_to_{to_date}.zip"
        
        # Return ZIP as downloadable file
        return send_file(
            BytesIO(zip_bytes),
            mimetype='application/zip',
            as_attachment=True,
            download_name=filename
        )
        
    except Exception as e:
        logger.error(f"Error generating evidence pack: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        
        # Audit log failure
        if conn:
            try:
                user_id = os.environ.get('RANSOMEYE_UI_USER_ID', 'system')
                cursor = conn.cursor()
                cursor.execute("SET search_path = ransomeye, public;")
                share_manager = ShareManager(conn)
                share_manager._audit_log('evidence_pack', user_id, 'all', 
                                       success=False, error=str(e))
                cursor.close()
            except:
                pass
        
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


@app.route('/api/ui/settings', methods=['POST'])
def update_ui_settings():
    """
    Update UI settings for current user.
    
    Request body (JSON):
        {
            "theme": "soc_dark" | "high_contrast" | "executive",
            "density": "compact" | "comfortable",
            "font_size": "small" | "medium" | "large"
        }
        
    All fields optional (partial updates supported).
    
    Returns:
        JSON with success status and updated settings
    """
    conn = get_db_connection()
    if not conn:
        return jsonify({
            "error": "Database unavailable",
            "status": "failed"
        }), 503
    
    try:
        # Parse request body
        if not request.is_json:
            return jsonify({
                "error": "Request must be JSON",
                "status": "failed"
            }), 400
        
        data = request.get_json()
        if not isinstance(data, dict):
            return jsonify({
                "error": "Request body must be a JSON object",
                "status": "failed"
            }), 400
        
        # Validate settings (fail-closed on invalid enum values)
        try:
            validated = validate_settings(data)
        except SettingsValidationError as e:
            return jsonify({
                "error": str(e),
                "status": "validation_failed"
            }), 400
        
        # Save settings
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        settings_manager = SettingsManager(conn)
        success, error_msg = settings_manager.save_settings(validated)
        
        if not success:
            cursor.close()
            conn.close()
            return jsonify({
                "error": error_msg or "Failed to save settings",
                "status": "failed"
            }), 500
        
        # Get updated settings (merged with defaults)
        updated_settings = settings_manager.get_settings()
        
        cursor.close()
        conn.close()
        
        return jsonify({
            "status": "success",
            "settings": updated_settings
        })
    
    except Exception as e:
        logger.error(f"Error updating UI settings: {e}", exc_info=True)
        if conn:
            try:
                conn.rollback()
                conn.close()
            except:
                pass
        return jsonify({
            "error": "Internal server error",
            "status": "failed"
        }), 500


if __name__ == '__main__':
    try:
        # FATAL startup log - unmissable
        logger.error("UI STARTUP: server.py reached, binding to 0.0.0.0:8081")
        
        # Enforce explicit binding - no conditional logic
        BIND_HOST = "0.0.0.0"
        BIND_PORT = 8081
        
        # Validate port range
        if BIND_PORT < 1 or BIND_PORT > 65535:
            logger.error(f"FAIL-CLOSED: Invalid port '{BIND_PORT}'. Must be 1-65535")
            sys.exit(1)
        
        # Log configuration for security audit
        logger.info("=" * 60)
        logger.info("RansomEye UI Server - Network Hardening Configuration")
        logger.info("=" * 60)
        logger.info(f"Bind Address: {BIND_HOST}:{BIND_PORT}")
        logger.info(f"Database: {DB_NAME}@{DB_HOST}:{DB_PORT}")
        logger.info(f"CORS Allowed Origins: {CORS_ORIGINS_LIST if CORS_ORIGINS_LIST else 'None (same-origin only)'}")
        logger.info(f"CORS Credentials: {CORS_CREDENTIALS}")
        logger.info(f"Proxy Trust: {TRUST_PROXY}")
        logger.info(f"Air-Gap Mode: {AIR_GAP_MODE}")
        logger.info("=" * 60)
        
        # Temporary startup log: DB connection user confirmation
        logger.info("UI DB connection initialized using user=%s", DB_USER)
        
        # Security warnings
        logger.warning("SECURITY: Binding to 0.0.0.0 exposes UI to all network interfaces. Ensure firewall rules are configured.")
        
        if not CORS_ORIGINS_LIST:
            logger.info("CORS: No allowed origins configured - same-origin only (most secure)")
        else:
            logger.info(f"CORS: Allowing origins: {', '.join(CORS_ORIGINS_LIST)}")
        
        if TRUST_PROXY:
            logger.warning("SECURITY: Proxy trust enabled - X-Forwarded-For headers will be trusted. Ensure proxy is trusted.")
        
        print(f"Starting RansomEye UI Server on {BIND_HOST}:{BIND_PORT}")
        print(f"SOC-Grade Schema-Safe Dashboard")
        print(f"Database: {DB_NAME}@{DB_HOST}:{DB_PORT}")
        print(f"Access from Windows: http://<server-ip>:{BIND_PORT}")
        print(f"CORS Origins: {CORS_ORIGINS_LIST if CORS_ORIGINS_LIST else 'Same-origin only'}")
        
        # Explicit binding - no Flask dev defaults, no reloader
        app.run(host=BIND_HOST, port=BIND_PORT, debug=False, use_reloader=False)
        
    except Exception as e:
        logger.error(f"FATAL: UI startup failed with exception: {e}", exc_info=True)
        import traceback
        logger.error(f"FATAL: Full traceback:\n{traceback.format_exc()}")
        sys.exit(1)
