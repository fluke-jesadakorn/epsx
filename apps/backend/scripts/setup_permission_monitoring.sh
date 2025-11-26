#!/bin/bash

# ================================================================================================
# PRODUCTION MONITORING CONFIGURATION - UNIFIED PERMISSION SYSTEM
# ================================================================================================
# This script sets up comprehensive monitoring and alerting for the refactored permission system.
# It monitors performance metrics, cache hit rates, error rates, and system health.
#
# Metrics Monitored:
# - Permission validation latency (target: <10ms)
# - Cache hit rates (target: >80%)
# - Database query performance
# - Error rates and alerting
# - System resource utilization
# - Materialized view refresh performance
# ================================================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT="${ENVIRONMENT:-production}"
PROJECT_ID="epsx-469400"
REGION="us-central1"
BACKEND_SERVICE="backend"
FRONTEND_SERVICE="frontend"
ADMIN_SERVICE="admin"

# Monitoring thresholds
PERMISSION_LATENCY_WARNING=5    # ms
PERMISSION_LATENCY_CRITICAL=10  # ms
CACHE_HIT_RATE_WARNING=70       # %
CACHE_HIT_RATE_CRITICAL=50      # %
ERROR_RATE_WARNING=5            # %
ERROR_RATE_CRITICAL=10          # %
DB_CONNECTION_WARNING=70        # % of pool
DB_CONNECTION_CRITICAL=90       # % of pool

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_monitor() {
    echo -e "${PURPLE}[MONITOR]${NC} $1"
}

# Create monitoring configuration directory
setup_monitoring_directory() {
    local monitor_dir="monitoring"

    log_info "Setting up monitoring configuration..."
    mkdir -p "$monitor_dir"/{dashboards,alerts,prometheus,grafana}

    # Create directory structure
    mkdir -p "$monitor_dir/dashboards/permission-system"
    mkdir -p "$monitor_dir/alerts/permission-system"
    mkdir -p "$monitor_dir/prometheus/rules"
    mkdir -p "$monitor_dir/grafana/dashboards"

    log_success "✓ Monitoring directory structure created"
}

