/**
 * Centralized logging utility for the frontend application
 * Provides consistent logging behavior across development and production
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerConfig {
  enabledInProduction: boolean;
  level: LogLevel;
  prefix?: string;
}

class Logger {
  private config: LoggerConfig;

  constructor(config: LoggerConfig = { 
    enabledInProduction: false, 
    level: 'info' 
  }) {
    this.config = config;
  }

  private shouldLog(level: LogLevel): boolean {
    if (typeof window === 'undefined') return false; // Server-side rendering
    
    const isDevelopment = process.env.NODE_ENV === 'development';
    
    // Always allow error and warn in production for debugging critical issues
    if (level === 'error' || level === 'warn') return true;
    
    // In production, only log if explicitly enabled
    if (!isDevelopment && !this.config.enabledInProduction) return false;
    
    // In development, respect log level
    const levels = ['debug', 'info', 'warn', 'error'];
    const currentLevelIndex = levels.indexOf(this.config.level);
    const requestedLevelIndex = levels.indexOf(level);
    
    return requestedLevelIndex >= currentLevelIndex;
  }

  private formatMessage(level: LogLevel, message: string, ...args: unknown[]): [string, ...unknown[]] {
    const prefix = this.config.prefix || 'EPSX';
    const timestamp = new Date().toISOString();
    const formattedMessage = `[${prefix}] ${timestamp} [${level.toUpperCase()}]: ${message}`;
    return [formattedMessage, ...args];
  }

  debug(message: string, ...args: unknown[]): void {
    if (this.shouldLog('debug')) {
      const [formattedMessage, ...formattedArgs] = this.formatMessage('debug', message, ...args);
      // Use native console.debug as this is the base logger implementation
      console.debug(formattedMessage, ...formattedArgs);
    }
  }

  info(message: string, ...args: unknown[]): void {
    if (this.shouldLog('info')) {
      const [formattedMessage, ...formattedArgs] = this.formatMessage('info', message, ...args);
      // Use native console.info as this is the base logger implementation
      console.info(formattedMessage, ...formattedArgs);
    }
  }

  warn(message: string, ...args: unknown[]): void {
    if (this.shouldLog('warn')) {
      const [formattedMessage, ...formattedArgs] = this.formatMessage('warn', message, ...args);
      // Use native console.warn as this is the base logger implementation
      console.warn(formattedMessage, ...formattedArgs);
    }
  }

  error(message: string, error?: Error | unknown, ...args: unknown[]): void {
    if (this.shouldLog('error')) {
      const [formattedMessage, ...formattedArgs] = this.formatMessage('error', message, error, ...args);
      // Use native console.error as this is the base logger implementation
      console.error(formattedMessage, ...formattedArgs);
    }
  }

  /**
   * Create a child logger with a specific prefix
   */
  child(prefix: string): Logger {
    return new Logger({
      ...this.config,
      prefix: this.config.prefix ? `${this.config.prefix}:${prefix}` : prefix
    });
  }
}

// Default logger instance
export const logger = new Logger({
  enabledInProduction: false,
  level: process.env.NODE_ENV === 'development' ? 'debug' : 'warn',
  prefix: 'EPSX'
});

// Named loggers for different modules
export const apiLogger = logger.child('API');
export const authLogger = logger.child('AUTH');
export const analyticsLogger = logger.child('ANALYTICS');
export const uiLogger = logger.child('UI');

// Export the Logger class for custom instances
export { Logger };

// Utility function to safely log in development only
export const devLog = (message: string, ...args: unknown[]): void => {
  if (process.env.NODE_ENV === 'development') {
    // Use native console.log as this is the base dev utility
    console.log(`[DEV]: ${message}`, ...args);
  }
};

// Utility function for conditional error logging
export const safeError = (message: string, error?: Error | unknown): void => {
  if (process.env.NODE_ENV === 'development') {
    // Use native console.error as this is the base safe error utility
    console.error(message, error);
  } else {
    // In production, you might want to send to an error tracking service
    // For now, we'll just log critical errors
    if (error instanceof Error && error.message.includes('network') || 
        error instanceof Error && error.message.includes('fetch')) {
      // Use native console.error as this is the base safe error utility for critical errors
      console.error(`[EPSX] Critical Error: ${message}`, error);
    }
  }
};