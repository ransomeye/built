# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/phase_d1_scale_readiness.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Phase D1 - Multi-Node & HA Validation

# Phase D1 - Multi-Node & High Availability Validation

**Date:** 2026-01-28  
**Phase:** PROMPT-53 — BLOCKER 5 Resolution  
**Status:** ✅ **FRAMEWORK COMPLETE**

---

## Objective

Validate multi-node deployment and high availability:
- Spin 2nd ingestion instance
- Validate DB pooling & backpressure
- Kill one node → verify continuity
- Validate failover behavior

---

## Multi-Node Architecture

### Node Configuration

**Node 1 (Primary):**
- Host: `core-node-1.example.com`
- IP: `10.0.0.10`
- Services: Core, Ingestion, DB
- Role: Primary

**Node 2 (Secondary):**
- Host: `core-node-2.example.com`
- IP: `10.0.0.11`
- Services: Core, Ingestion, DB (read replica)
- Role: Secondary

---

## Test Scenario 1: Second Ingestion Instance

### Setup

**Steps:**
1. Deploy second ingestion instance on Node 2
2. Configure load balancing (round-robin or least-connections)
3. Point agents to load balancer endpoint
4. Verify both instances receiving traffic

**Configuration:**
```yaml
# Node 2 ingestion config
ingestion:
  bind_address: "0.0.0.0"
  bind_port: 8080
  db_pool_size: 20
  max_connections: 1000
  backpressure_threshold: 80
```

**Validation:**
```bash
# Check Node 2 ingestion status
systemctl status ransomeye-ingestion.service

# Check Node 2 ingestion logs
journalctl -u ransomeye-ingestion.service -f

# Verify traffic distribution
# (Check load balancer logs or ingestion metrics)
```

**Expected Result:**
- ✅ Second ingestion instance running
- ✅ Both instances receiving traffic
- ✅ Load balanced correctly

---

## Test Scenario 2: Database Pooling & Backpressure

### Database Connection Pooling

**Configuration:**
```yaml
# Database pool configuration
database:
  host: "10.0.0.10"
  port: 5432
  database: "ransomeye"
  user: "gagan"
  password: "gagan"
  pool_size: 20
  max_connections: 100
  connection_timeout: 30
```

**Validation:**
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'ransomeye';

-- Check connection pool usage
SELECT 
    state,
    count(*) as connections
FROM pg_stat_activity
WHERE datname = 'ransomeye'
GROUP BY state;
```

**Expected Result:**
- ✅ Connection pool configured correctly
- ✅ Connections distributed across pool
- ✅ No connection exhaustion

---

### Backpressure Validation

**Test Procedure:**
1. Generate high load (10k events/min)
2. Monitor backpressure activation
3. Verify backpressure signals sent
4. Verify events buffered (not dropped)

**Validation:**
```bash
# Monitor backpressure metrics
curl http://localhost:8080/metrics | grep backpressure

# Check ingestion logs for backpressure
journalctl -u ransomeye-ingestion.service | grep -i backpressure

# Check agent logs for backpressure signals
journalctl -u ransomeye-linux-agent.service | grep -i backpressure
```

**Expected Result:**
- ✅ Backpressure activates at threshold (80%)
- ✅ Backpressure signals sent to agents
- ✅ Events buffered (not silently dropped)
- ✅ System recovers when load decreases

---

## Test Scenario 3: Node Failure & Continuity

### Kill One Node

**Test Procedure:**
1. Stop Node 1 (primary) services
2. Verify Node 2 continues operation
3. Verify agents reconnect to Node 2
4. Verify no data loss
5. Restart Node 1
6. Verify Node 1 rejoins cluster

**Steps:**
```bash
# On Node 1: Stop services
systemctl stop ransomeye-core.service
systemctl stop ransomeye-ingestion.service

# On Node 2: Verify services running
systemctl status ransomeye-core.service
systemctl status ransomeye-ingestion.service

# Check agent reconnection
journalctl -u ransomeye-linux-agent.service | grep -i "reconnect\|failover"

# Verify data continuity
# (Check database for events during failover period)
```

**Expected Result:**
- ✅ Node 2 continues operation
- ✅ Agents reconnect to Node 2
- ✅ No data loss during failover
- ✅ Node 1 rejoins cluster successfully

---

## High Availability Validation Matrix

| HA Aspect | Requirement | Validation Method | Status |
|-----------|-------------|-------------------|--------|
| **Multi-Node Deployment** | 2+ nodes operational | Check service status | PENDING |
| **Load Balancing** | Traffic distributed | Check load balancer logs | PENDING |
| **DB Pooling** | Connection pool configured | Check DB connections | PENDING |
| **Backpressure** | Backpressure activates | Check backpressure metrics | PENDING |
| **Node Failure** | Continuity maintained | Kill node, verify continuity | PENDING |
| **Failover** | Automatic failover | Verify agent reconnection | PENDING |
| **Data Loss** | Zero data loss | Verify event continuity | PENDING |
| **Recovery** | Node rejoins cluster | Restart node, verify join | PENDING |

---

## Database Replication (Optional)

### Read Replica Configuration

**Configuration:**
```yaml
# Node 2 read replica
database:
  host: "10.0.0.10"  # Primary DB
  port: 5432
  database: "ransomeye"
  read_replica: true
  replication_lag_threshold: 5  # seconds
```

**Validation:**
```sql
-- Check replication lag
SELECT 
    client_addr,
    state,
    sync_state,
    pg_wal_lsn_diff(pg_current_wal_lsn(), sent_lsn) as replication_lag
FROM pg_stat_replication;
```

**Expected Result:**
- ✅ Read replica configured
- ✅ Replication lag within threshold
- ✅ Read queries routed to replica

---

## Test Execution

### Test Script Location
`tests/multi_node_ha_validation.sh`

### Test Execution
```bash
# Run multi-node HA validation tests
./tests/multi_node_ha_validation.sh

# Expected output:
# - Multi-node deployment validation
# - DB pooling validation
# - Backpressure validation
# - Node failure & continuity validation
```

---

## Conclusion

**Phase D1 Status:** ✅ **FRAMEWORK COMPLETE**

Multi-node & HA validation framework is complete with:
- ✅ Multi-node architecture defined
- ✅ Database pooling procedures documented
- ✅ Backpressure validation procedures documented
- ✅ Node failure & continuity procedures documented
- ✅ Test execution framework ready

**Next Steps:**
1. Deploy second ingestion instance
2. Configure load balancing
3. Execute HA validation tests
4. Verify failover behavior
5. Proceed to Phase D2-D3

**Blocking Issues:** Multi-node deployment required (infrastructure setup)

