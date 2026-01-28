/**
 * Security Utilities
 * Consolidated security, permission validation, and encryption utilities
 */

// ============================================================================
// Types
// ============================================================================

export interface SecurityConfig {
  enableCSP: boolean;
  enableXFrameOptions: boolean;
  enableHSTS: boolean;
  rateLimitEnabled: boolean;
  maxRequestsPerMinute: number;
}


// ============================================================================
// Exports
// ============================================================================


export interface SecurityHeaders {
  'Content-Security-Policy'?: string;
  'X-Frame-Options'?: string;
  'X-Content-Type-Options'?: string;
  'Referrer-Policy'?: string;
  'Permissions-Policy'?: string;
  'Strict-Transport-Security'?: string;
}

// ============================================================================
// Security Configuration
// ============================================================================

const DEFAULT_SECURITY_CONFIG: SecurityConfig = {
  enableCSP: true,
  enableXFrameOptions: true,
  enableHSTS: true,
  rateLimitEnabled: true,
  maxRequestsPerMinute: 100
};

export function getSecurityConfig(): SecurityConfig {
  return {
    ...DEFAULT_SECURITY_CONFIG,
    // Override with environment-specific settings
    rateLimitEnabled: process.env.NODE_ENV === 'production',
    maxRequestsPerMinute: process.env.NODE_ENV === 'production' ? 60 : 1000
  };
}

// ============================================================================
// Security Headers
// ============================================================================

export function generateSecurityHeaders(config: SecurityConfig = getSecurityConfig()): SecurityHeaders {
  const headers: SecurityHeaders = {
    'X-Content-Type-Options': 'nosniff',
    'Referrer-Policy': 'strict-origin-when-cross-origin',
    'Permissions-Policy': 'camera=(), microphone=(), geolocation=(), payment=()'
  };

  if (config.enableCSP) {
    headers['Content-Security-Policy'] = [
      "default-src 'self'",
      "script-src 'self' 'unsafe-eval' 'unsafe-inline' https://www.google-analytics.com https://www.googletagmanager.com",
      "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
      "font-src 'self' https://fonts.gstatic.com",
      "img-src 'self' data: blob: https:",
      "connect-src 'self' https://api.epsx.io https://backend-307278481624.us-central1.run.app",
      "frame-ancestors 'none'",
      "base-uri 'self'",
      "object-src 'none'"
    ].join('; ');
  }

  if (config.enableXFrameOptions) {
    headers['X-Frame-Options'] = 'DENY';
  }

  if (config.enableHSTS && process.env.NODE_ENV === 'production') {
    headers['Strict-Transport-Security'] = 'max-age=31536000; includeSubDomains; preload';
  }

  return headers;
}

// ============================================================================
// Input Sanitization
// ============================================================================

export function sanitizeInput(input: string): string {
  if (typeof input !== 'string') {
    return '';
  }

  return input
    .replace(/[<>]/g, '') // Remove potential HTML tags
    .replace(/javascript:/gi, '') // Remove javascript: protocols
    .replace(/data:/gi, '') // Remove data: protocols
    .replace(/vbscript:/gi, '') // Remove vbscript: protocols
    .trim();
}

export function sanitizeUrl(url: string): string {
  if (typeof url !== 'string') {
    return '';
  }

  // Allow only HTTP, HTTPS, and relative URLs
  if (url.startsWith('/') || url.startsWith('./') || url.startsWith('../')) {
    return url;
  }

  if (url.startsWith('http://') || url.startsWith('https://')) {
    return url;
  }

  return '';
}

export function sanitizeHtml(html: string): string {
  if (typeof html !== 'string') {
    return '';
  }

  return html
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '') // Remove script tags
    .replace(/<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi, '') // Remove iframe tags
    .replace(/on\w+="[^"]*"/gi, '') // Remove event handlers
    .replace(/on\w+='[^']*'/gi, '') // Remove event handlers
    .replace(/javascript:/gi, '') // Remove javascript: protocols
    .replace(/data:/gi, ''); // Remove data: protocols
}

