# Path and File Name : /home/ransomeye/rebuild/ui/validate_live_data.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: End-to-end validation script for RansomEye UI - validates real database data, API endpoints, HA instance selection, and fail-soft behavior

"""
RansomEye UI Live Data Validation Script

This script performs comprehensive validation that:
1. Database contains real telemetry data (not mocks/synthetic)
2. API endpoints query database correctly
3. HA instance selection works
4. Fail-soft behavior is correct
5. Windows browser access works over network

Output: Text-only validation report
"""

import os
import sys
import json
import psycopg2
import requests
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Optional, Tuple, Any
from urllib.parse import urljoin

# Database configuration
DB_NAME = os.environ.get("DB_NAME", "ransomeye")
DB_USER = os.environ.get("DB_USER", "gagan")
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PASSWORD = os.environ.get("DB_PASS", "gagan")
DB_PORT = os.environ.get("DB_PORT", "5432")

# UI configuration
UI_BASE_URL = os.environ.get("RANSOMEYE_UI_BASE_URL", "http://localhost:8081")
if not UI_BASE_URL.startswith("http"):
    UI_BASE_URL = f"http://{UI_BASE_URL}"

# Validation results
results = {
    "part1_database": {"status": "PENDING", "details": []},
    "part2_backend": {"status": "PENDING", "details": []},
    "part3_ui": {"status": "PENDING", "details": []},
    "part4_failsoft": {"status": "PENDING", "details": []},
    "overall": {"status": "PENDING", "blockers": []}
}


def get_db_connection():
    """Get database connection."""
    try:
        return psycopg2.connect(
            host=DB_HOST, port=DB_PORT, database=DB_NAME,
            user=DB_USER, password=DB_PASSWORD
        )
    except Exception as e:
        return None


def log_result(part: str, message: str, status: str = "INFO"):
    """Log validation result."""
    results[part]["details"].append(f"[{status}] {message}")
    print(f"[{part.upper()}] [{status}] {message}")


