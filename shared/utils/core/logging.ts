/**
 * SHARED LOGGING UTILITIES
 * Consolidated logging utilities to replace app-specific implementations
 * Provides secure, performance-optimized logging for all applications
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
  data?: any;
  userId?: string;
  sessionId?: string;
  requestId?: string;
}

export interface SafeErrorResult {
  message: string;
  stack?: string;
  code?: string;
  status?: number;
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
  private maxEntries: number = 1000;

  constructor(context: string = 'App', minLevel: LogLevel['level'] = 'info') {
    this.context = context;
    this.minLevel = minLevel;
  }

  private shouldLog(level: LogLevel['level']): boolean {
    return LOG_LEVELS[level.toUpperCase()].priority >= LOG_LEVELS[this.minLevel.toUpperCase()].priority;
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

  private sanitizeData(data: any): any {
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
        // Only include safe fields
        const safeFields = ['status', 'code', 'type', 'category'];
        const sanitized: any = {};
        
        for (const field of safeFields) {
          if (field in data && typeof data[field] !== 'object') {
            sanitized[field] = data[field];
          }
        }
        
        return sanitized;
      }
      
      // For primitive types, return as-is if not sensitive
      return typeof data === 'string' ? this.sanitizeMessage(data) : data;
    }
    
    // In development, return full data for debugging
    return data;
  }

  private log(level: LogLevel['level'], message: string, data?: any): void {
    if (!this.shouldLog(level)) return;

    // Simplified logging - console output only, no buffering or transmission
    const consoleMethod = level === 'debug' ? 'log' : level;
    const timestamp = new Date().toLocaleTimeString();
    const prefix = `[${timestamp}] [${level.toUpperCase()}] [${this.context}]`;
    
    if (data) {
      console[consoleMethod](prefix, message, data);
    } else {
      console[consoleMethod](prefix, message);
    }
  }

  debug(message: string, data?: any): void {
    this.log('debug', message, data);
  }

  info(message: string, data?: any): void {
    this.log('info', message, data);
  }

  warn(message: string, data?: any): void {
    this.log('warn', message, data);
  }

  error(message: string, data?: any): void {
    this.log('error', message, data);
  }
}

// ============================================================================
// Error Handling
// ============================================================================

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
    const obj = error as any;
    return {
      message: obj.message || obj.error || JSON.stringify(error),
      stack: obj.stack,
      code: obj.code || obj.status,
      status: obj.status || obj.statusCode
    };
  }

  return { message: 'Unknown error occurred' };
}

// ============================================================================
// Shared Logger Instances
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
export const devLog = isDevelopment ? logger.debug.bind(logger) : () => {};

// Production-safe logging functions (no-ops in production)
export const devInfo = isDevelopment ? logger.info.bind(logger) : () => {};
export const devWarn = isDevelopment ? logger.warn.bind(logger) : () => {};

// Environment check utilities
export const isDevEnvironment = () => isDevelopment;
export const isProdEnvironment = () => isProduction;