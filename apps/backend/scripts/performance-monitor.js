#!/usr/bin/env node
/**
 * EPSX Performance Monitoring & Alerting System
 * 
 * Real-time performance monitoring for middleware stack with SLA validation
 * and automated alerting when performance degrades below thresholds.
 */

const http = require('http');
const https = require('https');
const WebSocket = require('ws');
const fs = require('fs').promises;
const path = require('path');

class PerformanceMonitor {
    constructor(config = {}) {
        this.config = {
            // Performance SLA thresholds
            sla: {
                middlewareLatencyP95: 10,      // 10ms P95 latency
                sessionValidationP95: 2,       // 2ms P95 session validation
                permissionCheckP95: 3,         // 3ms P95 permission checks
                cacheHitRatio: 95,            // 95% cache hit ratio
                successRate: 99.9,            // 99.9% success rate
                memoryUsage: 2048,            // 2GB memory limit
                cpuUsage: 80,                 // 80% CPU limit
                connectionPoolEfficiency: 90  // 90% connection pool efficiency
            },
            
            // Monitoring endpoints
            endpoints: {
                backend: 'http://localhost:8080',
                frontend: 'http://localhost:3000',
                admin: 'http://localhost:3001'
            },
            
            // Monitoring intervals
            intervals: {
                healthCheck: 5000,        // 5 seconds
                performanceMetrics: 10000, // 10 seconds
                alertCheck: 15000,        // 15 seconds
                reportGeneration: 60000   // 1 minute
            },
            
            // Alert configuration
            alerts: {
                webhooks: [], // Webhook URLs for alerts
                email: {
                    enabled: false,
                    recipients: []
                },
                slack: {
                    enabled: false,
                    webhook: null
                }
            },
            
            ...config
        };
        
        this.metrics = {
            requests: {
                total: 0,
                successful: 0,
                failed: 0,
                latencies: []
            },
            middleware: {
                latencies: [],
                sessionValidation: [],
                permissionChecks: [],
                cacheOperations: []
            },
            system: {
                memory: [],
                cpu: [],
                connections: []
            },
            alerts: []
        };
        
        this.isMonitoring = false;
        this.intervals = {};
    }
    
    // Start monitoring
    async start() {
        if (this.isMonitoring) {
            console.log('Performance monitor is already running');
            return;
        }
        
        console.log('Starting EPSX Performance Monitor...');
        this.isMonitoring = true;
        
        // Start monitoring intervals
        this.intervals.healthCheck = setInterval(() => this.checkHealth(), this.config.intervals.healthCheck);
        this.intervals.performanceMetrics = setInterval(() => this.collectMetrics(), this.config.intervals.performanceMetrics);
        this.intervals.alertCheck = setInterval(() => this.checkAlerts(), this.config.intervals.alertCheck);
        this.intervals.reportGeneration = setInterval(() => this.generateReport(), this.config.intervals.reportGeneration);
        
        console.log('Performance monitoring started');
        console.log(`SLA Thresholds: Middleware P95: ${this.config.sla.middlewareLatencyP95}ms, Success Rate: ${this.config.sla.successRate}%`);
    }
    
    // Stop monitoring
    stop() {
        if (!this.isMonitoring) {
            return;
        }
        
        console.log('Stopping performance monitor...');
        this.isMonitoring = false;
        
        // Clear all intervals
        Object.values(this.intervals).forEach(interval => clearInterval(interval));
        this.intervals = {};
        
        console.log('Performance monitoring stopped');
    }
    
    // Health check for all services
    async checkHealth() {
        for (const [service, endpoint] of Object.entries(this.config.endpoints)) {
            try {
                const startTime = Date.now();
                const response = await this.makeRequest(`${endpoint}/health`);
                const latency = Date.now() - startTime;
                
                this.metrics.requests.total++;
                
                if (response.status < 400) {
                    this.metrics.requests.successful++;
                    this.metrics.requests.latencies.push({
                        service,
                        latency,
                        timestamp: Date.now()
                    });
                } else {
                    this.metrics.requests.failed++;
                    console.warn(`Health check failed for ${service}: ${response.status}`);
                }
                
            } catch (error) {
                this.metrics.requests.failed++;
                console.error(`Health check error for ${service}:`, error.message);
            }
        }
        
        // Keep only recent latency data (last 1000 entries)
        if (this.metrics.requests.latencies.length > 1000) {
            this.metrics.requests.latencies = this.metrics.requests.latencies.slice(-1000);
        }
    }
    
