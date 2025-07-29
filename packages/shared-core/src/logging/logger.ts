import type { Logger, LogLevel, LogEntry, LogContext } from './types';

export class AppLogger implements Logger {
  private minLevel: LogLevel;
  private isServer: boolean;

  constructor(minLevel: LogLevel = 'info') {
    this.minLevel = minLevel;
    this.isServer = typeof window === 'undefined';
  }

  private shouldLog(level: LogLevel): boolean {
    const levels: Record<LogLevel, number> = {
      debug: 0,
      info: 1,
      warn: 2,
      error: 3
    };
    return levels[level] >= levels[this.minLevel];
  }

  private formatMessage(entry: LogEntry): string {
    const prefix = this.isServer ? '[Server]' : '[Client]';
    const timestamp = entry.timestamp.toISOString();
    const context = entry.context ? ` [${entry.context.component || 'Unknown'}]` : '';
    return `${prefix}${context} ${timestamp} [${entry.level.toUpperCase()}] ${entry.message}`;
  }

  private log(level: LogLevel, message: string, data?: any, context?: LogContext): void {
    if (!this.shouldLog(level)) return;

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date(),
      context,
      data
    };

    const formattedMessage = this.formatMessage(entry);

    switch (level) {
      case 'debug':
        console.debug(formattedMessage, data || '');
        break;
      case 'info':
        console.info(formattedMessage, data || '');
        break;
      case 'warn':
        console.warn(formattedMessage, data || '');
        break;
      case 'error':
        console.error(formattedMessage, data || '');
        break;
    }
  }

  debug(message: string, data?: any, context?: LogContext): void {
    this.log('debug', message, data, context);
  }

  info(message: string, data?: any, context?: LogContext): void {
    this.log('info', message, data, context);
  }

  warn(message: string, data?: any, context?: LogContext): void {
    this.log('warn', message, data, context);
  }

  error(message: string, data?: any, context?: LogContext): void {
    this.log('error', message, data, context);
  }
}

// Create default logger instance
export const logger = new AppLogger(
  process.env.NODE_ENV === 'development' ? 'debug' : 'info'
);