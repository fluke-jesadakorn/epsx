// Security utility functions - not server actions

import { headers, cookies } from 'next/headers';
import { z } from 'zod';
import { trackUserAction, trackError, trackSecurityEvent } from './firebase-analytics';
import { logger } from './logger';

// Security event constants and enums
export const SIGNIFICANT_EVENTS = [
  'LOGIN_SUCCESS',
  'LOGIN_FAILED',
  'REGISTER_SUCCESS',
  'REGISTER_FAILED',
  'LOGOUT',
  'PASSWORD_RESET_REQUEST',
  'PASSWORD_RESET_SUCCESS',
  'AUTHENTICATION_FAILED',
  'AUTHORIZATION_FAILED',
  'RATE_LIMIT_EXCEEDED',
  'INVALID_ORIGIN',
  'INVALID_USER_AGENT',
  'SESSION_EXPIRED',
  'PRIVILEGE_ESCALATION_ATTEMPT',
  'ACCOUNT_LOCKED',
  'SUSPICIOUS_ACTIVITY'
] as const;

export enum SecurityEventCategory {
  AUTHENTICATION = 'authentication',
  AUTHORIZATION = 'authorization',
  SESSION_MANAGEMENT = 'session_management',
  DATA_ACCESS = 'data_access',
  SYSTEM_SECURITY = 'system_security',
  USER_MANAGEMENT = 'user_management'
}

export enum EventSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical'
}

// Security configuration
const SECURITY_CONFIG = {
  maxRequestsPerMinute: 60,
  maxLoginAttemptsPerHour: 5,
  sessionTimeoutMs: 24 * 60 * 60 * 1000, // 24 hours
  csrfTokenLength: 32,
  allowedOrigins: [
    'http://localhost:3000',
    'https://epsx.com',
    'https://www.epsx.com',
    process.env.APP_URL
  ].filter(Boolean),
  secureCookieOptions: {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict' as const,
    maxAge: 24 * 60 * 60, // 24 hours
    path: '/'
  }
} as const;

// Rate limiting store (in production, use Redis)
const rateLimitStore = new Map<string, { count: number; resetTime: number }>();
const loginAttemptStore = new Map<string, { attempts: number; resetTime: number }>();

// Input sanitization
export function sanitizeInput(input: string): string {
  if (typeof input !== 'string') return '';
  
  return input
    .trim()
    .replace(/[<>'"&]/g, (match) => {
      const entityMap: Record<string, string> = {
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#x27;',
        '&': '&amp;'
      };
      return entityMap[match] || match;
    })
    .slice(0, 1000); // Limit length
}

