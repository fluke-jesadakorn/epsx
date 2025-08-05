import { NextRequest, NextResponse } from 'next/server';
import { getSecurityHeaders } from '../security';

// Security middleware for Next.js middleware
export function securityMiddleware(request: NextRequest): NextResponse | null {
  const response = NextResponse.next();
  
  // Apply security headers
  const securityHeaders = getSecurityHeaders();
  Object.entries(securityHeaders).forEach(([key, value]) => {
    response.headers.set(key, value);
  });
  
  // Check for suspicious patterns in URL
  const suspiciousPatterns = [
    /\.\./,  // Path traversal
    /<script/i,  // XSS attempts
    /union.*select/i,  // SQL injection
    /javascript:/i,  // JavaScript protocol
    /data:.*base64/i,  // Data URLs with base64
    /vbscript:/i,  // VBScript protocol
  ];
  
  const url = request.url.toLowerCase();
  if (suspiciousPatterns.some(pattern => pattern.test(url))) {
    console.warn('Suspicious URL detected', { url: request.url, ip: getClientIP(request) });
    return new NextResponse('Bad Request', { status: 400 });
  }
  
  // Block requests with suspicious User-Agent
  const userAgent = request.headers.get('user-agent') || '';
  const suspiciousUserAgents = [
    /sqlmap/i,
    /nikto/i,
    /nessus/i,
    /masscan/i,
    /nmap/i,
    /acunetix/i,
  ];
  
  if (suspiciousUserAgents.some(pattern => pattern.test(userAgent))) {
    console.warn('Suspicious User-Agent detected', { userAgent, ip: getClientIP(request) });
    return new NextResponse('Forbidden', { status: 403 });
  }
  
  // Rate limiting for login/register endpoints
  const sensitiveEndpoints = ['/login', '/register', '/api/v1/auth'];
  const isSensitiveEndpoint = sensitiveEndpoints.some(endpoint => 
    request.nextUrl.pathname.startsWith(endpoint)
  );
  
  if (isSensitiveEndpoint) {
    const ip = getClientIP(request);
    const rateLimitKey = `auth_${ip}`;
    
    // This would need to be implemented with a proper store (Redis, etc.)
    // Log authentication endpoint access for monitoring
    console.info('Auth endpoint access', {
      ip,
      path: request.nextUrl.pathname,
      userAgent: request.headers.get('user-agent')
    });
  }
  
  return response;
}

// Helper function to get client IP from request
function getClientIP(request: NextRequest): string {
  const forwarded = request.headers.get('x-forwarded-for');
  const realIP = request.headers.get('x-real-ip');
  const cfIP = request.headers.get('cf-connecting-ip');
  
  if (forwarded) {
    return forwarded.split(',')[0].trim();
  }
  
  if (realIP) return realIP;
  if (cfIP) return cfIP;
  
  return request.ip || '127.0.0.1';
}

// CSRF protection for forms
export function csrfMiddleware(request: NextRequest): NextResponse | null {
  // Only check CSRF for POST, PUT, DELETE, PATCH requests
  if (!['POST', 'PUT', 'DELETE', 'PATCH'].includes(request.method)) {
    return NextResponse.next();
  }
  
  // Skip CSRF for API routes (they should use other authentication)
  if (request.nextUrl.pathname.startsWith('/api/')) {
    return NextResponse.next();
  }
  
  const referer = request.headers.get('referer');
  const origin = request.headers.get('origin');
  
  if (!referer && !origin) {
    console.warn('CSRF: No referer or origin header', { path: request.nextUrl.pathname, ip: getClientIP(request) });
    return new NextResponse('Forbidden', { status: 403 });
  }
  
  const requestOrigin = origin || (referer ? new URL(referer).origin : null);
  const allowedOrigins = [
    process.env.APP_URL,
    'http://localhost:3000',
    'https://epsx.com',
    'https://www.epsx.com'
  ].filter(Boolean);
  
  if (!requestOrigin || !allowedOrigins.includes(requestOrigin)) {
    console.warn('CSRF: Invalid origin', { requestOrigin, path: request.nextUrl.pathname, ip: getClientIP(request) });
    return new NextResponse('Forbidden', { status: 403 });
  }
  
  return NextResponse.next();
}

// Content-Type validation
export function contentTypeMiddleware(request: NextRequest): NextResponse | null {
  // Only check content-type for POST/PUT requests
  if (!['POST', 'PUT', 'PATCH'].includes(request.method)) {
    return NextResponse.next();
  }
  
  const contentType = request.headers.get('content-type') || '';
  
  // Allow form data and JSON
  const allowedTypes = [
    'application/json',
    'application/x-www-form-urlencoded',
    'multipart/form-data',
    'text/plain'
  ];
  
  const isAllowed = allowedTypes.some(type => contentType.includes(type));
  
  if (!isAllowed && contentType) {
    console.warn('Invalid content-type', { contentType, path: request.nextUrl.pathname, ip: getClientIP(request) });
    return new NextResponse('Unsupported Media Type', { status: 415 });
  }
  
  return NextResponse.next();
}

// Request size validation
export function requestSizeMiddleware(request: NextRequest): NextResponse | null {
  const contentLength = request.headers.get('content-length');
  
  if (contentLength) {
    const size = parseInt(contentLength, 10);
    const maxSize = 10 * 1024 * 1024; // 10MB
    
    if (size > maxSize) {
      console.warn('Request too large', { size, maxSize, path: request.nextUrl.pathname, ip: getClientIP(request) });
      return new NextResponse('Payload Too Large', { status: 413 });
    }
  }
  
  return NextResponse.next();
}

// Combine all security middlewares
export function createSecurityMiddleware() {
  return (request: NextRequest): NextResponse => {
    // Apply security middlewares in order
    const middlewares = [
      securityMiddleware,
      csrfMiddleware,
      contentTypeMiddleware,
      requestSizeMiddleware
    ];
    
    for (const middleware of middlewares) {
      const result = middleware(request);
      if (result && result.status !== 200) {
        return result;
      }
    }
    
    return NextResponse.next();
  };
}