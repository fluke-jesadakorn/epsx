// Artillery.js Load Test Processor for EPSX Middleware Performance Testing

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

// Performance metrics tracking
let performanceMetrics = {
    middlewareLatency: [],
    sessionValidationLatency: [],
    permissionCheckLatency: [],
    cacheHitRatio: 0,
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0
};

// Generate test tokens and user data
function generateTestToken(context, events, done) {
    // Simulate valid JWT tokens for testing
    const testTokens = {
        adminToken: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbi11c2VyIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE2ODM5ODU2MDAsInBlcm1pc3Npb25zIjpbIkFETUlOX0ZVTEwiXSwicHJvZmlsZXMiOlsiYWRtaW4tZnVsbC0wMDQiXX0.test-admin-signature',
        userToken: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImlhdCI6MTUxNjIzOTAyMiwiZXhwIjoxNjgzOTg1NjAwLCJwZXJtaXNzaW9ucyI6WyJSRUFEX1BST0ZJTEUiLCJUUkFERV9TVE9DS1MiXSwicHJvZmlsZXMiOlsidXNlci1iYXNpYy0wMDEiXX0.test-user-signature'
    };
    
    context.vars.adminToken = testTokens.adminToken;
    context.vars.userToken = testTokens.userToken;
    
    return done();
}

// Generate large symbol list for payload testing
function generateLargeSymbolList(context, events, done) {
    const symbols = [];
    const commonSymbols = ['AAPL', 'GOOGL', 'MSFT', 'TSLA', 'AMZN', 'META', 'NVDA', 'NFLX'];
    
    // Generate 100 symbols for large payload testing
    for (let i = 0; i < 100; i++) {
        if (i < commonSymbols.length) {
            symbols.push(commonSymbols[i]);
        } else {
            symbols.push(`SYM${i.toString().padStart(3, '0')}`);
        }
    }
    
    context.vars.largeSymbolList = symbols;
    return done();
}

// Track middleware performance metrics
function trackMiddlewareLatency(context, ee, next) {
    return function(req, res, next) {
        const startTime = Date.now();
        
        res.on('finish', () => {
            const latency = Date.now() - startTime;
            performanceMetrics.middlewareLatency.push(latency);
            performanceMetrics.totalRequests++;
            
            if (res.statusCode < 400) {
                performanceMetrics.successfulRequests++;
            } else {
                performanceMetrics.failedRequests++;
            }
            
            // Check if response contains cache hit indicators
            if (res.headers['x-cache-status'] === 'hit') {
                performanceMetrics.cacheHitRatio++;
            }
            
            // Emit custom metrics for Artillery
            ee.emit('customStat', 'middleware_latency', latency);
            ee.emit('customStat', 'success_rate', 
                (performanceMetrics.successfulRequests / performanceMetrics.totalRequests) * 100);
        });
        
        next();
    };
}

// Validate performance SLAs
function validatePerformanceSLA(context, ee, next) {
    return function(req, res, next) {
        const latency = res.responseTime || 0;
        
        // Check P95 latency targets
        const p95Threshold = 10; // 10ms for middleware overhead
        if (latency > p95Threshold) {
            ee.emit('customStat', 'sla_violation', 1);
        }
        
        // Track session validation performance
        if (req.url.includes('/auth/session') || req.headers.authorization) {
            performanceMetrics.sessionValidationLatency.push(latency);
            
            const sessionSLAThreshold = 2; // 2ms for session validation
            if (latency > sessionSLAThreshold) {
                ee.emit('customStat', 'session_sla_violation', 1);
            }
        }
        
        // Track permission check performance
        if (req.url.includes('/admin/') && req.headers.authorization) {
            performanceMetrics.permissionCheckLatency.push(latency);
            
            const permissionSLAThreshold = 3; // 3ms for permission checks
            if (latency > permissionSLAThreshold) {
                ee.emit('customStat', 'permission_sla_violation', 1);
            }
        }
        
        next();
    };
}