    // Collect detailed performance metrics
    async collectMetrics() {
        try {
            // Collect backend performance metrics
            const backendMetrics = await this.getBackendMetrics();
            if (backendMetrics) {
                this.processBackendMetrics(backendMetrics);
            }
            
            // Collect system metrics
            const systemMetrics = await this.getSystemMetrics();
            if (systemMetrics) {
                this.processSystemMetrics(systemMetrics);
            }
            
            // Clean old metrics data
            this.cleanOldMetrics();
            
        } catch (error) {
            console.error('Error collecting performance metrics:', error);
        }
    }
    
    // Get backend-specific performance metrics
    async getBackendMetrics() {
        try {
            const response = await this.makeRequest(`${this.config.endpoints.backend}/api/internal/metrics`);
            if (response.status === 200) {
                return response.data;
            }
        } catch (error) {
            // Metrics endpoint might not be available, that's okay
            return null;
        }
    }
    
    // Get system metrics
    async getSystemMetrics() {
        try {
            const response = await this.makeRequest(`${this.config.endpoints.backend}/api/internal/system-metrics`);
            if (response.status === 200) {
                return response.data;
            }
        } catch (error) {
            return null;
        }
    }
    
    // Process backend metrics
    processBackendMetrics(metrics) {
        if (metrics.middleware_latency) {
            this.metrics.middleware.latencies.push({
                value: metrics.middleware_latency,
                timestamp: Date.now()
            });
        }
        
        if (metrics.session_validation_latency) {
            this.metrics.middleware.sessionValidation.push({
                value: metrics.session_validation_latency,
                timestamp: Date.now()
            });
        }
        
        if (metrics.permission_check_latency) {
            this.metrics.middleware.permissionChecks.push({
                value: metrics.permission_check_latency,
                timestamp: Date.now()
            });
        }
        
        if (metrics.cache_operations) {
            this.metrics.middleware.cacheOperations.push({
                hit_ratio: metrics.cache_operations.hit_ratio,
                avg_latency: metrics.cache_operations.avg_latency,
                timestamp: Date.now()
            });
        }
    }
    
    // Process system metrics
    processSystemMetrics(metrics) {
        if (metrics.memory_usage) {
            this.metrics.system.memory.push({
                value: metrics.memory_usage,
                timestamp: Date.now()
            });
        }
        
        if (metrics.cpu_usage) {
            this.metrics.system.cpu.push({
                value: metrics.cpu_usage,
                timestamp: Date.now()
            });
        }
        
        if (metrics.active_connections) {
            this.metrics.system.connections.push({
                value: metrics.active_connections,
                timestamp: Date.now()
            });
        }
    }
    
    // Check for SLA violations and trigger alerts
    async checkAlerts() {
        const violations = [];
        
        // Check middleware latency P95
        const middlewareP95 = this.calculatePercentile(
            this.metrics.middleware.latencies.map(m => m.value), 95
        );
        if (middlewareP95 > this.config.sla.middlewareLatencyP95) {
            violations.push({
                type: 'middleware_latency_p95',
                current: middlewareP95,
                threshold: this.config.sla.middlewareLatencyP95,
                severity: 'critical'
            });
        }
        
        // Check session validation latency P95
        const sessionP95 = this.calculatePercentile(
            this.metrics.middleware.sessionValidation.map(m => m.value), 95
        );
        if (sessionP95 > this.config.sla.sessionValidationP95) {
            violations.push({
                type: 'session_validation_p95',
                current: sessionP95,
                threshold: this.config.sla.sessionValidationP95,
                severity: 'high'
            });
        }
        
        // Check permission check latency P95
        const permissionP95 = this.calculatePercentile(
            this.metrics.middleware.permissionChecks.map(m => m.value), 95
        );
        if (permissionP95 > this.config.sla.permissionCheckP95) {
            violations.push({
                type: 'permission_check_p95',
                current: permissionP95,
                threshold: this.config.sla.permissionCheckP95,
                severity: 'high'
            });
        }
        
        // Check success rate
        const successRate = this.metrics.requests.total > 0 ? 
            (this.metrics.requests.successful / this.metrics.requests.total) * 100 : 100;
        if (successRate < this.config.sla.successRate) {
            violations.push({
                type: 'success_rate',
                current: successRate,
                threshold: this.config.sla.successRate,
                severity: 'critical'
            });
        }
        
        // Check memory usage
        if (this.metrics.system.memory.length > 0) {
            const latestMemory = this.metrics.system.memory[this.metrics.system.memory.length - 1].value;
            if (latestMemory > this.config.sla.memoryUsage) {
                violations.push({
                    type: 'memory_usage',
                    current: latestMemory,
                    threshold: this.config.sla.memoryUsage,
                    severity: 'high'
                });
            }
        }
        
        // Check CPU usage
        if (this.metrics.system.cpu.length > 0) {
            const latestCPU = this.metrics.system.cpu[this.metrics.system.cpu.length - 1].value;
            if (latestCPU > this.config.sla.cpuUsage) {
                violations.push({
                    type: 'cpu_usage',
                    current: latestCPU,
                    threshold: this.config.sla.cpuUsage,
                    severity: 'medium'
                });
            }
        }
        
        // Check cache hit ratio
        if (this.metrics.middleware.cacheOperations.length > 0) {
            const latestCache = this.metrics.middleware.cacheOperations[this.metrics.middleware.cacheOperations.length - 1];
            if (latestCache.hit_ratio < this.config.sla.cacheHitRatio) {
                violations.push({
                    type: 'cache_hit_ratio',
                    current: latestCache.hit_ratio,
                    threshold: this.config.sla.cacheHitRatio,
                    severity: 'medium'
                });
            }
        }
        
        // Process violations
        for (const violation of violations) {
            await this.handleAlert(violation);
        }
    }
    