# Create Prometheus configuration
create_prometheus_config() {
    local prometheus_config="monitoring/prometheus/prometheus.yml"

    log_monitor "Creating Prometheus configuration..."

    cat > "$prometheus_config" << 'EOF'
# Prometheus Configuration for EPSX Unified Permission System
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'epsx-production'
    replica: 'prometheus-1'

rule_files:
  - "rules/*_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # Backend service monitoring
  - job_name: 'epsx-backend'
    static_configs:
      - targets: ['backend:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s

    # Permission system specific metrics
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        replacement: 'epsx-backend'

  # Frontend service monitoring (if available)
  - job_name: 'epsx-frontend'
    static_configs:
      - targets: ['frontend:3000']
    metrics_path: '/api/metrics'
    scrape_interval: 30s

  # Admin service monitoring (if available)
  - job_name: 'epsx-admin'
    static_configs:
      - targets: ['admin:3001']
    metrics_path: '/api/metrics'
    scrape_interval: 30s

  # PostgreSQL database monitoring
  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s

  # Redis monitoring (for caching)
  - job_name: 'redis-exporter'
    static_configs:
      - targets: ['redis-exporter:9121']
    scrape_interval: 30s

  # Node exporter for system metrics
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 30s
EOF

    log_success "✓ Prometheus configuration created"
}

# Create custom Prometheus metrics for permission system
create_permission_metrics() {
    local metrics_file="monitoring/prometheus/permission_metrics.yml"

    log_monitor "Creating permission system custom metrics..."

    cat > "$metrics_file" << 'EOF'
# Custom Metrics for EPSX Unified Permission System

# Permission validation latency histogram
permission_validation_duration_seconds{quantile="0.5"} 0.005
permission_validation_duration_seconds{quantile="0.9"} 0.008
permission_validation_duration_seconds{quantile="0.95"} 0.010
permission_validation_duration_seconds{quantile="0.99"} 0.015
permission_validation_duration_seconds_sum 1250.5
permission_validation_duration_seconds_count 250000

# Cache hit rate gauge
permission_cache_hit_rate 85.5

# Permission validation counter
permission_validation_total{status="success"} 247500
permission_validation_total{status="failure"} 2500

# Database query performance
permission_db_query_duration_seconds{query_type="validation",quantile="0.5"} 0.002
permission_db_query_duration_seconds{query_type="validation",quantile="0.9"} 0.004
permission_db_query_duration_seconds{query_type="validation",quantile="0.95"} 0.005

# Active permissions gauge
active_permissions_total 15420

# Wallet user count
wallet_users_total 8932

# Materialized view refresh performance
materialized_view_refresh_duration_seconds{view="wallet_permissions_view"} 0.050

# Permission source distribution
permissions_by_source{source_type="direct"} 8234
permissions_by_source{source_type="group"} 5632
permissions_by_source{source_type="route"} 1554

# Expired permissions cleanup metrics
expired_permissions_cleaned_total 234
expired_permissions_cleanup_duration_seconds 2.5
EOF

    log_success "✓ Permission metrics configuration created"
}

# Create alerting rules
create_alerting_rules() {
    local rules_file="monitoring/prometheus/rules/permission_system_rules.yml"

    log_monitor "Creating alerting rules for permission system..."

    cat > "$rules_file" << EOF
# Alerting Rules for EPSX Unified Permission System

groups:
  - name: permission_system_performance
    rules:
      # Permission validation latency alerts
      - alert: PermissionValidationLatencyHigh
        expr: permission_validation_duration_seconds{quantile="0.95"} > ${PERMISSION_LATENCY_CRITICAL/1000}
        for: 2m
        labels:
          severity: critical
          service: backend
          component: permission-system
        annotations:
          summary: "Permission validation latency is critical"
          description: "95th percentile latency is {{ \$value }}s, threshold is ${PERMISSION_LATENCY_CRITICAL/1000}s"

      - alert: PermissionValidationLatencyWarning
        expr: permission_validation_duration_seconds{quantile="0.95"} > ${PERMISSION_LATENCY_WARNING/1000}
        for: 5m
        labels:
          severity: warning
          service: backend
          component: permission-system
        annotations:
          summary: "Permission validation latency elevated"
          description: "95th percentile latency is {{ \$value }}s, threshold is ${PERMISSION_LATENCY_WARNING/1000}s"

      # Cache hit rate alerts
      - alert: CacheHitRateLow
        expr: permission_cache_hit_rate < ${CACHE_HIT_RATE_CRITICAL}
        for: 5m
        labels:
          severity: critical
          service: backend
          component: permission-system
        annotations:
          summary: "Permission cache hit rate is critically low"
          description: "Cache hit rate is {{ \$value }}%, threshold is ${CACHE_HIT_RATE_CRITICAL}%"

      - alert: CacheHitRateDegraded
        expr: permission_cache_hit_rate < ${CACHE_HIT_RATE_WARNING}
        for: 10m
        labels:
          severity: warning
          service: backend
          component: permission-system
        annotations:
          summary: "Permission cache hit rate degraded"
          description: "Cache hit rate is {{ \$value }}%, threshold is ${CACHE_HIT_RATE_WARNING}%"

      # Permission validation error rate
      - alert: PermissionValidationErrorRateHigh
        expr: |
          (
            rate(permission_validation_total{status="failure"}[5m]) /
            (
              rate(permission_validation_total{status="failure"}[5m]) +
              rate(permission_validation_total{status="success"}[5m])
            )
          ) * 100 > ${ERROR_RATE_CRITICAL}
        for: 2m
        labels:
          severity: critical
          service: backend
          component: permission-system
        annotations:
          summary: "High permission validation error rate"
          description: "Error rate is {{ \$value }}%, threshold is ${ERROR_RATE_CRITICAL}%"

      - alert: PermissionValidationErrorRateElevated
        expr: |
          (
            rate(permission_validation_total{status="failure"}[5m]) /
            (
              rate(permission_validation_total{status="failure"}[5m]) +
              rate(permission_validation_total{status="success"}[5m])
            )
          ) * 100 > ${ERROR_RATE_WARNING}
        for: 5m
        labels:
          severity: warning
          service: backend
          component: permission-system
        annotations:
          summary: "Elevated permission validation error rate"
          description: "Error rate is {{ \$value }}%, threshold is ${ERROR_RATE_WARNING}%"

  - name: database_performance
    rules:
      # Database query performance
      - alert: DatabaseQueryLatencyHigh
        expr: permission_db_query_duration_seconds{quantile="0.95"} > 0.01
        for: 3m
        labels:
          severity: warning
          service: backend
          component: database
        annotations:
          summary: "Database query latency elevated"
          description: "95th percentile query latency is {{ \$value }}s"

      # Database connection pool usage
      - alert: DatabaseConnectionPoolHigh
        expr: db_connections_active / db_connections_max * 100 > ${DB_CONNECTION_CRITICAL}
        for: 2m
        labels:
          severity: critical
          service: backend
          component: database
        annotations:
          summary: "Database connection pool usage critical"
          description: "Connection pool usage is {{ \$value }}%, threshold is ${DB_CONNECTION_CRITICAL}%"

      - alert: DatabaseConnectionPoolElevated
        expr: db_connections_active / db_connections_max * 100 > ${DB_CONNECTION_WARNING}
        for: 5m
        labels:
          severity: warning
          service: backend
          component: database
        annotations:
          summary: "Database connection pool usage elevated"
          description: "Connection pool usage is {{ \$value }}%, threshold is ${DB_CONNECTION_WARNING}%"

  - name: system_health
    rules:
      # Materialized view refresh
      - alert: MaterializedViewRefreshSlow
        expr: materialized_view_refresh_duration_seconds > 1.0
        for: 5m
        labels:
          severity: warning
          service: backend
          component: database
        annotations:
          summary: "Materialized view refresh slow"
          description: "wallet_permissions_view refresh took {{ \$value }}s"

      # Service availability
      - alert: BackendServiceDown
        expr: up{job="epsx-backend"} == 0
        for: 1m
        labels:
          severity: critical
          service: backend
        annotations:
          summary: "Backend service is down"
          description: "The EPSX backend service has been down for more than 1 minute"

      - alert: DatabaseDown
        expr: up{job="postgres-exporter"} == 0
        for: 1m
        labels:
          severity: critical
          service: database
        annotations:
          summary: "Database is down"
          description: "PostgreSQL database has been down for more than 1 minute"

      - alert: RedisDown
        expr: up{job="redis-exporter"} == 0
        for: 2m
        labels:
          severity: warning
          service: cache
        annotations:
          summary: "Redis cache is down"
          description: "Redis cache has been down for more than 2 minutes"
EOF

    log_success "✓ Alerting rules created"
}

# Create Grafana dashboard configuration
create_grafana_dashboard() {
    local dashboard_file="monitoring/grafana/dashboards/permission-system.json"

    log_monitor "Creating Grafana dashboard for permission system..."

    cat > "$dashboard_file" << 'EOF'
{
  "dashboard": {
    "id": null,
    "title": "EPSX Permission System Dashboard",
    "tags": ["epsx", "permissions", "performance"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Permission Validation Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, permission_validation_duration_seconds_bucket)",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, permission_validation_duration_seconds_bucket)",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.99, permission_validation_duration_seconds_bucket)",
            "legendFormat": "99th percentile"
          }
        ],
        "yAxes": [
          {
            "label": "Latency (seconds)",
            "min": 0
          }
        ],
        "gridPos": {
          "x": 0,
          "y": 0,
          "w": 12,
          "h": 8
        }
      },
      {
        "id": 2,
        "title": "Cache Hit Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "permission_cache_hit_rate",
            "legendFormat": "Hit Rate %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {
                  "color": "red",
                  "value": 0
                },
                {
                  "color": "yellow",
                  "value": 70
                },
                {
                  "color": "green",
                  "value": 80
                }
              ]
            }
          }
        },
        "gridPos": {
          "x": 12,
          "y": 0,
          "w": 6,
          "h": 8
        }
      },
      {
        "id": 3,
        "title": "Permission Validation Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(permission_validation_total{status=\"success\"}[5m])",
            "legendFormat": "Success Rate"
          },
          {
            "expr": "rate(permission_validation_total{status=\"failure\"}[5m])",
            "legendFormat": "Failure Rate"
          }
        ],
        "yAxes": [
          {
            "label": "Validations per second"
          }
        ],
        "gridPos": {
          "x": 18,
          "y": 0,
          "w": 6,
          "h": 8
        }
      },
      {
        "id": 4,
        "title": "Database Query Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, permission_db_query_duration_seconds_bucket)",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "rate(permission_db_query_duration_seconds_sum[5m]) / rate(permission_db_query_duration_seconds_count[5m])",
            "legendFormat": "Average"
          }
        ],
        "yAxes": [
          {
            "label": "Query Time (seconds)"
          }
        ],
        "gridPos": {
          "x": 0,
          "y": 8,
          "w": 12,
          "h": 8
        }
      },
      {
        "id": 5,
        "title": "Active Permissions Count",
        "type": "stat",
        "targets": [
          {
            "expr": "active_permissions_total",
            "legendFormat": "Active Permissions"
          }
        ],
        "gridPos": {
          "x": 12,
          "y": 8,
          "w": 6,
          "h": 8
        }
      },
      {
        "id": 6,
        "title": "Wallet Users Count",
        "type": "stat",
        "targets": [
          {
            "expr": "wallet_users_total",
            "legendFormat": "Wallet Users"
          }
        ],
        "gridPos": {
          "x": 18,
          "y": 8,
          "w": 6,
          "h": 8
        }
      },
      {
        "id": 7,
        "title": "Permission Sources Distribution",
        "type": "piechart",
        "targets": [
          {
            "expr": "permissions_by_source",
            "legendFormat": "{{source_type}}"
          }
        ],
        "gridPos": {
          "x": 0,
          "y": 16,
          "w": 8,
          "h": 8
        }
      },
      {
        "id": 8,
        "title": "Materialized View Refresh Time",
        "type": "graph",
        "targets": [
          {
            "expr": "materialized_view_refresh_duration_seconds",
            "legendFormat": "Refresh Duration"
          }
        ],
        "yAxes": [
          {
            "label": "Refresh Time (seconds)"
          }
        ],
        "gridPos": {
          "x": 8,
          "y": 16,
          "w": 8,
          "h": 8
        }
      },
      {
        "id": 9,
        "title": "Database Connection Pool Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "db_connections_active / db_connections_max * 100",
            "legendFormat": "Pool Usage %"
          }
        ],
        "yAxes": [
          {
            "label": "Usage %",
            "max": 100,
            "min": 0
          }
        ],
        "gridPos": {
          "x": 16,
          "y": 16,
          "w": 8,
          "h": 8
        }
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}
EOF

    log_success "✓ Grafana dashboard configuration created"
}

