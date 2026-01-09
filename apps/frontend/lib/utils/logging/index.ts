/**
 * Logging and Monitoring Utilities
 * Consolidated logging, monitoring, and error handling utilities
 */

// ============================================================================
// Types
// ============================================================================

export interface LogLevel {
  level: 'debug' | 'info' | 'warn' | 'error';
  priority: number;
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel['level'];
  message: string;
  context?: string;
  data?: unknown;
  userId?: string;
  sessionId?: string;
  requestId?: string;
}

export interface MonitoringMetrics {
  responseTime: number;
  memoryUsage: number;
  cpuUsage: number;
  requestCount: number;
  errorCount: number;
  timestamp: number;
}

// ============================================================================
// Log Levels
// ============================================================================

const LOG_LEVELS: Record<string, LogLevel> = {
  DEBUG: { level: 'debug', priority: 0 },
  INFO: { level: 'info', priority: 1 },
  WARN: { level: 'warn', priority: 2 },
  ERROR: { level: 'error', priority: 3 }
};

// ============================================================================
// Logger Class
// ============================================================================

export class Logger {
  private context: string;
  private minLevel: LogLevel['level'];
  private entries: LogEntry[] = [];
  private maxEntries: number = 1000;

  constructor(context: string = 'App', minLevel: LogLevel['level'] = 'info') {
    this.context = context;
    this.minLevel = minLevel;
  }

  private shouldLog(level: LogLevel['level']): boolean {
    return LOG_LEVELS[level.toUpperCase()].priority >= LOG_LEVELS[this.minLevel.toUpperCase()].priority;
  }

  private createLogEntry(level: LogLevel['level'], message: string, data?: unknown): LogEntry {
    return {
      timestamp: new Date().toISOString(),
      level,
      message: this.sanitizeMessage(message),
      context: this.context,
      data: this.sanitizeData(data),
      // User/session identification disabled for security
      userId: undefined,
      sessionId: undefined,
      requestId: undefined
    };
  }

  private sanitizeMessage(message: string): string {
    if (!message) return message;

    // Remove common sensitive patterns
    return message
      .replace(/token[=:]\s*[^\s,}]+/gi, 'token=***')
      .replace(/password[=:]\s*[^\s,}]+/gi, 'password=***')
      .replace(/key[=:]\s*[^\s,}]+/gi, 'key=***')
      .replace(/secret[=:]\s*[^\s,}]+/gi, 'secret=***')
      .replace(/authorization[=:]\s*[^\s,}]+/gi, 'authorization=***')
      .replace(/bearer\s+[^\s,}]+/gi, 'bearer ***')
      .replace(/[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g, '***@***.***');
  }

  private sanitizeData(data: unknown): unknown {
    if (!data) return data;

    // In production, limit data logging for security
    if (process.env.NODE_ENV === 'production') {
      // Only log error messages and basic metadata
      if (data instanceof Error) {
        return {
          error: data.message,
          name: data.name
          // Stack traces excluded in production for security
        };
      }

      if (typeof data === 'object') {
        // Handle BigInt values and objects
        try {
          // Convert BigInt values to strings to prevent serialization errors
          const serialized = JSON.stringify(data, (key, value) =>
            typeof value === 'bigint' ? value.toString() + 'n' : value
          );
          return JSON.parse(serialized);
        } catch {
          // Fallback to safe field extraction
          const safeFields = ['status', 'code', 'type', 'category'];
          const sanitized: Record<string, unknown> = {};

          for (const field of safeFields) {
            const dataObj = data as Record<string, unknown>;
            if (field in dataObj && typeof dataObj[field] !== 'object' && typeof dataObj[field] !== 'bigint') {
              sanitized[field] = dataObj[field];
            }
          }

          return sanitized;
        }
      }

      // Handle BigInt primitive type
      if (typeof data === 'bigint') {
        return data.toString() + 'n';
      }

      // For primitive types, return as-is if not sensitive
      return typeof data === 'string' ? this.sanitizeMessage(data) : data;
    }

    // In development, still need to handle BigInt for safe logging
    // Handle BigInt primitive type
    if (typeof data === 'bigint') {
      return data.toString() + 'n';
    }

    // Handle objects that may contain BigInt values
    if (typeof data === 'object' && data !== null) {
      try {
        // Convert BigInt values to strings to prevent serialization errors
        const serialized = JSON.stringify(data, (key, value) =>
          typeof value === 'bigint' ? value.toString() + 'n' : value
        );
        return JSON.parse(serialized);
      } catch {
        // If JSON serialization fails, return the original data
        // The console will handle its own serialization
        return data;
      }
    }

    return data;
  }


  private log(level: LogLevel['level'], message: string, data?: unknown): void {
    if (!this.shouldLog(level)) return;

    // Skip logging if console is not available (SSR/hydration safety)
    if (typeof console === 'undefined') return;

    // Sanitize data to handle BigInt values before logging
    const safeData = data !== undefined ? this.sanitizeData(data) : undefined;

    // Simplified logging - console output only, no buffering or transmission
    const consoleMethod = level === 'debug' ? 'log' : level;
    const timestamp = new Date().toLocaleTimeString();
    const prefix = `[${timestamp}] [${level.toUpperCase()}] [${this.context}]`;

    // Safe console access with fallback to console.log
    // eslint-disable-next-line no-console
    const logFunction = console[consoleMethod as keyof Console] || console.log;
    if (typeof logFunction !== 'function') {
      // Fallback to console.log if specific method doesn't exist
      // eslint-disable-next-line no-console
      const fallbackLog = console.log;
      if (typeof fallbackLog === 'function') {
        if (safeData !== undefined) {
          fallbackLog(prefix, message, safeData);
        } else {
          fallbackLog(prefix, message);
        }
      }
      return;
    }

    try {
      if (safeData !== undefined) {
        (logFunction as Function)(prefix, message, safeData);
      } else {
        (logFunction as Function)(prefix, message);
      }
    } catch (_error) {
      // Fallback to console.log if specific method fails
      // eslint-disable-next-line no-console
      if (typeof console.log === 'function') {
        // eslint-disable-next-line no-console
        console.log(prefix, message, safeData);
      }
    }
  }

  private async sendToServer(_entry: LogEntry): Promise<void> {
    // Server transmission disabled for performance and security
    // Logs are kept local only
  }

  debug(message: string, data?: unknown): void {
    this.log('debug', message, data);
  }

  info(message: string, data?: unknown): void {
    this.log('info', message, data);
  }

  warn(message: string, data?: unknown): void {
    this.log('warn', message, data);
  }

  error(message: string, data?: unknown): void {
    this.log('error', message, data);
  }

  getEntries(_level?: LogLevel['level']): LogEntry[] {
    // Log buffering disabled - return empty array
    return [];
  }

  clearEntries(): void {
    // No entries to clear - buffering disabled
  }
}

