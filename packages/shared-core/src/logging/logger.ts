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

  private serializeData(data: any): any {
    if (!data) return undefined;
    
    // Handle Error objects specially
    if (data instanceof Error) {
      return {
        name: data.name,
        message: data.message,
        stack: data.stack,
        ...Object.getOwnPropertyNames(data).reduce((acc, key) => {
          acc[key] = (data as any)[key];
          return acc;
        }, {} as any)
      };
    }
    
    // Handle plain objects - recursively serialize nested objects
    if (typeof data === 'object' && data !== null) {
      const result: any = {};
      let hasContent = false;
      
      for (const [key, value] of Object.entries(data)) {
        const serializedValue = this.serializeData(value);
        if (serializedValue !== undefined) {
          result[key] = serializedValue;
          hasContent = true;
        }
      }
      
      return hasContent ? result : undefined;
    }
    
    return data;
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
    const serializedData = this.serializeData(data);

    switch (level) {
      case 'debug':
        if (serializedData !== undefined) {
          console.debug(formattedMessage, serializedData);
        } else {
          console.debug(formattedMessage);
        }
        break;
      case 'info':
        if (serializedData !== undefined) {
          console.info(formattedMessage, serializedData);
        } else {
          console.info(formattedMessage);
        }
        break;
      case 'warn':
        if (serializedData !== undefined) {
          console.warn(formattedMessage, serializedData);
        } else {
          console.warn(formattedMessage);
        }
        break;
      case 'error':
        if (serializedData !== undefined) {
          console.error(formattedMessage, serializedData);
        } else {
          console.error(formattedMessage);
        }
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