# Create Cloud Monitoring metrics configuration
create_cloud_monitoring_config() {
    local cloud_monitoring_file="monitoring/alerts/cloud-monitoring.yml"

    log_monitor "Creating Cloud Monitoring configuration..."

    cat > "$cloud_monitoring_file" << 'EOF'
# Cloud Monitoring Configuration for EPSX Unified Permission System

# Custom Metrics
custom_metrics:
  - name: permission_validation_latency
    type: GAUGE
    description: "Permission validation latency in milliseconds"
    labels:
      - quantile
      - service

  - name: permission_cache_hit_rate
    type: GAUGE
    description: "Permission validation cache hit rate percentage"
    labels:
      - service

  - name: permission_validation_total
    type: COUNTER
    description: "Total number of permission validations"
    labels:
      - status
      - service

  - name: active_permissions_total
    type: GAUGE
    description: "Total number of active permissions"
    labels:
      - service

  - name: wallet_users_total
    type: GAUGE
    description: "Total number of wallet users"
    labels:
      - service

# Alerting Policies
alert_policies:
  - name: PermissionValidationLatencyCritical
    condition:
      filter: 'metric.type="custom.googleapis.com/permission_validation_latency" AND resource.label.service_name="backend" AND metric.label.quantile="95"'
      aggregation:
        alignmentPeriod: "60s"
        perSeriesAligner: ALIGN_PERCENTILE_95
      trigger:
        count: 1
    threshold_value: 10  # 10ms
    duration: "120s"
    notification_channels:
      - projects/epsx-469400/notificationChannels/1234567890

  - name: CacheHitRateLow
    condition:
      filter: 'metric.type="custom.googleapis.com/permission_cache_hit_rate" AND resource.label.service_name="backend"'
      aggregation:
        alignmentPeriod: "300s"
        perSeriesAligner: ALIGN_MEAN
      trigger:
        count: 1
    threshold_value: 50  # 50%
    duration: "300s"
    notification_channels:
      - projects/epsx-469400/notificationChannels/1234567890

  - name: BackendServiceDown
    condition:
      filter: 'metric.type="run.googleapis.com/container/instance/restart_count" AND resource.label.service_name="backend"'
      aggregation:
        alignmentPeriod: "60s"
        perSeriesAligner: ALIGN_DELTA
      trigger:
        count: 1
    threshold_value: 1
    duration: "60s"
    notification_channels:
      - projects/epsx-469400/notificationChannels/1234567890

# Dashboards
dashboards:
  - name: EPSX Permission System
    widgets:
      - title: "Permission Validation Latency"
        chart:
          type: LINE
          metrics:
            - type: custom.googleapis.com/permission_validation_latency
              resource: cloud_run_revision
              service: backend
              filters:
                - quantile: "50"
                - quantile: "95"
                - quantile: "99"

      - title: "Cache Hit Rate"
        chart:
          type: GAUGE
          metrics:
            - type: custom.googleapis.com/permission_cache_hit_rate
              resource: cloud_run_revision
              service: backend

      - title: "Permission Validation Rate"
        chart:
          type: STACKED_BAR
          metrics:
            - type: custom.googleapis.com/permission_validation_total
              resource: cloud_run_revision
              service: backend
              filters:
                - status: "success"
                - status: "failure"
EOF

    log_success "✓ Cloud Monitoring configuration created"
}

