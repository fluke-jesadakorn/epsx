/**
 * Health Check API Documentation
 * 
 * Base Path: /health
 * 
 * Endpoints:
 * GET /              - Overall system health status
 * GET /database      - Database health details (Admin only)
 * GET /readiness     - Application readiness check
 * GET /liveness      - Application liveness check
 * 
 * Protected Endpoints (Admin only):
 * GET /database/details  - Detailed database health information
 * GET /database/events   - Database connection events history
 */

export * from './health.controller';

// Health check response types
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';
export type ReadinessStatus = 'ready' | 'not_ready';
export type ConnectionType = 'connected' | 'disconnected' | 'reconnected' | 'error';

// HTTP status codes used by health check endpoints
export const enum HealthCheckHttpStatus {
  OK = 200,
  SERVICE_UNAVAILABLE = 503,
  UNAUTHORIZED = 401,
  FORBIDDEN = 403
}

// Default configuration values
export const HEALTH_CHECK_DEFAULTS = {
  INTERVAL: 30000,          // 30 seconds
  MAX_EVENTS: 100,          // Maximum number of connection events to store
  TIMEOUT: 5000,            // 5 seconds timeout for health checks
  DEGRADED_THRESHOLD: 1000, // Response time threshold for degraded status (ms)
  QUEUE_THRESHOLD: 100      // Queue size threshold for degraded status
} as const;

// URL paths for health check endpoints
export const HEALTH_CHECK_PATHS = {
  BASE: '/health',
  DATABASE: '/health/database',
  DATABASE_DETAILS: '/health/database/details',
  DATABASE_EVENTS: '/health/database/events',
  READINESS: '/health/readiness',
  LIVENESS: '/health/liveness'
} as const;

// Swagger documentation tags
export const HEALTH_CHECK_TAGS = {
  HEALTH: 'Health',
  DATABASE: 'Database Health',
  MONITORING: 'System Monitoring'
} as const;