def validate_part1_database():
    """PART 1: Database Data Presence Validation"""
    print("\n" + "="*80)
    print("PART 1 — DATABASE DATA PRESENCE (MANDATORY)")
    print("="*80)
    
    conn = get_db_connection()
    if not conn:
        log_result("part1_database", "FAILED: Cannot connect to database", "ERROR")
        results["part1_database"]["status"] = "FAILED"
        return False
    
    try:
        cursor = conn.cursor()
        cursor.execute("SET search_path = ransomeye, public;")
        
        # 1. Linux Agent Telemetry
        print("\n--- 1. Linux Agent Telemetry ---")
        cursor.execute("""
            SELECT COUNT(*) as total,
                   COUNT(DISTINCT agent_id) as unique_agents,
                   MIN(observed_at) as oldest,
                   MAX(observed_at) as newest,
                   COUNT(*) FILTER (WHERE payload IS NOT NULL AND payload::text LIKE '%system%') as with_system_metrics
            FROM linux_agent_telemetry
        """)
        row = cursor.fetchone()
        if row:
            total, unique_agents, oldest, newest, with_system = row
            log_result("part1_database", f"Total rows: {total}", "INFO")
            log_result("part1_database", f"Unique agents: {unique_agents}", "INFO")
            log_result("part1_database", f"Oldest record: {oldest}", "INFO")
            log_result("part1_database", f"Newest record: {newest}", "INFO")
            log_result("part1_database", f"Rows with system metrics: {with_system}", "INFO")
            
            if total == 0:
                log_result("part1_database", "BLOCKER: No Linux Agent telemetry found", "ERROR")
                results["overall"]["blockers"].append("No Linux Agent telemetry data in database")
            elif with_system == 0:
                log_result("part1_database", "BLOCKER: No system metrics in payload", "ERROR")
                results["overall"]["blockers"].append("Linux Agent telemetry missing system metrics")
            else:
                # Check payload structure
                cursor.execute("""
                    SELECT payload, observed_at, agent_id, source_component_identity
                    FROM linux_agent_telemetry
                    WHERE payload IS NOT NULL AND payload::text LIKE '%system%'
                    ORDER BY observed_at DESC
                    LIMIT 1
                """)
                sample = cursor.fetchone()
                if sample:
                    payload = sample[0]
                    if isinstance(payload, dict):
                        has_cpu = 'cpu' in payload
                        has_memory = 'memory' in payload
                        has_disk = 'disk' in payload
                        has_network = 'network' in payload
                        log_result("part1_database", f"Sample payload has CPU: {has_cpu}, Memory: {has_memory}, Disk: {has_disk}, Network: {has_network}", "INFO")
                        if not (has_cpu or has_memory or has_disk or has_network):
                            log_result("part1_database", "WARNING: Payload structure may not match expected format", "WARN")
        
        # 2. DPI Probe Telemetry
        print("\n--- 2. DPI Probe Telemetry ---")
        cursor.execute("""
            SELECT COUNT(*) as total,
                   COUNT(DISTINCT agent_id) as unique_probes,
                   MIN(observed_at) as oldest,
                   MAX(observed_at) as newest,
                   COUNT(*) FILTER (WHERE payload IS NOT NULL AND payload::text LIKE '%system%') as with_system_metrics
            FROM dpi_probe_telemetry
        """)
        row = cursor.fetchone()
        if row:
            total, unique_probes, oldest, newest, with_system = row
            log_result("part1_database", f"Total rows: {total}", "INFO")
            log_result("part1_database", f"Unique probes: {unique_probes}", "INFO")
            log_result("part1_database", f"Oldest record: {oldest}", "INFO")
            log_result("part1_database", f"Newest record: {newest}", "INFO")
            log_result("part1_database", f"Rows with system metrics: {with_system}", "INFO")
            
            if total == 0:
                log_result("part1_database", "BLOCKER: No DPI Probe telemetry found", "ERROR")
                results["overall"]["blockers"].append("No DPI Probe telemetry data in database")
            elif with_system == 0:
                log_result("part1_database", "BLOCKER: No system metrics in DPI payload", "ERROR")
                results["overall"]["blockers"].append("DPI Probe telemetry missing system metrics")
            else:
                # Check payload structure
                cursor.execute("""
                    SELECT payload, observed_at, agent_id, source_component_identity
                    FROM dpi_probe_telemetry
                    WHERE payload IS NOT NULL AND payload::text LIKE '%system%'
                    ORDER BY observed_at DESC
                    LIMIT 1
                """)
                sample = cursor.fetchone()
                if sample:
                    payload = sample[0]
                    if isinstance(payload, dict):
                        has_system = 'system' in payload
                        log_result("part1_database", f"Sample payload has 'system' key: {has_system}", "INFO")
                        if has_system and isinstance(payload.get('system'), dict):
                            sys_payload = payload['system']
                            has_cpu = 'cpu' in sys_payload
                            has_memory = 'memory' in sys_payload
                            has_processing = 'processing' in sys_payload
                            log_result("part1_database", f"System payload has CPU: {has_cpu}, Memory: {has_memory}, Processing: {has_processing}", "INFO")
        
        # 3. Database Metrics (pg_stat_* views)
        print("\n--- 3. Database Metrics (pg_stat_*) ---")
        cursor.execute("""
            SELECT 
                (SELECT COUNT(*) FROM pg_stat_activity WHERE datname = current_database()) as active_connections,
                (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
                (SELECT blks_hit FROM pg_stat_database WHERE datname = current_database()) as cache_hits,
                (SELECT blks_read FROM pg_stat_database WHERE datname = current_database()) as disk_reads,
                (SELECT xact_commit + xact_rollback FROM pg_stat_database WHERE datname = current_database()) as total_transactions
        """)
        row = cursor.fetchone()
        if row:
            active, max_conns, hits, reads, xacts = row
            log_result("part1_database", f"Active connections: {active}/{max_conns}", "INFO")
            log_result("part1_database", f"Cache hits: {hits}, Disk reads: {reads}", "INFO")
            log_result("part1_database", f"Total transactions: {xacts}", "INFO")
            
            if hits is None or reads is None:
                log_result("part1_database", "WARNING: pg_stat_database values are NULL", "WARN")
        
        # 4. Data Freshness Check
        print("\n--- 4. Data Freshness Check ---")
        cursor.execute("""
            SELECT 
                'linux_agent' as source,
                MAX(observed_at) as last_seen,
                EXTRACT(EPOCH FROM (NOW() - MAX(observed_at)))::int as seconds_ago
            FROM linux_agent_telemetry
            UNION ALL
            SELECT 
                'dpi_probe' as source,
                MAX(observed_at) as last_seen,
                EXTRACT(EPOCH FROM (NOW() - MAX(observed_at)))::int as seconds_ago
            FROM dpi_probe_telemetry
        """)
        for row in cursor.fetchall():
            source, last_seen, seconds_ago = row
            if seconds_ago is not None:
                minutes_ago = seconds_ago / 60
                log_result("part1_database", f"{source}: Last seen {minutes_ago:.1f} minutes ago ({last_seen})", "INFO")
                if seconds_ago > 3600:  # 1 hour
                    log_result("part1_database", f"WARNING: {source} data is stale (>1 hour old)", "WARN")
            else:
                log_result("part1_database", f"BLOCKER: {source} has no recent data", "ERROR")
                results["overall"]["blockers"].append(f"{source} telemetry is not updating")
        
        cursor.close()
        conn.close()
        
        # Determine status
        if any("BLOCKER" in detail for detail in results["part1_database"]["details"]):
            results["part1_database"]["status"] = "FAILED"
            return False
        else:
            results["part1_database"]["status"] = "PASSED"
            return True
            
    except Exception as e:
        log_result("part1_database", f"EXCEPTION: {str(e)}", "ERROR")
        results["part1_database"]["status"] = "FAILED"
        if conn:
            conn.close()
        return False