# Create health check endpoint metrics
create_health_monitoring() {
    local health_config="monitoring/alerts/health-checks.yml"

    log_monitor "Creating health check configuration..."

    cat > "$health_config" << EOF
# Health Check Configuration for EPSX Unified Permission System

# Service Health Endpoints
health_checks:
  backend:
    url: "https://backend-307278481624.us-central1.run.app/health"
    interval: 30s
    timeout: 10s
    checks:
      - status_code: 200
      - response_time: <5s
      - content_checks:
          - field: "status"
            expected: "healthy"
          - field: "permission_system"
            expected: "operational"

  frontend:
    url: "https://frontend-307278481624.us-central1.run.app"
    interval: 60s
    timeout: 10s
    checks:
      - status_code: 200
      - response_time: <3s

  admin:
    url: "https://admin-307278481624.us-central1.run.app"
    interval: 60s
    timeout: 10s
    checks:
      - status_code: 200
      - response_time: <3s

# Permission System Specific Health Metrics
permission_health:
  validation_endpoint: "/api/v1/permissions/validate"
  test_cases:
    - wallet: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
      permission: "admin:users:manage"
      expected_result: true
      timeout: 10ms
    - wallet: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
      permission: "nonexistent:permission"
      expected_result: false
      timeout: 10ms

  performance_checks:
    - metric: "validation_latency_p95"
      threshold: ${PERMISSION_LATENCY_CRITICAL}ms
    - metric: "cache_hit_rate"
      threshold: ${CACHE_HIT_RATE_CRITICAL}%
    - metric: "error_rate"
      threshold: ${ERROR_RATE_CRITICAL}%

# Database Health Checks
database_health:
  connection_pool:
    check_interval: 30s
    max_connections_warning: ${DB_CONNECTION_WARNING}%
    max_connections_critical: ${DB_CONNECTION_CRITICAL}%

  query_performance:
    slow_query_threshold: 100ms
    connection_timeout: 30s

  materialized_view:
    refresh_threshold: 1s
    staleness_threshold: 300s

# Notification Channels
notification_channels:
  slack:
    webhook_url: "\${SLACK_WEBHOOK_URL}"
    channel: "#epsx-alerts"

  email:
    recipients:
      - "admin@epsx.com"
      - "devops@epsx.com"

  pagerduty:
    service_key: "\${PAGERDUTY_SERVICE_KEY}"
EOF

    log_success "✓ Health check configuration created"
}

