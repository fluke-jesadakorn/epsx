// Security utility functions - not server actions

import { headers, cookies } from 'next/headers';
import { z } from 'zod';

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
    process.env.NEXT_PUBLIC_APP_URL
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
    console.error('Session validation error:', error);
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
  
  if (password.length >= 12) score++;
  else feedback.push('Consider using 12 or more characters for better security');
  
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
    score = Math.max(0, score - 2);
    feedback.push('Avoid common patterns and words');
  }
  
  return {
    isValid: score >= 4,
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
}

export async function logSecurityEvent(event: Omit<AuditLog, 'ip' | 'userAgent' | 'timestamp'>): Promise<void> {
  const headersList = await headers();
  
  const auditLog: AuditLog = {
    ...event,
    ip: await getClientIP(),
    userAgent: headersList.get('user-agent') || 'unknown',
    timestamp: new Date()
  };
  
  // In production, send to logging service
  console.log('Security Event:', JSON.stringify(auditLog, null, 2));
  
  // Store in database or send to monitoring service
  // await storeAuditLog(auditLog);
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
    console.error('Security check error:', error);
    return { success: false, error: 'Security validation failed' };
  }
}