    // Handle performance alert
    async handleAlert(violation) {
        const alert = {
            id: `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            timestamp: new Date().toISOString(),
            ...violation
        };
        
        this.metrics.alerts.push(alert);
        
        console.warn(`🚨 PERFORMANCE ALERT [${alert.severity.toUpperCase()}]: ${alert.type}`);
        console.warn(`   Current: ${alert.current}, Threshold: ${alert.threshold}`);
        
        // Send alert notifications
        await this.sendAlertNotifications(alert);
        
        // Keep only recent alerts
        if (this.metrics.alerts.length > 100) {
            this.metrics.alerts = this.metrics.alerts.slice(-100);
        }
    }
    
    // Send alert notifications
    async sendAlertNotifications(alert) {
        // Send webhook notifications
        for (const webhook of this.config.alerts.webhooks) {
            try {
                await this.makeRequest(webhook, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        alert_type: 'performance_sla_violation',
                        service: 'epsx_backend',
                        ...alert
                    })
                });
            } catch (error) {
                console.error('Failed to send webhook alert:', error.message);
            }
        }
        
        // Send Slack notification if configured
        if (this.config.alerts.slack.enabled && this.config.alerts.slack.webhook) {
            try {
                await this.makeRequest(this.config.alerts.slack.webhook, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        text: `🚨 EPSX Performance Alert: ${alert.type}`,
                        attachments: [{
                            color: alert.severity === 'critical' ? 'danger' : 'warning',
                            fields: [
                                { title: 'Metric', value: alert.type, short: true },
                                { title: 'Current Value', value: alert.current.toString(), short: true },
                                { title: 'Threshold', value: alert.threshold.toString(), short: true },
                                { title: 'Severity', value: alert.severity, short: true }
                            ],
                            timestamp: alert.timestamp
                        }]
                    })
                });
            } catch (error) {
                console.error('Failed to send Slack alert:', error.message);
            }
        }
    }
    
    // Generate performance report
    async generateReport() {
        const report = {
            timestamp: new Date().toISOString(),
            summary: {
                totalRequests: this.metrics.requests.total,
                successfulRequests: this.metrics.requests.successful,
                failedRequests: this.metrics.requests.failed,
                successRate: this.metrics.requests.total > 0 ? 
                    (this.metrics.requests.successful / this.metrics.requests.total) * 100 : 0,
                activeAlerts: this.metrics.alerts.filter(a => 
                    Date.now() - new Date(a.timestamp).getTime() < 300000 // Last 5 minutes
                ).length
            },
            middleware: {
                latencyP50: this.calculatePercentile(this.metrics.middleware.latencies.map(m => m.value), 50),
                latencyP95: this.calculatePercentile(this.metrics.middleware.latencies.map(m => m.value), 95),
                latencyP99: this.calculatePercentile(this.metrics.middleware.latencies.map(m => m.value), 99),
                sessionValidationP95: this.calculatePercentile(this.metrics.middleware.sessionValidation.map(m => m.value), 95),
                permissionCheckP95: this.calculatePercentile(this.metrics.middleware.permissionChecks.map(m => m.value), 95),
                cacheHitRatio: this.metrics.middleware.cacheOperations.length > 0 ?
                    this.metrics.middleware.cacheOperations[this.metrics.middleware.cacheOperations.length - 1].hit_ratio : 0
            },
            system: {
                memoryUsage: this.metrics.system.memory.length > 0 ?
                    this.metrics.system.memory[this.metrics.system.memory.length - 1].value : 0,
                cpuUsage: this.metrics.system.cpu.length > 0 ?
                    this.metrics.system.cpu[this.metrics.system.cpu.length - 1].value : 0,
                activeConnections: this.metrics.system.connections.length > 0 ?
                    this.metrics.system.connections[this.metrics.system.connections.length - 1].value : 0
            },
            slaCompliance: {
                middlewareLatency: this.calculateSLACompliance(
                    this.metrics.middleware.latencies.map(m => m.value),
                    this.config.sla.middlewareLatencyP95, 95
                ),
                sessionValidation: this.calculateSLACompliance(
                    this.metrics.middleware.sessionValidation.map(m => m.value),
                    this.config.sla.sessionValidationP95, 95
                ),
                permissionChecks: this.calculateSLACompliance(
                    this.metrics.middleware.permissionChecks.map(m => m.value),
                    this.config.sla.permissionCheckP95, 95
                ),
                successRate: report.summary.successRate >= this.config.sla.successRate
            }
        };
        
        // Write report to file
        const reportDir = path.join(__dirname, '../test-results');
        await fs.mkdir(reportDir, { recursive: true });
        const reportPath = path.join(reportDir, `performance-monitor-${Date.now()}.json`);
        await fs.writeFile(reportPath, JSON.stringify(report, null, 2));
        
        // Log summary
        console.log(`📊 Performance Report Generated:`);
        console.log(`   Success Rate: ${report.summary.successRate.toFixed(2)}%`);
        console.log(`   Middleware P95: ${report.middleware.latencyP95}ms`);
        console.log(`   Active Alerts: ${report.summary.activeAlerts}`);
        console.log(`   Report saved: ${reportPath}`);
        
        return report;
    }
    
    // Calculate percentile
    calculatePercentile(arr, percentile) {
        if (arr.length === 0) return 0;
        const sorted = [...arr].sort((a, b) => a - b);
        const index = Math.ceil((percentile / 100) * sorted.length) - 1;
        return sorted[Math.max(0, index)] || 0;
    }
    
    // Calculate SLA compliance percentage
    calculateSLACompliance(values, threshold, percentile) {
        if (values.length === 0) return 100;
        const pValue = this.calculatePercentile(values, percentile);
        return pValue <= threshold ? 100 : 0;
    }
    
    // Make HTTP request
    async makeRequest(url, options = {}) {
        return new Promise((resolve, reject) => {
            const urlObj = new URL(url);
            const isHttps = urlObj.protocol === 'https:';
            const lib = isHttps ? https : http;
            
            const requestOptions = {
                hostname: urlObj.hostname,
                port: urlObj.port || (isHttps ? 443 : 80),
                path: urlObj.pathname + urlObj.search,
                method: options.method || 'GET',
                headers: options.headers || {},
                timeout: options.timeout || 5000
            };
            
            const req = lib.request(requestOptions, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    try {
                        const parsedData = data ? JSON.parse(data) : null;
                        resolve({ status: res.statusCode, data: parsedData });
                    } catch (error) {
                        resolve({ status: res.statusCode, data: data });
                    }
                });
            });
            
            req.on('error', reject);
            req.on('timeout', () => reject(new Error('Request timeout')));
            
            if (options.body) {
                req.write(options.body);
            }
            
            req.end();
        });
    }
    
    // Clean old metrics data
    cleanOldMetrics() {
        const maxAge = 5 * 60 * 1000; // 5 minutes
        const cutoff = Date.now() - maxAge;
        
        // Clean middleware metrics
        this.metrics.middleware.latencies = this.metrics.middleware.latencies.filter(m => m.timestamp > cutoff);
        this.metrics.middleware.sessionValidation = this.metrics.middleware.sessionValidation.filter(m => m.timestamp > cutoff);
        this.metrics.middleware.permissionChecks = this.metrics.middleware.permissionChecks.filter(m => m.timestamp > cutoff);
        this.metrics.middleware.cacheOperations = this.metrics.middleware.cacheOperations.filter(m => m.timestamp > cutoff);
        
        // Clean system metrics
        this.metrics.system.memory = this.metrics.system.memory.filter(m => m.timestamp > cutoff);
        this.metrics.system.cpu = this.metrics.system.cpu.filter(m => m.timestamp > cutoff);
        this.metrics.system.connections = this.metrics.system.connections.filter(m => m.timestamp > cutoff);
        
        // Clean request latencies
        this.metrics.requests.latencies = this.metrics.requests.latencies.filter(m => m.timestamp > cutoff);
    }
}

// CLI usage
if (require.main === module) {
    const monitor = new PerformanceMonitor({
        alerts: {
            slack: {
                enabled: process.env.SLACK_WEBHOOK_URL ? true : false,
                webhook: process.env.SLACK_WEBHOOK_URL
            },
            webhooks: process.env.ALERT_WEBHOOKS ? process.env.ALERT_WEBHOOKS.split(',') : []
        }
    });
    
    // Handle graceful shutdown
    process.on('SIGTERM', () => monitor.stop());
    process.on('SIGINT', () => monitor.stop());
    
    monitor.start().catch(console.error);
}

module.exports = PerformanceMonitor;