# Create Log-based monitoring
create_log_monitoring() {
    local log_config="monitoring/alerts/log-monitoring.yml"

    log_monitor "Creating log-based monitoring configuration..."

    cat > "$log_config" << 'EOF'
# Log-based Monitoring for EPSX Unified Permission System

# Log Filters for Cloud Logging
log_filters:
  permission_errors:
    description: "Permission validation errors"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      severity>=ERROR
      (
        jsonPayload.permission_validation="failed" OR
        jsonPayload.error_type="permission_denied" OR
        jsonPayload.error_type="invalid_permission_format"
      )
    aggregation_period: 300s
    threshold: 5
    notification_channels:
      - "projects/epsx-469400/notificationChannels/1234567890"

  performance_degradation:
    description: "Performance degradation alerts"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      jsonPayload.validation_time_ms>10
    aggregation_period: 600s
    threshold: 10
    notification_channels:
      - "projects/epsx-469400/notificationChannels/1234567890"

  database_errors:
    description: "Database connection and query errors"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      severity>=ERROR
      (
        jsonPayload.error_type="database_connection" OR
        jsonPayload.error_type="query_timeout" OR
        jsonPayload.error_type="pool_exhausted"
      )
    aggregation_period: 300s
    threshold: 3
    notification_channels:
      - "projects/epsx-469400/notificationChannels/1234567890"

  cache_errors:
    description: "Cache-related errors"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      severity>=WARNING
      (
        jsonPayload.cache_operation="miss" AND
        jsonPayload.cache_type="permission" AND
        jsonPayload.result="error"
      )
    aggregation_period: 300s
    threshold: 10

  materialized_view_issues:
    description: "Materialized view refresh issues"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      severity>=WARNING
      (
        jsonPayload.operation="materialized_view_refresh" AND
        (
          jsonPayload.result="failed" OR
          jsonPayload.duration_ms>1000
        )
      )
    aggregation_period: 600s
    threshold: 1

# Log Metrics
log_metrics:
  permission_validation_success_rate:
    description: "Success rate of permission validations"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      jsonPayload.permission_validation="success"
    metric_type: COUNTER

  permission_validation_latency:
    description: "Latency of permission validations"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      jsonPayload.validation_time_ms
    metric_type: GAUGE
    unit: ms

  database_query_time:
    description: "Database query execution time"
    filter: |
      resource.type="cloud_run_revision"
      resource.labels.service_name="backend"
      jsonPayload.db_query_time_ms
    metric_type: GAUGE
    unit: ms

# Alerting Patterns
alert_patterns:
  - name: SuddenErrorSpike
    description: "Sudden spike in error rates"
    condition: |
      rate(log_entries_count{severity>=ERROR}[5m]) >
      5 * rate(log_entries_count{severity>=ERROR}[1h] offset 5m)

  - name: PermissionValidationFailure
    description: "Permission validation failures increasing"
    condition: |
      rate(log_entries_count{jsonPayload.permission_validation="failed"}[5m]) >
      0.01 * rate(log_entries_count{jsonPayload.permission_validation="success"}[5m])

  - name: DatabasePerformanceDegradation
    description: "Database query times increasing"
    condition: |
      avg_over_time(jsonPayload.db_query_time_ms[10m]) > 100
EOF

    log_success "✓ Log-based monitoring configuration created"
}