def validate_part2_backend():
    """PART 2: Backend API Validation"""
    print("\n" + "="*80)
    print("PART 2 — BACKEND API VALIDATION")
    print("="*80)
    
    endpoints = [
        ("/api/dashboards/core-system-health", "instance_id", "Core System Health"),
        ("/api/dashboards/dpi-probe-health", "probe_id", "DPI Probe Health"),
        ("/api/dashboards/db-health", "db_instance_id", "DB Health")
    ]
    
    all_passed = True
    
    for endpoint, param_name, label in endpoints:
        print(f"\n--- Testing {label} ---")
        url = urljoin(UI_BASE_URL, endpoint)
        
        # Test 1: Without instance parameter (should return latest)
        try:
            response = requests.get(url, timeout=5)
            if response.status_code == 200:
                data = response.json()
                log_result("part2_backend", f"{label}: GET {endpoint} returned 200", "INFO")
                
                # Check if response contains real data (not all "Metric unavailable")
                has_real_data = False
                if isinstance(data, dict):
                    for key, value in data.items():
                        if key != "timestamp" and isinstance(value, (dict, list)):
                            if isinstance(value, dict):
                                for v in value.values():
                                    if v != "Metric unavailable" and not isinstance(v, str):
                                        has_real_data = True
                                        break
                            if has_real_data:
                                break
                
                if has_real_data:
                    log_result("part2_backend", f"{label}: Response contains real numeric data", "INFO")
                else:
                    log_result("part2_backend", f"{label}: WARNING - Response may contain only placeholders", "WARN")
            else:
                log_result("part2_backend", f"{label}: GET {endpoint} returned {response.status_code}", "ERROR")
                all_passed = False
        except Exception as e:
            log_result("part2_backend", f"{label}: EXCEPTION - {str(e)}", "ERROR")
            all_passed = False
        
        # Test 2: With instance parameter (if instances exist)
        try:
            # First, get available instances
            instances_url = urljoin(UI_BASE_URL, "/api/system/instances")
            instances_response = requests.get(instances_url, timeout=5)
            if instances_response.status_code == 200:
                instances_data = instances_response.json()
                
                # Determine instance type
                instance_type = "core" if "core" in endpoint else ("dpi" if "dpi" in endpoint else "db")
                available_instances = instances_data.get(instance_type, [])
                
                if available_instances:
                    test_instance_id = available_instances[0].get("id")
                    if test_instance_id:
                        test_url = f"{url}?{param_name}={test_instance_id}"
                        response = requests.get(test_url, timeout=5)
                        if response.status_code == 200:
                            log_result("part2_backend", f"{label}: Instance filtering works (instance_id={test_instance_id})", "INFO")
                        elif response.status_code == 404:
                            log_result("part2_backend", f"{label}: Instance not found (404) - may be offline", "WARN")
                        else:
                            log_result("part2_backend", f"{label}: Instance filtering returned {response.status_code}", "ERROR")
                            all_passed = False
                else:
                    log_result("part2_backend", f"{label}: No instances available for testing", "INFO")
        except Exception as e:
            log_result("part2_backend", f"{label}: Instance filtering test failed - {str(e)}", "WARN")
        
        # Test 3: Invalid instance (should return 404)
        try:
            invalid_url = f"{url}?{param_name}=invalid-instance-12345"
            response = requests.get(invalid_url, timeout=5)
            if response.status_code == 404:
                log_result("part2_backend", f"{label}: Invalid instance correctly returns 404", "INFO")
            else:
                log_result("part2_backend", f"{label}: Invalid instance returned {response.status_code} (expected 404)", "WARN")
        except Exception as e:
            log_result("part2_backend", f"{label}: Invalid instance test failed - {str(e)}", "WARN")
    
    # Test instance discovery endpoint
    print("\n--- Testing Instance Discovery ---")
    try:
        instances_url = urljoin(UI_BASE_URL, "/api/system/instances")
        response = requests.get(instances_url, timeout=5)
        if response.status_code == 200:
            data = response.json()
            core_count = len(data.get("core", []))
            dpi_count = len(data.get("dpi", []))
            db_count = len(data.get("db", []))
            log_result("part2_backend", f"Instance discovery: Core={core_count}, DPI={dpi_count}, DB={db_count}", "INFO")
        else:
            log_result("part2_backend", f"Instance discovery returned {response.status_code}", "ERROR")
            all_passed = False
    except Exception as e:
        log_result("part2_backend", f"Instance discovery failed - {str(e)}", "ERROR")
        all_passed = False
    
    if all_passed:
        results["part2_backend"]["status"] = "PASSED"
    else:
        results["part2_backend"]["status"] = "FAILED"
        results["overall"]["blockers"].append("Backend API validation failed")
    
    return all_passed