// Generate realistic user behavior patterns
function simulateUserBehavior(context, events, done) {
    // Simulate different user types and their typical request patterns
    const userTypes = ['basic_user', 'premium_user', 'admin_user'];
    const selectedType = userTypes[Math.floor(Math.random() * userTypes.length)];
    
    context.vars.userType = selectedType;
    
    // Set appropriate tokens and permissions based on user type
    switch (selectedType) {
        case 'basic_user':
            context.vars.currentToken = context.vars.userToken;
            context.vars.permissions = ['READ_PROFILE', 'TRADE_BASIC'];
            break;
        case 'premium_user':
            context.vars.currentToken = context.vars.userToken;
            context.vars.permissions = ['READ_PROFILE', 'TRADE_ADVANCED', 'VIEW_ANALYTICS'];
            break;
        case 'admin_user':
            context.vars.currentToken = context.vars.adminToken;
            context.vars.permissions = ['ADMIN_FULL', 'USER_MANAGEMENT'];
            break;
    }
    
    return done();
}

// Simulate cache scenarios (hit/miss patterns)
function simulateCacheScenarios(context, events, done) {
    // Simulate realistic cache hit/miss patterns
    const cacheScenarios = ['cache_hit', 'cache_miss', 'cache_warm'];
    const scenario = cacheScenarios[Math.floor(Math.random() * cacheScenarios.length)];
    
    context.vars.cacheScenario = scenario;
    
    // Modify request patterns to test different cache behaviors
    if (scenario === 'cache_hit') {
        // Use consistent user IDs to hit existing cache entries
        context.vars.userId = `cached-user-${Math.floor(Math.random() * 10)}`;
    } else if (scenario === 'cache_miss') {
        // Use unique user IDs to force cache misses
        context.vars.userId = `unique-user-${crypto.randomBytes(4).toString('hex')}`;
    } else {
        // Mix of both for cache warming scenarios
        context.vars.userId = Math.random() > 0.5 ? 
            `cached-user-${Math.floor(Math.random() * 10)}` : 
            `unique-user-${crypto.randomBytes(4).toString('hex')}`;
    }
    
    return done();
}

// Memory pressure testing
function simulateMemoryPressure(context, events, done) {
    // Generate large payloads to test memory allocation under load
    const payloadSizes = ['small', 'medium', 'large', 'xlarge'];
    const size = payloadSizes[Math.floor(Math.random() * payloadSizes.length)];
    
    let payload = { type: 'memory_test', size: size };
    
    switch (size) {
        case 'small':
            payload.data = Array(100).fill().map((_, i) => ({ id: i, value: `data_${i}` }));
            break;
        case 'medium':
            payload.data = Array(500).fill().map((_, i) => ({ 
                id: i, 
                value: `data_${i}`, 
                metadata: Array(10).fill(`meta_${i}`) 
            }));
            break;
        case 'large':
            payload.data = Array(1000).fill().map((_, i) => ({ 
                id: i, 
                value: `data_${i}`, 
                metadata: Array(20).fill(`meta_${i}`),
                analytics: { trades: i * 10, volume: i * 100.50 }
            }));
            break;
        case 'xlarge':
            payload.data = Array(2000).fill().map((_, i) => ({ 
                id: i, 
                value: `data_${i}`, 
                metadata: Array(30).fill(`meta_${i}`),
                analytics: { 
                    trades: i * 10, 
                    volume: i * 100.50,
                    history: Array(50).fill({ timestamp: Date.now(), value: i })
                }
            }));
            break;
    }
    
    context.vars.memoryTestPayload = payload;
    return done();
}

// Database connection pool stress testing
function simulateDatabaseLoad(context, events, done) {
    // Simulate different types of database operations
    const dbOperations = ['read_heavy', 'write_heavy', 'mixed', 'complex_query'];
    const operation = dbOperations[Math.floor(Math.random() * dbOperations.length)];
    
    context.vars.dbOperation = operation;
    
    // Set query parameters based on operation type
    switch (operation) {
        case 'read_heavy':
            context.vars.queryLimit = 50;
            context.vars.queryOffset = Math.floor(Math.random() * 1000);
            break;
        case 'write_heavy':
            context.vars.updateCount = Math.floor(Math.random() * 10) + 1;
            break;
        case 'mixed':
            context.vars.queryLimit = 25;
            context.vars.updateCount = Math.floor(Math.random() * 5) + 1;
            break;
        case 'complex_query':
            context.vars.dateRange = {
                start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
                end: new Date().toISOString()
            };
            break;
    }
    
    return done();
}