# Create deployment verification script
create_deployment_verification() {
    local verification_script="monitoring/verify-deployment.sh"

    log_monitor "Creating deployment verification script..."

    cat > "$verification_script" << 'EOF'
#!/bin/bash

# Deployment Verification Script for Permission System
set -euo pipefail

BACKEND_URL="https://backend-307278481624.us-central1.run.app"
FRONTEND_URL="https://frontend-307278481624.us-central1.run.app"
ADMIN_URL="https://admin-307278481624.us-central1.run.app"

echo "🔍 Verifying EPSX Permission System Deployment..."

# Health checks
echo "📊 Checking service health..."
curl -f -s "$BACKEND_URL/health" | jq '.status' || exit 1
curl -f -s "$FRONTEND_URL" > /dev/null || exit 1
curl -f -s "$ADMIN_URL" > /dev/null || exit 1

# Permission validation test
echo "🔐 Testing permission validation..."
curl -X POST "$BACKEND_URL/api/v1/permissions/validate" \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6","permission":"admin:users:manage"}' \
  -f -s | jq '.has_permission' || exit 1

# Performance verification
echo "⚡ Checking performance metrics..."
curl -s "$BACKEND_URL/metrics" | grep "permission_validation_duration_seconds" || exit 1
curl -s "$BACKEND_URL/metrics" | grep "permission_cache_hit_rate" || exit 1

echo "✅ All verifications passed!"
EOF

    chmod +x "$verification_script"
    log_success "✓ Deployment verification script created"
}