// ============================================================================
// Specialized Loggers
// ============================================================================

// Environment-based logging controls
const isDevelopment = process.env.NODE_ENV === 'development';
const isProduction = process.env.NODE_ENV === 'production';

// Production: Only error logging for security and performance
// Development: Full debug logging
const productionLogLevel = 'error';
const developmentLogLevel = 'debug';

export const logger = new Logger('App', isDevelopment ? developmentLogLevel : productionLogLevel);
export const apiLogger = new Logger('API', isDevelopment ? developmentLogLevel : productionLogLevel);
export const authLogger = new Logger('Auth', isDevelopment ? developmentLogLevel : productionLogLevel);
export const analyticsLogger = new Logger('Analytics', isDevelopment ? developmentLogLevel : productionLogLevel);
export const uiLogger = new Logger('UI', isDevelopment ? developmentLogLevel : productionLogLevel);

// Development-only logging function
export const devLog = isDevelopment ? logger.debug.bind(logger) : () => { };

// Production-safe logging functions (no-ops in production)
export const devInfo = isDevelopment ? logger.info.bind(logger) : () => { };
export const devWarn = isDevelopment ? logger.warn.bind(logger) : () => { };

// Environment check utilities
export const isDevEnvironment = () => isDevelopment;
export const isProdEnvironment = () => isProduction;

// ============================================================================
// Error Handling
// ============================================================================

export interface SafeErrorResult {
  message: string;
  stack?: string;
  code?: string;
  status?: number;
}

export function safeError(error: unknown): SafeErrorResult {
  if (error instanceof Error) {
    return {
      message: error.message,
      stack: error.stack,
      code: 'name' in error ? error.name : undefined
    };
  }

  if (typeof error === 'string') {
    return { message: error };
  }

  if (error && typeof error === 'object') {
    const obj = error as Record<string, unknown>;
    return {
      message: (obj.message as string) || (obj.error as string) || JSON.stringify(error),
      stack: obj.stack as string | undefined,
      code: (obj.code as string | undefined) || (obj.status as string | undefined),
      status: (obj.status as number | undefined) || (obj.statusCode as number | undefined)
    };
  }

  return { message: 'Unknown error occurred' };
}

// ============================================================================
// Console Replacer
// ============================================================================

export class ConsoleReplacer {
  private originalConsole: Console;
  private isReplaced = false;

  constructor() {
    this.originalConsole = { ...console };
  }

  replace(): void {
    // Console replacement disabled for performance
    // Native console methods are used directly
  }

  restore(): void {
    if (!this.isReplaced) return;

    Object.assign(console, this.originalConsole);
    this.isReplaced = false;
  }
}

// ============================================================================
// Performance Monitoring
// ============================================================================

export class PerformanceMonitor {
  private metrics: MonitoringMetrics[] = [];
  private maxMetrics = 100;

  recordMetric(_metric: Partial<MonitoringMetrics>): void {
    // Performance monitoring disabled for better user experience
  }

  getMetrics(_since?: number): MonitoringMetrics[] {
    // Performance monitoring disabled - return empty array
    return [];
  }

  getAverageResponseTime(_since?: number): number {
    // Performance monitoring disabled - return 0
    return 0;
  }

  getErrorRate(_since?: number): number {
    // Performance monitoring disabled - return 0
    return 0;
  }

  measureFunction<T>(name: string, fn: () => T): T {
    // Performance measurement disabled - just execute function
    return fn();
  }

  async measureAsyncFunction<T>(name: string, fn: () => Promise<T>): Promise<T> {
    // Performance measurement disabled - just execute function
    return fn();
  }
}

// ============================================================================
// Streaming Utilities
// ============================================================================

export class StreamingLogger {
  // Streaming logger disabled for performance and security
  // All methods are no-ops to maintain compatibility

  constructor() {
    // No initialization - streaming disabled
  }

  log(_entry: LogEntry): void {
    // No-op - streaming disabled
  }

  private startFlushTimer(): void {
    // No-op - streaming disabled
  }

  private async flush(): Promise<void> {
    // No-op - streaming disabled
  }

  destroy(): void {
    // No-op - streaming disabled
  }
}

// ============================================================================
// Exports
// ============================================================================

export const consoleReplacer = new ConsoleReplacer();
export const performanceMonitor = new PerformanceMonitor();
export const streamingLogger = new StreamingLogger();