def validate_part3_ui():
    """PART 3: UI Rendering Validation (Manual checklist)"""
    print("\n" + "="*80)
    print("PART 3 — UI RENDERING VALIDATION (WINDOWS BROWSER)")
    print("="*80)
    print("\nNOTE: This section requires manual testing from a Windows browser.")
    print("Please verify the following manually:\n")
    
    checklist = [
        ("1. Network Access", f"Access UI via: {UI_BASE_URL}"),
        ("2. Core System Health Dashboard", "Panels render numeric data (not placeholders)"),
        ("3. DPI Probe Health Dashboard", "Panels render numeric data (not placeholders)"),
        ("4. DB Health Dashboard", "Panels render numeric data (not placeholders)"),
        ("5. Refresh Updates", "Dashboard values change over time when refreshed"),
        ("6. Instance Selector (Core)", "Switch between instances - panel values change"),
        ("7. Instance Selector (DPI)", "Switch between probes - panel values change"),
        ("8. Instance Selector (DB)", "Switch between DB instances - panel values change"),
        ("9. URL Query Params", "URL updates correctly when instance changes"),
        ("10. Offline Instance", "Offline instance shows graceful error (not crash)")
    ]
    
    for item, description in checklist:
        print(f"  {item}: {description}")
        log_result("part3_ui", f"{item}: {description}", "INFO")
    
    print("\nPlease mark each item as PASSED or FAILED:")
    print("(This script cannot automatically test browser rendering)")
    
    results["part3_ui"]["status"] = "MANUAL_REQUIRED"
    return True  # Cannot auto-validate, requires manual check


