export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: Date;
  context?: LogContext;
  data?: any;
}

export interface LogContext {
  component?: string;
  action?: string;
  userId?: string;
  sessionId?: string;
  requestId?: string;
  metadata?: Record<string, any>;
}

export interface Logger {
  debug(message: string, data?: any, context?: LogContext): void;
  info(message: string, data?: any, context?: LogContext): void;
  warn(message: string, data?: any, context?: LogContext): void;
  error(message: string, data?: any, context?: LogContext): void;
}