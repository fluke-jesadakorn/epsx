// Production logging utility

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: string;
  context?: Record<string, any>;
  userId?: string;
}

class Logger {
  private isDevelopment = process.env.NODE_ENV === 'development';
  private logLevel: LogLevel = (process.env.LOG_LEVEL as LogLevel) || 'info';

  private getLogLevelPriority(level: LogLevel): number {
    const priorities = { debug: 0, info: 1, warn: 2, error: 3 };
    return priorities[level];
  }

  private shouldLog(level: LogLevel): boolean {
    return this.getLogLevelPriority(level) >= this.getLogLevelPriority(this.logLevel);
  }

  private createLogEntry(level: LogLevel, message: string, context?: Record<string, any>): LogEntry {
    return {
      level,
      message,
      timestamp: new Date().toISOString(),
      context,
      userId: this.getCurrentUserId(),
    };
  }

  private getCurrentUserId(): string | undefined {
    // In production, get user ID from authentication context
    try {
      if (typeof window !== 'undefined') {
        // Client-side: get from auth context or local storage
        const userSession = localStorage.getItem('user_session');
        if (userSession) {
          const session = JSON.parse(userSession);
          return session.user_id;
        }
      }
    } catch (error) {
      // Ignore errors in getting user ID
    }
    return undefined;
  }

  private formatLogMessage(entry: LogEntry): string {
    const { level, message, timestamp, context, userId } = entry;
    const contextStr = context ? ` | Context: ${JSON.stringify(context)}` : '';
    const userStr = userId ? ` | User: ${userId}` : '';
    return `[${timestamp}] ${level.toUpperCase()}: ${message}${userStr}${contextStr}`;
  }

  private writeToConsole(entry: LogEntry): void {
    const formattedMessage = this.formatLogMessage(entry);
    
    switch (entry.level) {
      case 'debug':
        console.debug(formattedMessage);
        break;
      case 'info':
        console.info(formattedMessage);
        break;
      case 'warn':
        console.warn(formattedMessage);
        break;
      case 'error':
        console.error(formattedMessage);
        break;
    }
  }

  private async sendToRemoteLogger(entry: LogEntry): Promise<void> {
    // In production, send logs to external service like DataDog, CloudWatch, etc.
    if (!this.isDevelopment && entry.level === 'error') {
      try {
        const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
        await fetch(`${backendUrl}/api/v1/logs`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify(entry),
        });
      } catch (error) {
        // Fallback to console if remote logging fails
        console.error('Failed to send log to remote service:', error);
      }
    }
  }

  private log(level: LogLevel, message: string, context?: Record<string, any>): void {
    if (!this.shouldLog(level)) return;

    const entry = this.createLogEntry(level, message, context);

    // Always write to console in development
    if (this.isDevelopment) {
      this.writeToConsole(entry);
    }

    // Send to remote logger in production
    if (!this.isDevelopment) {
      this.sendToRemoteLogger(entry).catch(() => {
        // Fallback to console if remote logging fails
        this.writeToConsole(entry);
      });
    }
  }

  debug(message: string, context?: Record<string, any>): void {
    this.log('debug', message, context);
  }

  info(message: string, context?: Record<string, any>): void {
    this.log('info', message, context);
  }

  warn(message: string, context?: Record<string, any>): void {
    this.log('warn', message, context);
  }

  error(message: string, context?: Record<string, any>): void {
    this.log('error', message, context);
  }

  // Convenience methods for common use cases
  authEvent(action: string, success: boolean, details?: Record<string, any>): void {
    this.info(`Auth: ${action}`, { success, ...details });
  }

  apiCall(endpoint: string, method: string, status: number, duration?: number): void {
    const level = status >= 400 ? 'error' : 'info';
    this.log(level, `API: ${method} ${endpoint}`, { status, duration });
  }

  userAction(action: string, details?: Record<string, any>): void {
    this.info(`User action: ${action}`, details);
  }

  performanceMetric(metric: string, value: number, unit: string = 'ms'): void {
    this.debug(`Performance: ${metric} = ${value}${unit}`);
  }
}

export const logger = new Logger();