# Setup monitoring infrastructure
setup_monitoring_infrastructure() {
    log_monitor "Setting up monitoring infrastructure..."

    # Create Docker Compose for local monitoring
    cat > "monitoring/docker-compose.monitoring.yml" << 'EOF'
# Local Monitoring Stack for EPSX Permission System
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus:/etc/prometheus
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3002:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources
    depends_on:
      - prometheus

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager:/etc/alertmanager
      - alertmanager_data:/alertmanager
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
      - '--storage.path=/alertmanager'

  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:latest
    environment:
      - DATA_SOURCE_NAME=postgresql://postgres:password@host.docker.internal:5432/epsx_db?sslmode=disable
    ports:
      - "9187:9187"
    depends_on:
      - prometheus

  redis-exporter:
    image: oliver006/redis_exporter:latest
    environment:
      - REDIS_ADDR=redis://host.docker.internal:6379
    ports:
      - "9121:9121"
    depends_on:
      - prometheus

  node-exporter:
    image: prom/node-exporter:latest
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.rootfs=/rootfs'
      - '--path.sysfs=/host/sys'

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:
EOF

    log_success "✓ Monitoring infrastructure setup completed"
}

# Create alertmanager configuration
create_alertmanager_config() {
    local alertmanager_file="monitoring/alertmanager/alertmanager.yml"

    log_monitor "Creating Alertmanager configuration..."

    cat > "$alertmanager_file" << 'EOF'
# Alertmanager Configuration for EPSX Unified Permission System
global:
  smtp_smarthost: 'smtp.gmail.com:587'
  smtp_from: 'alerts@epsx.com'
  smtp_auth_username: 'alerts@epsx.com'
  smtp_auth_password: 'your-app-password'

route:
  group_by: ['alertname', 'service', 'component']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'default-receiver'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
      group_wait: 5s
      repeat_interval: 30m

    - match:
        severity: warning
      receiver: 'warning-alerts'
      group_wait: 30s
      repeat_interval: 2h

    - match:
        service: backend
        component: permission-system
      receiver: 'permission-team'

receivers:
  - name: 'default-receiver'
    email_configs:
      - to: 'devops@epsx.com'
        subject: '[EPSX Alert] {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Labels: {{ range .Labels.SortedPairs }}{{ .Name }}={{ .Value }} {{ end }}
          {{ end }}

  - name: 'critical-alerts'
    email_configs:
      - to: 'oncall@epsx.com'
        subject: '[CRITICAL] EPSX Alert: {{ .GroupLabels.alertname }}'
        body: |
          CRITICAL ALERT - IMMEDIATE ATTENTION REQUIRED

          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Severity: {{ .Labels.severity }}
          Service: {{ .Labels.service }}
          Component: {{ .Labels.component }}
          Time: {{ .StartsAt }}
          {{ end }}
    slack_configs:
      - api_url: '${SLACK_WEBHOOK_URL}'
        channel: '#epsx-critical'
        title: '🚨 Critical EPSX Alert'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          *Service:* {{ .Labels.service }}
          *Component:* {{ .Labels.component }}
          {{ end }}
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
        severity: 'critical'

  - name: 'warning-alerts'
    email_configs:
      - to: 'dev-team@epsx.com'
        subject: '[WARNING] EPSX Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}
    slack_configs:
      - api_url: '${SLACK_WEBHOOK_URL}'
        channel: '#epsx-alerts'
        title: '⚠️ EPSX Warning'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          {{ end }}

  - name: 'permission-team'
    email_configs:
      - to: 'permission-team@epsx.com'
        subject: '[Permission System] EPSX Alert: {{ .GroupLabels.alertname }}'
        body: |
          Permission System Alert

          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}
    slack_configs:
      - api_url: '${SLACK_WEBHOOK_URL}'
        channel: '#epsx-permissions'
        title: '🔐 Permission System Alert'
        text: |
          {{ range .Alerts }}
          *Alert:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          {{ end }}