// Generate performance report
function generatePerformanceReport(context, events, done) {
    if (performanceMetrics.totalRequests > 0) {
        const report = {
            timestamp: new Date().toISOString(),
            totalRequests: performanceMetrics.totalRequests,
            successRate: (performanceMetrics.successfulRequests / performanceMetrics.totalRequests) * 100,
            middlewareLatency: {
                p50: calculatePercentile(performanceMetrics.middlewareLatency, 50),
                p95: calculatePercentile(performanceMetrics.middlewareLatency, 95),
                p99: calculatePercentile(performanceMetrics.middlewareLatency, 99),
                avg: performanceMetrics.middlewareLatency.reduce((a, b) => a + b, 0) / performanceMetrics.middlewareLatency.length
            },
            sessionValidationLatency: {
                p50: calculatePercentile(performanceMetrics.sessionValidationLatency, 50),
                p95: calculatePercentile(performanceMetrics.sessionValidationLatency, 95),
                p99: calculatePercentile(performanceMetrics.sessionValidationLatency, 99),
                avg: performanceMetrics.sessionValidationLatency.length > 0 ? 
                    performanceMetrics.sessionValidationLatency.reduce((a, b) => a + b, 0) / performanceMetrics.sessionValidationLatency.length : 0
            },
            permissionCheckLatency: {
                p50: calculatePercentile(performanceMetrics.permissionCheckLatency, 50),
                p95: calculatePercentile(performanceMetrics.permissionCheckLatency, 95),
                p99: calculatePercentile(performanceMetrics.permissionCheckLatency, 99),
                avg: performanceMetrics.permissionCheckLatency.length > 0 ? 
                    performanceMetrics.permissionCheckLatency.reduce((a, b) => a + b, 0) / performanceMetrics.permissionCheckLatency.length : 0
            },
            cacheHitRatio: performanceMetrics.totalRequests > 0 ? 
                (performanceMetrics.cacheHitRatio / performanceMetrics.totalRequests) * 100 : 0,
            slaCompliance: {
                middlewareOverhead: performanceMetrics.middlewareLatency.filter(l => l < 10).length / performanceMetrics.middlewareLatency.length * 100,
                sessionValidation: performanceMetrics.sessionValidationLatency.filter(l => l < 2).length / Math.max(performanceMetrics.sessionValidationLatency.length, 1) * 100,
                permissionChecks: performanceMetrics.permissionCheckLatency.filter(l => l < 3).length / Math.max(performanceMetrics.permissionCheckLatency.length, 1) * 100
            }
        };
        
        // Write report to file
        const reportPath = path.join(__dirname, '../test-results', `performance-report-${Date.now()}.json`);
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        
        console.log('Performance Report:', report);
    }
    
    return done();
}

// Helper function to calculate percentiles
function calculatePercentile(arr, percentile) {
    if (arr.length === 0) return 0;
    
    const sorted = [...arr].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[index] || 0;
}

// Connection pool stress testing
function testConnectionPool(context, events, done) {
    // Simulate high concurrent database connections
    context.vars.concurrentConnections = Math.floor(Math.random() * 30) + 1;
    context.vars.connectionHoldTime = Math.floor(Math.random() * 1000) + 100; // 100-1100ms
    
    return done();
}

// Rate limiting test scenarios
function testRateLimiting(context, events, done) {
    // Test rate limiting under different load patterns
    const rateLimitScenarios = ['normal', 'burst', 'sustained_high', 'spike'];
    const scenario = rateLimitScenarios[Math.floor(Math.random() * rateLimitScenarios.length)];
    
    context.vars.rateLimitScenario = scenario;
    
    // Adjust request timing based on scenario
    switch (scenario) {
        case 'burst':
            // Rapid consecutive requests
            context.vars.requestDelay = 0;
            break;
        case 'sustained_high':
            // High but manageable rate
            context.vars.requestDelay = Math.floor(Math.random() * 50) + 10;
            break;
        case 'spike':
            // Sudden spike in requests
            context.vars.requestDelay = Math.random() > 0.8 ? 0 : 100;
            break;
        default:
            // Normal request pattern
            context.vars.requestDelay = Math.floor(Math.random() * 200) + 50;
    }
    
    return done();
}

// Export all processor functions
module.exports = {
    generateTestToken,
    generateLargeSymbolList,
    trackMiddlewareLatency,
    validatePerformanceSLA,
    simulateUserBehavior,
    simulateCacheScenarios,
    simulateMemoryPressure,
    simulateDatabaseLoad,
    generatePerformanceReport,
    testConnectionPool,
    testRateLimiting
};