// Production logging utility for admin frontend

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: string;
  context?: Record<string, any>;
  adminId?: string;
  component?: string;
}

class AdminLogger {
  private isDevelopment = process.env.NODE_ENV === 'development';
  private logLevel: LogLevel = (process.env.LOG_LEVEL as LogLevel) || 'info';

  private getLogLevelPriority(level: LogLevel): number {
    const priorities = { debug: 0, info: 1, warn: 2, error: 3 };
    return priorities[level];
  }

  private shouldLog(level: LogLevel): boolean {
    return this.getLogLevelPriority(level) >= this.getLogLevelPriority(this.logLevel);
  }

  private createLogEntry(level: LogLevel, message: string, context?: Record<string, any>, component?: string): LogEntry {
    return {
      level,
      message,
      timestamp: new Date().toISOString(),
      context,
      adminId: this.getCurrentAdminId(),
      component,
    };
  }

  private getCurrentAdminId(): string | undefined {
    try {
      if (typeof window !== 'undefined') {
        const adminSession = localStorage.getItem('admin_session');
        if (adminSession) {
          const session = JSON.parse(adminSession);
          return session.admin_id;
        }
      }
    } catch (error) {
      // Ignore errors
    }
    return undefined;
  }

  private formatLogMessage(entry: LogEntry): string {
    const { level, message, timestamp, context, adminId, component } = entry;
    const contextStr = context ? ` | Context: ${JSON.stringify(context)}` : '';
    const adminStr = adminId ? ` | Admin: ${adminId}` : '';
    const componentStr = component ? ` | Component: ${component}` : '';
    return `[${timestamp}] ${level.toUpperCase()}: ${message}${adminStr}${componentStr}${contextStr}`;
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
    if (!this.isDevelopment && entry.level === 'error') {
      try {
        await fetch('/api/v1/admin/logs', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(entry),
        });
      } catch (error) {
        console.error('Failed to send admin log to remote service:', error);
      }
    }
  }

  private log(level: LogLevel, message: string, context?: Record<string, any>, component?: string): void {
    if (!this.shouldLog(level)) return;

    const entry = this.createLogEntry(level, message, context, component);

    if (this.isDevelopment) {
      this.writeToConsole(entry);
    }

    if (!this.isDevelopment) {
      this.sendToRemoteLogger(entry).catch(() => {
        this.writeToConsole(entry);
      });
    }
  }

  debug(message: string, context?: Record<string, any>, component?: string): void {
    this.log('debug', message, context, component);
  }

  info(message: string, context?: Record<string, any>, component?: string): void {
    this.log('info', message, context, component);
  }

  warn(message: string, context?: Record<string, any>, component?: string): void {
    this.log('warn', message, context, component);
  }

  error(message: string, context?: Record<string, any>, component?: string): void {
    this.log('error', message, context, component);
  }

  // Admin-specific logging methods
  adminAction(action: string, targetType: string, targetId?: string, details?: Record<string, any>): void {
    this.info(`Admin action: ${action}`, { targetType, targetId, ...details });
  }

  permissionChange(action: 'grant' | 'revoke', permission: string, targetUser: string, details?: Record<string, any>): void {
    this.warn(`Permission ${action}: ${permission} for user ${targetUser}`, details);
  }

  securityEvent(event: string, severity: 'low' | 'medium' | 'high', details?: Record<string, any>): void {
    const level = severity === 'high' ? 'error' : severity === 'medium' ? 'warn' : 'info';
    this.log(level, `Security event: ${event}`, { severity, ...details });
  }

  dataAccess(resource: string, action: string, success: boolean, details?: Record<string, any>): void {
    const level = success ? 'info' : 'error';
    this.log(level, `Data access: ${action} ${resource}`, { success, ...details });
  }
}

export const adminLogger = new AdminLogger();