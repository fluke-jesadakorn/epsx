/**
 * Admin frontend logger — wraps shared logger with domain methods
 */
import { logger as sharedLogger } from '@/shared/utils/logger'

type LogContext = Record<string, unknown>

interface FwdOpts {
  level: 'debug' | 'info' | 'warn' | 'error'
  tag: string
  message: string
}

function isContext(arg: unknown): arg is LogContext {
  return typeof arg === 'object' && arg !== null && !Array.isArray(arg)
}

function fwd(opts: FwdOpts, args: unknown[]): void {
  const prefix = opts.tag ? `[${opts.tag}] ${opts.message}` : opts.message
  if (args.length === 1 && isContext(args[0])) {
    sharedLogger[opts.level](prefix, args[0])
  } else if (args.length > 0) {
    sharedLogger[opts.level](prefix, ...args)
  } else {
    sharedLogger[opts.level](prefix)
  }
}

class AdminLogger {
  debug(message: string, ...args: unknown[]): void { fwd({ level: 'debug', tag: '', message }, args) }
  info(message: string, ...args: unknown[]): void { fwd({ level: 'info', tag: '', message }, args) }
  warn(message: string, ...args: unknown[]): void { fwd({ level: 'warn', tag: '', message }, args) }
  error(message: string, ...args: unknown[]): void { fwd({ level: 'error', tag: '', message }, args) }

  auth = {
    login: (message: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'AUTH', message }, ctx ? [{ ...ctx, component: 'auth' }] : [{ component: 'auth' }]),
    logout: (message: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'AUTH', message }, ctx ? [{ ...ctx, component: 'auth' }] : [{ component: 'auth' }]),
    error: (message: string, ctx?: LogContext) => fwd({ level: 'error', tag: 'AUTH', message }, ctx ? [{ ...ctx, component: 'auth' }] : [{ component: 'auth' }]),
    success: (message: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'AUTH', message }, ctx ? [{ ...ctx, component: 'auth' }] : [{ component: 'auth' }]),
  }

  action = {
    start: (action: string, ctx?: LogContext) => fwd({ level: 'debug', tag: 'ACTION', message: `Starting ${action}` }, ctx ? [{ ...ctx, action }] : [{ action }]),
    success: (action: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'ACTION', message: `${action} completed` }, ctx ? [{ ...ctx, action }] : [{ action }]),
    error: (action: string, error: unknown, ctx?: LogContext) =>
      fwd({ level: 'error', tag: 'ACTION', message: `${action} failed` }, [{
        ...ctx,
        action,
        error: error instanceof Error ? error.message : String(error),
      }]),
  }

  admin = {
    userOperation: (op: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'ADMIN', message: `User ${op}` }, ctx ? [{ ...ctx, component: 'admin-users' }] : [{ component: 'admin-users' }]),
    permission: (op: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'ADMIN', message: `Permission ${op}` }, ctx ? [{ ...ctx, component: 'admin-permissions' }] : [{ component: 'admin-permissions' }]),
    audit: (op: string, ctx?: LogContext) => fwd({ level: 'info', tag: 'AUDIT', message: op }, ctx ? [{ ...ctx, component: 'audit' }] : [{ component: 'audit' }]),
  }
}

export const logger = new AdminLogger()
export type { LogContext }