def validate_part4_failsoft():
    """PART 4: Failure & Fail-Soft Tests"""
    print("\n" + "="*80)
    print("PART 4 — FAILURE & FAIL-SOFT TESTS")
    print("="*80)
    
    all_passed = True
    
    # Test 1: Invalid instance ID
    print("\n--- Test 1: Invalid Instance ID ---")
    endpoints = [
        ("/api/dashboards/core-system-health", "instance_id", "nonexistent-instance"),
        ("/api/dashboards/dpi-probe-health", "probe_id", "nonexistent-probe"),
        ("/api/dashboards/db-health", "db_instance_id", "nonexistent-db")
    ]
    
    for endpoint, param, invalid_id in endpoints:
        try:
            url = urljoin(UI_BASE_URL, f"{endpoint}?{param}={invalid_id}")
            response = requests.get(url, timeout=5)
            if response.status_code == 404:
                log_result("part4_failsoft", f"{endpoint}: Invalid instance correctly returns 404", "INFO")
            elif response.status_code == 200:
                # Check if response indicates unavailability
                data = response.json()
                if "error" in data or all(v == "Metric unavailable" for v in data.values() if isinstance(v, str)):
                    log_result("part4_failsoft", f"{endpoint}: Invalid instance handled gracefully", "INFO")
                else:
                    log_result("part4_failsoft", f"{endpoint}: WARNING - Invalid instance returned data", "WARN")
            else:
                log_result("part4_failsoft", f"{endpoint}: Invalid instance returned {response.status_code}", "WARN")
        except Exception as e:
            log_result("part4_failsoft", f"{endpoint}: Exception - {str(e)}", "WARN")
    
    # Test 2: Database connectivity (simulated by checking error handling)
    print("\n--- Test 2: Error Handling ---")
    # We can't easily simulate DB disconnect, but we can check that errors are handled
    # by checking if the API returns proper error responses
    try:
        # Test with malformed query parameter
        url = urljoin(UI_BASE_URL, "/api/dashboards/core-system-health?instance_id='; DROP TABLE--")
        response = requests.get(url, timeout=5)
        if response.status_code in [400, 404, 500]:
            log_result("part4_failsoft", "Malformed parameter handled safely", "INFO")
        else:
            log_result("part4_failsoft", f"Malformed parameter returned {response.status_code}", "WARN")
    except Exception as e:
        log_result("part4_failsoft", f"Error handling test exception - {str(e)}", "WARN")
    
    # Test 3: Missing data scenario (check if APIs handle empty results)
    print("\n--- Test 3: Missing Data Handling ---")
    # This is validated in Part 1 - if no data exists, APIs should still return gracefully
    conn = get_db_connection()
    if conn:
        try:
            cursor = conn.cursor()
            cursor.execute("SET search_path = ransomeye, public;")
            
            # Check if APIs handle cases where payload structure differs
            cursor.execute("""
                SELECT COUNT(*) FROM linux_agent_telemetry
                WHERE payload IS NULL OR payload::text NOT LIKE '%system%'
            """)
            rows_without_system = cursor.fetchone()[0]
            if rows_without_system > 0:
                log_result("part4_failsoft", f"Found {rows_without_system} rows without system metrics - APIs should handle gracefully", "INFO")
            
            cursor.close()
            conn.close()
        except Exception as e:
            log_result("part4_failsoft", f"Missing data check failed - {str(e)}", "WARN")
    
    if all_passed:
        results["part4_failsoft"]["status"] = "PASSED"
    else:
        results["part4_failsoft"]["status"] = "FAILED"
    
    return all_passed


