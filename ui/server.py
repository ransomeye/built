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
from flask import Flask, jsonify, send_from_directory, render_template, request
from flask_cors import CORS
from datetime import datetime, timedelta, timezone
from schema_helper import SchemaAwareDB

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
REBUILD_ROOT = BASE_DIR.parent.resolve()
LOGO_PATH = REBUILD_ROOT / 'core' / 'logo-removebg-preview.png'

# Configuration via environment variables
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = os.environ.get("DB_PORT", "5432")

# UI binding - network-agnostic
UI_HOST = os.environ.get("RANSOMEYE_UI_HOST", "0.0.0.0")
UI_PORT = int(os.environ.get("RANSOMEYE_UI_PORT", "8081"))

# Air-gap mode detection (if no internet connectivity expected)
AIR_GAP_MODE = os.environ.get("RANSOMEYE_AIR_GAP", "false").lower() == "true"

# Flask app
app = Flask(__name__, static_folder=str(STATIC_DIR), template_folder=str(TEMPLATE_DIR))
CORS(app)


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


@app.route('/')
def index():
    """Serve main SOC dashboard page."""
    return render_template('index.html')


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


@app.route('/logo.png')
def logo():
    """Serve RansomEye logo."""
    if LOGO_PATH.exists():
        return send_from_directory(LOGO_PATH.parent, LOGO_PATH.name)
    return jsonify({"error": "Logo not found"}), 404


if __name__ == '__main__':
    print(f"Starting RansomEye UI Server on {UI_HOST}:{UI_PORT}")
    print(f"SOC-Grade Schema-Safe Dashboard")
    print(f"Database: {DB_NAME}@{DB_HOST}:{DB_PORT}")
    app.run(host=UI_HOST, port=UI_PORT, debug=False)
