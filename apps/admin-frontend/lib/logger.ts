/**
 * Centralized logging service for admin frontend
 * Replaces console.log statements with structured logging
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LogContext {
  [key: string]: any
}

class Logger {
  private isDevelopment = process.env.NODE_ENV === 'development'
  private minLevel: LogLevel = this.isDevelopment ? 'debug' : 'info'

  private levels: Record<LogLevel, number> = {
    debug: 0,
    info: 1, 
    warn: 2,
    error: 3
  }

  private shouldLog(level: LogLevel): boolean {
    return this.levels[level] >= this.levels[this.minLevel]
  }

  private formatMessage(level: LogLevel, message: string, context?: LogContext): string {
    const timestamp = new Date().toISOString()
    const contextStr = context ? ` ${JSON.stringify(context)}` : ''
    return `[${timestamp}] [${level.toUpperCase()}] ${message}${contextStr}`
  }

  private log(level: LogLevel, message: string, context?: LogContext): void {
    if (!this.shouldLog(level)) return

    const formattedMessage = this.formatMessage(level, message, context)

    switch (level) {
      case 'debug':
        console.debug(formattedMessage)
        break
      case 'info':
        console.info(formattedMessage)
        break
      case 'warn':
        console.warn(formattedMessage)
        break
      case 'error':
        console.error(formattedMessage)
        break
    }
  }

  debug(message: string, context?: LogContext): void {
    this.log('debug', message, context)
  }

  info(message: string, context?: LogContext): void {
    this.log('info', message, context)
  }

  warn(message: string, context?: LogContext): void {
    this.log('warn', message, context)
  }

  error(message: string, context?: LogContext): void {
    this.log('error', message, context)
  }

  // Auth-specific logging helpers
  auth = {
    login: (message: string, context?: LogContext) => 
      this.info(`🔐 [AUTH] ${message}`, { ...context, component: 'auth' }),
    
    logout: (message: string, context?: LogContext) => 
      this.info(`🚪 [AUTH] ${message}`, { ...context, component: 'auth' }),
    
    error: (message: string, context?: LogContext) => 
      this.error(`🚨 [AUTH] ${message}`, { ...context, component: 'auth' }),
    
    success: (message: string, context?: LogContext) => 
      this.info(`✅ [AUTH] ${message}`, { ...context, component: 'auth' })
  }

  // Server action logging helpers
  action = {
    start: (action: string, context?: LogContext) => 
      this.debug(`▶️ [ACTION] Starting ${action}`, { ...context, action }),
    
    success: (action: string, context?: LogContext) => 
      this.info(`✅ [ACTION] ${action} completed successfully`, { ...context, action }),
    
    error: (action: string, error: unknown, context?: LogContext) => 
      this.error(`❌ [ACTION] ${action} failed`, { 
        ...context, 
        action,
        error: error instanceof Error ? error.message : String(error) 
      })
  }

  // Admin operations logging
  admin = {
    userOperation: (operation: string, context?: LogContext) => 
      this.info(`👤 [ADMIN] User ${operation}`, { ...context, component: 'admin-users' }),
    
    permission: (operation: string, context?: LogContext) => 
      this.info(`🔑 [ADMIN] Permission ${operation}`, { ...context, component: 'admin-permissions' }),
    
    audit: (operation: string, context?: LogContext) => 
      this.info(`📋 [AUDIT] ${operation}`, { ...context, component: 'audit' })
  }
}

// Export singleton instance
export const logger = new Logger()

// Export types for use in components
export type { LogLevel, LogContext }

// Legacy console replacement - helps with gradual migration
export const console_log = logger.debug
export const console_info = logger.info
export const console_warn = logger.warn  
export const console_error = logger.error