# Inhibit rules to prevent alert spam
inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'service']
EOF

    log_success "✓ Alertmanager configuration created"
}

# Main setup function
main() {
    log_info "=== EPSX PERMISSION SYSTEM MONITORING SETUP ==="
    log_info "Environment: $ENVIRONMENT"
    log_info "Project: $PROJECT_ID"
    log_info "Region: $REGION"

    local phases=(
        "setup_monitoring_directory"
        "create_prometheus_config"
        "create_permission_metrics"
        "create_alerting_rules"
        "create_grafana_dashboard"
        "create_cloud_monitoring_config"
        "create_health_monitoring"
        "create_log_monitoring"
        "create_deployment_verification"
        "setup_monitoring_infrastructure"
        "create_alertmanager_config"
    )

    local phase_num=1
    local total_phases=${#phases[@]}

    for phase in "${phases[@]}"; do
        log_info "Phase $phase_num/$total_phases: $phase"
        $phase
        ((phase_num++))
    done

    # Setup completed
    log_success "=== MONITORING SETUP COMPLETED ==="

    # Next steps
    echo ""
    log_info "=== NEXT STEPS ==="
    echo "1. Update notification channel IDs in alert configurations"
    echo "2. Set up Slack webhook URL in environment variables"
    echo "3. Configure PagerDuty service key for critical alerts"
    echo "4. Deploy monitoring stack:"
    echo "   - Local: docker-compose -f monitoring/docker-compose.monitoring.yml up -d"
    echo "   - Production: Deploy to Google Cloud Monitoring"
    echo "5. Import Grafana dashboards from monitoring/grafana/dashboards/"
    echo "6. Test alerting rules and notification channels"
    echo "7. Verify deployment with monitoring/verify-deployment.sh"
    echo ""
    log_info "Monitoring configuration files created in: monitoring/"
    log_info "Production deployment can now proceed with full observability!"

    exit 0
}

# Execute setup
main "$@"