// SQL injection prevention (for raw queries)
export function sanitizeSqlInput(input: string): string {
  if (typeof input !== 'string') return '';
  
  return input
    .replace(/['";\\]/g, '') // Remove dangerous characters
    .replace(/--/g, '') // Remove SQL comments
    .replace(/\/\*/g, '') // Remove SQL block comments
    .replace(/\*\//g, '')
    .trim()
    .slice(0, 255);
}

// XSS prevention
export function sanitizeHtml(input: string): string {
  if (typeof input !== 'string') return '';
  
  return input
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');
}

// CSRF protection
export async function generateCsrfToken(): Promise<string> {
  const bytes = new Uint8Array(SECURITY_CONFIG.csrfTokenLength);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

export async function validateCsrfToken(token: string): Promise<boolean> {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('csrf-token');
  
  if (!sessionToken || !token) return false;
  
  // Constant-time comparison to prevent timing attacks
  const encoder = new TextEncoder();
  const sessionBytes = encoder.encode(sessionToken.value);
  const tokenBytes = encoder.encode(token);
  
  if (sessionBytes.length !== tokenBytes.length) return false;
  
  // Manual constant-time comparison
  let result = 0;
  for (let i = 0; i < sessionBytes.length; i++) {
    result |= sessionBytes[i] ^ tokenBytes[i];
  }
  
  return result === 0;
}

// Rate limiting
export function checkRateLimit(identifier: string, limit: number = SECURITY_CONFIG.maxRequestsPerMinute): boolean {
  const now = Date.now();
  const windowMs = 60 * 1000; // 1 minute window
  
  const record = rateLimitStore.get(identifier);
  
  if (!record || now > record.resetTime) {
    rateLimitStore.set(identifier, { count: 1, resetTime: now + windowMs });
    return true;
  }
  
  if (record.count >= limit) {
    return false;
  }
  
  record.count++;
  return true;
}

// Login attempt tracking
export function checkLoginAttempts(identifier: string): boolean {
  const now = Date.now();
  const windowMs = 60 * 60 * 1000; // 1 hour window
  
  const record = loginAttemptStore.get(identifier);
  
  if (!record || now > record.resetTime) {
    loginAttemptStore.set(identifier, { attempts: 1, resetTime: now + windowMs });
    return true;
  }
  
  if (record.attempts >= SECURITY_CONFIG.maxLoginAttemptsPerHour) {
    return false;
  }
  
  record.attempts++;
  return true;
}

// Reset login attempts on successful login
export function resetLoginAttempts(identifier: string): void {
  loginAttemptStore.delete(identifier);
}

// Secure cookie management
export async function setSecureCookie(name: string, value: string, options: Partial<typeof SECURITY_CONFIG.secureCookieOptions> = {}): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.set(name, value, {
    ...SECURITY_CONFIG.secureCookieOptions,
    ...options
  });
}

export async function getSecureCookie(name: string): Promise<string | undefined> {
  const cookieStore = await cookies();
  const cookie = cookieStore.get(name);
  return cookie?.value;
}

export async function deleteSecureCookie(name: string): Promise<void> {
  const cookieStore = await cookies();
  cookieStore.delete(name);
}

// IP address extraction
export async function getClientIP(): Promise<string> {
  const headersList = await headers();
  
  // Check various headers for IP (in order of preference)
  const ipHeaders = [
    'x-forwarded-for',
    'x-real-ip',
    'x-client-ip',
    'cf-connecting-ip', // Cloudflare
    'x-forwarded',
    'forwarded-for',
    'forwarded'
  ];
  
  for (const header of ipHeaders) {
    const ip = headersList.get(header);
    if (ip) {
      // x-forwarded-for can contain multiple IPs, take the first one
      return ip.split(',')[0].trim();
    }
  }
  
  return '127.0.0.1'; // Fallback
}

// Origin validation
export async function validateOrigin(): Promise<boolean> {
  const headersList = await headers();
  const origin = headersList.get('origin');
  const referer = headersList.get('referer');
  
  if (!origin && !referer) {
    // Allow requests without origin/referer for same-origin requests
    return true;
  }
  
  const requestOrigin = origin || (referer ? new URL(referer).origin : null);
  
  if (!requestOrigin) return false;
  
  return SECURITY_CONFIG.allowedOrigins.includes(requestOrigin);
}

// User agent validation (basic bot detection)
export async function validateUserAgent(): Promise<boolean> {
  const headersList = await headers();
  const userAgent = headersList.get('user-agent');
  
  if (!userAgent) return false;
  
  // Block common bot patterns
  const botPatterns = [
    /bot/i,
    /crawler/i,
    /spider/i,
    /scraper/i,
    /curl/i,
    /wget/i,
    /python/i,
    /php/i
  ];
  
  return !botPatterns.some(pattern => pattern.test(userAgent));
}

// Session validation
export async function validateSession(sessionToken: string): Promise<boolean> {
  if (!sessionToken) return false;
  
  try {
    // Validate session token format
    const tokenPattern = /^[A-Za-z0-9+/=]+$/;
    if (!tokenPattern.test(sessionToken)) return false;
    
    // Additional session validation would go here
    // (e.g., check against database, verify JWT signature, etc.)
    
    return true;
  } catch (error) {
    logger.error('Session validation error', { error: error instanceof Error ? error.message : error });
    return false;
  }
}

// Password strength validation
export function validatePasswordStrength(password: string): { 
  isValid: boolean; 
  score: number; 
  feedback: string[] 
} {
  const feedback: string[] = [];
  let score = 0;
  
  if (password.length >= 8) score++;
  else feedback.push('Password must be at least 8 characters long');
  
  // Removed 12+ character requirement
  
  if (/[a-z]/.test(password)) score++;
  else feedback.push('Add lowercase letters');
  
  if (/[A-Z]/.test(password)) score++;
  else feedback.push('Add uppercase letters');
  
  if (/[0-9]/.test(password)) score++;
  else feedback.push('Add numbers');
  
  if (/[^A-Za-z0-9]/.test(password)) score++;
  else feedback.push('Add special characters');
  
  // Check for common patterns
  const commonPatterns = [
    /(.)\1{2,}/, // Repeated characters
    /123|abc|qwe/i, // Sequential patterns
    /password|admin|user/i // Common words
  ];
  
  if (commonPatterns.some(pattern => pattern.test(password))) {
    score = Math.max(0, score - 1);
    feedback.push('Avoid common patterns and words');
  }
  
  return {
    isValid: score >= 3,
    score,
    feedback
  };
}

// Security headers
export function getSecurityHeaders(): Record<string, string> {
  return {
    'X-Content-Type-Options': 'nosniff',
    'X-Frame-Options': 'DENY',
    'X-XSS-Protection': '1; mode=block',
    'Referrer-Policy': 'strict-origin-when-cross-origin',
    'Content-Security-Policy': 
      "default-src 'self'; " +
      "script-src 'self' 'unsafe-inline' 'unsafe-eval'; " +
      "style-src 'self' 'unsafe-inline'; " +
      "img-src 'self' data: https:; " +
      "font-src 'self'; " +
      "connect-src 'self' https:; " +
      "frame-ancestors 'none';",
    'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
    'Permissions-Policy': 'camera=(), microphone=(), geolocation=()'
  };
}

// Audit logging
export interface AuditLog {
  userId?: string;
  action: string;
  resource: string;
  ip: string;
  userAgent: string;
  timestamp: Date;
  success: boolean;
  details?: Record<string, any>;
  category?: SecurityEventCategory;
  severity?: EventSeverity;
}

export interface SecurityEvent {
  action: string;
  resource: string;
  userId?: string;
  success: boolean;
  details?: Record<string, any>;
  category?: SecurityEventCategory;
  severity?: EventSeverity;
}

// Store audit log in PostgreSQL database
async function storeAuditLogInDatabase(auditLog: AuditLog): Promise<void> {
  try {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/v1/audit/logs`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: auditLog.userId || null,
        action: auditLog.action,
        resource_type: auditLog.resource,
        details: auditLog.details || {},
        ip_address: auditLog.ip,
        user_agent: auditLog.userAgent,
        event_category: auditLog.category || SecurityEventCategory.SYSTEM_SECURITY,
        severity: auditLog.severity || EventSeverity.MEDIUM,
        success: auditLog.success
      }),
    });

    if (!response.ok) {
      logger.error('Failed to store audit log in database', { status: response.status });
    }
  } catch (error) {
    logger.error('Error storing audit log in database', { error: error instanceof Error ? error.message : error });
  }
}

// Track security event in Firebase Analytics
function trackSecurityEventInAnalytics(auditLog: AuditLog): void {
  try {
    if (typeof window !== 'undefined') {
      trackSecurityEvent(
        auditLog.action,
        auditLog.resource,
        auditLog.success,
        auditLog.userId,
        auditLog.category,
        auditLog.severity,
        {
          ip_address: auditLog.ip,
          user_agent: auditLog.userAgent,
          timestamp: auditLog.timestamp.toISOString(),
          ...auditLog.details
        }
      );
    }
  } catch (error) {
    logger.error('Error tracking security event in analytics', { error: error instanceof Error ? error.message : error });
  }
}

// Determine event category and severity based on action
function categorizeSecurityEvent(action: string): { category: SecurityEventCategory; severity: EventSeverity } {
  const authActions = ['LOGIN_SUCCESS', 'LOGIN_FAILED', 'REGISTER_SUCCESS', 'REGISTER_FAILED', 'PASSWORD_RESET_REQUEST', 'PASSWORD_RESET_SUCCESS'];
  const criticalActions = ['PRIVILEGE_ESCALATION_ATTEMPT', 'ACCOUNT_LOCKED', 'SUSPICIOUS_ACTIVITY'];
  const highSeverityActions = ['AUTHENTICATION_FAILED', 'AUTHORIZATION_FAILED', 'RATE_LIMIT_EXCEEDED'];
  
  let category = SecurityEventCategory.SYSTEM_SECURITY;
  let severity = EventSeverity.MEDIUM;
  
  if (authActions.includes(action)) {
    category = SecurityEventCategory.AUTHENTICATION;
    severity = action.includes('FAILED') ? EventSeverity.HIGH : EventSeverity.MEDIUM;
  } else if (criticalActions.includes(action)) {
    severity = EventSeverity.CRITICAL;
  } else if (highSeverityActions.includes(action)) {
    severity = EventSeverity.HIGH;
  } else if (action.includes('SESSION')) {
    category = SecurityEventCategory.SESSION_MANAGEMENT;
  } else if (action.includes('ORIGIN') || action.includes('USER_AGENT')) {
    category = SecurityEventCategory.SYSTEM_SECURITY;
    severity = EventSeverity.HIGH;
  }
  
  return { category, severity };
}

export async function logSecurityEvent(event: Omit<AuditLog, 'ip' | 'userAgent' | 'timestamp'>): Promise<void> {
  const headersList = await headers();
  
  const { category, severity } = event.category && event.severity 
    ? { category: event.category, severity: event.severity }
    : categorizeSecurityEvent(event.action);
  
  const auditLog: AuditLog = {
    ...event,
    ip: await getClientIP(),
    userAgent: headersList.get('user-agent') || 'unknown',
    timestamp: new Date(),
    category,
    severity
  };
  
  // Log security event
  logger.info('Security Event', { 
    action: auditLog.action, 
    resource: auditLog.resource, 
    success: auditLog.success, 
    category: auditLog.category,
    severity: auditLog.severity,
    userId: auditLog.userId,
    ip: auditLog.ip
  });
  
  // Always track in Firebase Analytics
  trackSecurityEventInAnalytics(auditLog);
  
  // Store significant events in PostgreSQL database
  if (SIGNIFICANT_EVENTS.includes(auditLog.action as any)) {
    await storeAuditLogInDatabase(auditLog);
  }
}

// Unified logging function that routes events appropriately
export async function logEvent(event: SecurityEvent): Promise<void> {
  await logSecurityEvent(event);
}

// Analytics-only event tracking for non-significant events
export function trackAnalyticsEvent(
  action: string, 
  category: string, 
  label?: string, 
  userId?: string, 
  additionalData?: Record<string, any>
): void {
  try {
    if (typeof window !== 'undefined') {
      trackUserAction(action, category, label, undefined, userId);
      
      // Log additional analytics data if provided
      if (additionalData) {
        logger.debug('Analytics Event', { action, category, label, userId, ...additionalData });
      }
    }
  } catch (error) {
    logger.error('Error tracking analytics event', { error: error instanceof Error ? error.message : error });
  }
}

// Comprehensive security check for Server Actions
export async function performSecurityChecks(options: {
  requireAuth?: boolean;
  checkOrigin?: boolean;
  checkUserAgent?: boolean;
  rateLimitKey?: string;
}): Promise<{ success: boolean; error?: string }> {
  try {
    // Rate limiting
    if (options.rateLimitKey) {
      const clientIP = await getClientIP();
      const rateLimitKey = `${options.rateLimitKey}_${clientIP}`;
      
      if (!checkRateLimit(rateLimitKey)) {
        await logSecurityEvent({
          action: 'RATE_LIMIT_EXCEEDED',
          resource: options.rateLimitKey,
          success: false
        });
        return { success: false, error: 'Rate limit exceeded' };
      }
    }
    
    // Origin validation
    if (options.checkOrigin) {
      if (!await validateOrigin()) {
        await logSecurityEvent({
          action: 'INVALID_ORIGIN',
          resource: 'request',
          success: false
        });
        return { success: false, error: 'Invalid origin' };
      }
    }
    
    // User agent validation
    if (options.checkUserAgent) {
      if (!await validateUserAgent()) {
        await logSecurityEvent({
          action: 'INVALID_USER_AGENT',
          resource: 'request',
          success: false
        });
        return { success: false, error: 'Invalid user agent' };
      }
    }
    
    // Authentication check
    if (options.requireAuth) {
      const sessionToken = await getSecureCookie('__session');
      
      if (!sessionToken || !await validateSession(sessionToken)) {
        await logSecurityEvent({
          action: 'AUTHENTICATION_FAILED',
          resource: 'session',
          success: false
        });
        return { success: false, error: 'Authentication required' };
      }
    }
    
    return { success: true };
  } catch (error) {
    logger.error('Security check error', { error: error instanceof Error ? error.message : error });
    return { success: false, error: 'Security validation failed' };
  }
}