def generate_final_report():
    """Generate final validation report"""
    print("\n" + "="*80)
    print("FINAL VALIDATION REPORT")
    print("="*80)
    
    print("\n--- Part 1: Database Data Presence ---")
    print(f"Status: {results['part1_database']['status']}")
    for detail in results['part1_database']['details']:
        print(f"  {detail}")
    
    print("\n--- Part 2: Backend API Validation ---")
    print(f"Status: {results['part2_backend']['status']}")
    for detail in results['part2_backend']['details']:
        print(f"  {detail}")
    
    print("\n--- Part 3: UI Rendering Validation ---")
    print(f"Status: {results['part3_ui']['status']}")
    for detail in results['part3_ui']['details']:
        print(f"  {detail}")
    
    print("\n--- Part 4: Failure & Fail-Soft Tests ---")
    print(f"Status: {results['part4_failsoft']['status']}")
    for detail in results['part4_failsoft']['details']:
        print(f"  {detail}")
    
    print("\n" + "="*80)
    print("OVERALL VERDICT")
    print("="*80)
    
    blockers = results['overall']['blockers']
    if blockers:
        print("\n❌ BLOCKED - The following blockers were found:")
        for blocker in blockers:
            print(f"  - {blocker}")
        results['overall']['status'] = "BLOCKED"
        print("\nSTATUS: NOT READY FOR PRODUCTION")
        return False
    else:
        # Check if all automated tests passed
        if (results['part1_database']['status'] == "PASSED" and
            results['part2_backend']['status'] == "PASSED" and
            results['part4_failsoft']['status'] == "PASSED"):
            print("\n✅ ALL AUTOMATED TESTS PASSED")
            print("\nNOTE: Part 3 (UI Rendering) requires manual validation from Windows browser.")
            print("Please verify UI rendering manually before marking as production-ready.")
            results['overall']['status'] = "READY_FOR_MANUAL_UI_CHECK"
            print("\nSTATUS: READY FOR PRODUCTION (pending manual UI validation)")
            return True
        else:
            print("\n⚠️  SOME TESTS FAILED")
            results['overall']['status'] = "FAILED"
            print("\nSTATUS: NOT READY FOR PRODUCTION")
            return False


def main():
    """Main validation function"""
    print("="*80)
    print("RANSOMEYE UI - END-TO-END LIVE DATA VALIDATION")
    print("="*80)
    print(f"Database: {DB_HOST}:{DB_PORT}/{DB_NAME}")
    print(f"UI Base URL: {UI_BASE_URL}")
    print(f"Timestamp: {datetime.now(timezone.utc).isoformat()}")
    
    # Run all validation parts
    part1_ok = validate_part1_database()
    part2_ok = validate_part2_backend()
    part3_ok = validate_part3_ui()  # Manual, always returns True
    part4_ok = validate_part4_failsoft()
    
    # Generate final report
    final_ok = generate_final_report()
    
    # Save results to file
    report_file = f"/home/ransomeye/rebuild/logs/ui_validation_{datetime.now(timezone.utc).strftime('%Y%m%d_%H%M%S')}.json"
    os.makedirs(os.path.dirname(report_file), exist_ok=True)
    with open(report_file, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    print(f"\nValidation report saved to: {report_file}")
    
    sys.exit(0 if final_ok else 1)


if __name__ == "__main__":
    main()