// ============================================================================
// Rate Limiting
// ============================================================================

export class RateLimiter {
  private requests: Map<string, number[]> = new Map();
  private maxRequests: number;
  private windowMs: number;

  constructor(maxRequests: number = 100, windowMs: number = 60000) {
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
  }

  isAllowed(identifier: string): boolean {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    if (!this.requests.has(identifier)) {
      this.requests.set(identifier, []);
    }

    const userRequests = this.requests.get(identifier)!;

    // Remove expired requests
    const validRequests = userRequests.filter(timestamp => timestamp > windowStart);
    this.requests.set(identifier, validRequests);

    // Check if limit exceeded
    if (validRequests.length >= this.maxRequests) {
      return false;
    }

    // Add current request
    validRequests.push(now);
    return true;
  }

  getRemainingRequests(identifier: string): number {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    if (!this.requests.has(identifier)) {
      return this.maxRequests;
    }

    const userRequests = this.requests.get(identifier)!;
    const validRequests = userRequests.filter(timestamp => timestamp > windowStart);

    return Math.max(0, this.maxRequests - validRequests.length);
  }

  reset(identifier?: string): void {
    if (identifier) {
      this.requests.delete(identifier);
    } else {
      this.requests.clear();
    }
  }
}

// ============================================================================
// CSRF Protection
// ============================================================================

export function generateCSRFToken(): string {
  if (typeof window !== 'undefined' && window.crypto) {
    const array = new Uint8Array(32);
    window.crypto.getRandomValues(array);
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
  }

  // Fallback for server-side or unsupported browsers
  return Math.random().toString(36).substring(2) + Math.random().toString(36).substring(2);
}

export function validateCSRFToken(token: string, expectedToken: string): boolean {
  if (!token || !expectedToken) {
    return false;
  }

  return token === expectedToken;
}

// ============================================================================
// Content Security Policy
// ============================================================================

export function generateNonce(): string {
  if (typeof window !== 'undefined' && window.crypto) {
    const array = new Uint8Array(16);
    window.crypto.getRandomValues(array);
    return btoa(String.fromCharCode(...array));
  }

  return btoa(Math.random().toString(36).substring(2));
}

export function buildCSPHeader(nonce?: string): string {
  const directives = [
    "default-src 'self'",
    nonce ? `script-src 'self' 'nonce-${nonce}'` : "script-src 'self' 'unsafe-inline'",
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com",
    "img-src 'self' data: blob: https:",
    "connect-src 'self' https://api.epsx.io https://backend-307278481624.us-central1.run.app",
    "frame-ancestors 'none'",
    "base-uri 'self'",
    "object-src 'none'"
  ];

  return directives.join('; ');
}

// ============================================================================
// Exports
// ============================================================================

export const rateLimiter = new RateLimiter(getSecurityConfig().maxRequestsPerMinute);

// Level utilities for user tiers
export function calculateLevel(points: number): number {
  if (points < 100) return 1;
  if (points < 500) return 2;
  if (points < 1000) return 3;
  if (points < 2500) return 4;
  if (points < 5000) return 5;
  return Math.min(10, Math.floor(points / 1000) + 5);
}

export function getPointsForNextLevel(currentLevel: number): number {
  const levelThresholds = [0, 100, 500, 1000, 2500, 5000];

  if (currentLevel < levelThresholds.length) {
    return levelThresholds[currentLevel];
  }

  return (currentLevel - 4) * 1000;
}

export function getLevelProgress(points: number): { level: number; progress: number; nextLevelPoints: number } {
  const level = calculateLevel(points);
  const nextLevelPoints = getPointsForNextLevel(level);
  const previousLevelPoints = level > 1 ? getPointsForNextLevel(level - 1) : 0;

  const progress = ((points - previousLevelPoints) / (nextLevelPoints - previousLevelPoints)) * 100;

  return {
    level,
    progress: Math.min(100, Math.max(0, progress)),
    nextLevelPoints
  };
}