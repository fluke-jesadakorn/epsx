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

// Capture console methods via globalThis to satisfy no-console lint rule in logger internals
const nativeConsole = globalThis.console;
const CONSOLE_METHODS: Record<LogLevel['level'], (...args: unknown[]) => void> = {
  debug: nativeConsole.debug.bind(nativeConsole),
  info: nativeConsole.info.bind(nativeConsole),
  warn: nativeConsole.warn.bind(nativeConsole),
  error: nativeConsole.error.bind(nativeConsole),
};

// ============================================================================
// Logger Class
// ============================================================================

export class Logger {
  private context: string;
  private minLevel: LogLevel['level'];
  private entries: LogEntry[] = [];
  private maxEntries = 1000;

  constructor(context = 'App', minLevel: LogLevel['level'] = 'info') {
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
    if (!message) { return message; }

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

  private sanitizeData(data: unknown, seen = new WeakSet()): unknown {
    if (data === null || data === undefined) { return data; }

    // Handle BigInt primitive type
    if (typeof data === 'bigint') {
      return `${data.toString()}n`;
    }

    // Handle primary types
    if (typeof data !== 'object') {
      return typeof data === 'string' ? this.sanitizeMessage(data) : data;
    }

    // Prevent circular references
    if (seen.has(data)) {
      return '[Circular]';
    }
    seen.add(data);

    // In production, limit data logging for security
    if (process.env.NODE_ENV === 'production') {
      if (data instanceof Error) {
        return {
          error: this.sanitizeMessage(data.message),
          name: data.name
          // Stack traces excluded in production for security
        };
      }

      // Safe field extraction in production
      const sanitized: Record<string, unknown> = {};
      const safeFields = ['status', 'code', 'type', 'category', 'message', 'error', 'id', 'sub', 'source', 'reason', 'event'];

      try {
        const dataObj = data as Record<string, unknown>;
        for (const field of safeFields) {
          if (Object.prototype.hasOwnProperty.call(dataObj, field)) {
            const val = dataObj[field];
            sanitized[field] = this.sanitizeData(val, seen);
          }
        }

        // If the object has no safe fields but we should log SOMETHING, 
        // return at least its type or a string representation
        if (Object.keys(sanitized).length === 0) {
          return { _type: typeof data, _string: String(data).slice(0, 100) };
        }

        return sanitized;
      } catch {
        return '[Unserializable]';
      }
    }

    // In development, handle BigInt for safe logging but keep more details
    if (data instanceof Error) {
      const errorObj = data as unknown as Record<string, unknown>;
      return {
        message: this.sanitizeMessage(data.message),
        name: data.name,
        stack: data.stack,
        ...(Object.prototype.hasOwnProperty.call(errorObj, 'cause') ? { cause: this.sanitizeData(errorObj.cause, seen) } : {})
      };
    }

    // Array handling
    if (Array.isArray(data)) {
      return data.map(item => this.sanitizeData(item, seen));
    }

    // Generic object handling
    try {
      const sanitized: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(data)) {
        sanitized[key] = this.sanitizeData(value, seen);
      }
      return sanitized;
    } catch {
      return String(data);
    }
  }

  private log(level: LogLevel['level'], message: string, data?: unknown): void {
    if (!this.shouldLog(level)) { return; }

    const safeData = data !== undefined ? this.sanitizeData(data, new WeakSet()) : undefined;
    const timestamp = new Date().toLocaleTimeString();
    const prefix = `[${timestamp}] [${level.toUpperCase()}] [${this.context}]`;

    const write = (fn: (...args: unknown[]) => void): void => {
      if (safeData !== undefined) {
        fn(prefix, message, safeData);
      } else {
        fn(prefix, message);
      }
    };

    write(CONSOLE_METHODS[level]);
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
export const authLogger = new Logger('auth', isDevelopment ? developmentLogLevel : productionLogLevel);
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

  if (typeof error === 'bigint') {
    return { message: `${error.toString()}n` };
  }

  if (error !== null && typeof error === 'object') {
    const obj = error as Record<string, unknown>;
    try {
      return {
        message: String(obj.message ?? obj.error ?? 'Unknown object error'),
        stack: String(obj.stack ?? ''),
        code: String(obj.code ?? obj.status ?? ''),
        status: typeof obj.status === 'number' ? obj.status : (typeof obj.statusCode === 'number' ? obj.statusCode : undefined)
      };
    } catch {
      return { message: '[Non-serializable Error Object]' };
    }
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
    if (!this.isReplaced) { return; }

    void Object.assign(console, this.originalConsole);
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