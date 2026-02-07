/**
 * XSS Protection and Input Sanitization Utilities
 * Comprehensive protection against Cross-Site Scripting attacks
 */

// eslint-disable-next-line no-control-regex
const DANGEROUS_CHARS = /[<>'"&`\x00-\x1f]/g;
const HTML_ENTITIES: Record<string, string> = {
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#x27;',
  '&': '&amp;',
  '`': '&#x60;'
};

/**
 * HTML encode dangerous characters
 */
export function htmlEncode(input: string): string {
  if (typeof input !== 'string') {return '';}
  
  return input.replace(DANGEROUS_CHARS, (char) => {
    return HTML_ENTITIES[char] || char.charCodeAt(0) < 32 ? '' : char;
  });
}

/**
 * Strict email validation with XSS protection
 */
export function sanitizeEmail(email: string): string {
  if (typeof email !== 'string') {return '';}
  
  // Remove all non-email characters
  const cleaned = email
    .toLowerCase()
    .replace(/[^a-z0-9@._-]/g, '')
    .substring(0, 254); // RFC limit
  
  // Basic email format validation
  const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
  return emailRegex.test(cleaned) ? cleaned : '';
}

/**
 * Sanitize display name with length limits
 */
export function sanitizeDisplayName(name: string): string {
  if (typeof name !== 'string') {return '';}

  return name
    // eslint-disable-next-line no-control-regex
    .replace(/[<>'"&`\x00-\x1f]/g, '') // Remove dangerous chars
    .replace(/\s+/g, ' ') // Normalize whitespace
    .trim()
    .substring(0, 100); // Reasonable length limit
}

/**
 * Sanitize URL parameters to prevent XSS in redirects
 */
export function sanitizeRedirectUrl(url: string, allowedOrigins: string[] = []): string {
  if (typeof url !== 'string') {return '/';}
  
  try {
    const parsedUrl = new URL(url, window.location.origin);
    
    // Only allow same-origin or explicitly allowed origins
    const isAllowedOrigin = parsedUrl.origin === window.location.origin || 
                           allowedOrigins.includes(parsedUrl.origin);
    
    if (!isAllowedOrigin) {
      return '/';
    }
    
    // Remove potentially dangerous query parameters
    const dangerousParams = ['javascript:', 'data:', 'vbscript:', 'onload', 'onerror'];
    const cleanUrl = url.toLowerCase();
    
    for (const dangerous of dangerousParams) {
      if (cleanUrl.includes(dangerous)) {
        return '/';
      }
    }
    
    return parsedUrl.pathname + parsedUrl.search;
  } catch {
    return '/';
  }
}

/**
 * Validate and sanitize form data
 */
export interface SanitizedFormData {
  email: string;
  displayName?: string;
  redirectTo?: string;
  isValid: boolean;
  errors: string[];
}

export function sanitizeAuthFormData(formData: {
  email?: string;
  display_name?: string;
  redirectTo?: string;
}): SanitizedFormData {
  const errors: string[] = [];
  
  // Sanitize email
  const email = sanitizeEmail(formData.email || '');
  if (!email) {
    errors.push('Invalid email format');
  }
  
  // Sanitize display name (optional)
  const displayName = formData.display_name ? 
    sanitizeDisplayName(formData.display_name) : undefined;
  
  // Sanitize redirect URL
  const redirectTo = formData.redirectTo ? 
    sanitizeRedirectUrl(formData.redirectTo) : undefined;
  
  return {
    email,
    displayName,
    redirectTo,
    isValid: errors.length === 0,
    errors
  };
}

/**
 * Content Security Policy headers for auth pages
 */
export function getAuthCSPHeaders(): Record<string, string> {
  const nonce = generateNonce();
  
  return {
    'Content-Security-Policy': [
      "default-src 'self'",
      `script-src 'self' 'nonce-${nonce}' 'strict-dynamic'`,
      `style-src 'self' 'unsafe-inline'`, // Tailwind requires inline styles
      "img-src 'self' data: https:",
      "font-src 'self' https:",
      "connect-src 'self' https:",
      "frame-ancestors 'none'",
      "form-action 'self'",
      "base-uri 'self'"
    ].join('; '),
    'X-Content-Type-Options': 'nosniff',
    'X-Frame-Options': 'DENY',
    'X-XSS-Protection': '1; mode=block',
    'Referrer-Policy': 'strict-origin-when-cross-origin'
  };
}

/**
 * Generate secure nonce for CSP
 */
function generateNonce(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array));
}

/**
 * Input validation for passwords
 */
export function validatePassword(password: string): { isValid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (typeof password !== 'string') {
    errors.push('Password must be a string');
    return { isValid: false, errors };
  }
  
  if (password.length < 8) {
    errors.push('Password must be at least 8 characters long');
  }
  
  if (password.length > 128) {
    errors.push('Password must be less than 128 characters');
  }
  
  if (!/[a-z]/.test(password)) {
    errors.push('Password must contain at least one lowercase letter');
  }
  
  if (!/[A-Z]/.test(password)) {
    errors.push('Password must contain at least one uppercase letter');
  }
  
  if (!/[0-9]/.test(password)) {
    errors.push('Password must contain at least one number');
  }
  
  if (!/[!@#$%^&*()_+\-=[\]{}|;':",./<>?]/.test(password)) {
    errors.push('Password must contain at least one special character');
  }
  
  // Check for common patterns
  const commonPatterns = [
    /(.)\1{3,}/, // Repeated characters (4 or more)
    /123456/,    // Sequential numbers
    /qwerty/i,   // Common keyboard patterns
    /password/i, // Common words
    /admin/i,
    /user/i
  ];
  
  for (const pattern of commonPatterns) {
    if (pattern.test(password)) {
      errors.push('Password contains common patterns and is not secure');
      break;
    }
  }
  
  return {
    isValid: errors.length === 0,
    errors
  };
}