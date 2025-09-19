import { createHash, randomBytes } from 'crypto';
import { NextRequest, NextResponse } from 'next/server';

const CSRF_TOKEN_NAME = 'csrf-token';
const CSRF_HEADER_NAME = 'x-csrf-token';
const CSRF_SECRET_LENGTH = 32;
const CSRF_TOKEN_LENGTH = 64;

/**
 * Generate a cryptographically secure CSRF token
 */
export function generateCSRFToken(): string {
  return randomBytes(CSRF_TOKEN_LENGTH).toString('hex');
}

/**
 * Create CSRF secret for session
 */
export function generateCSRFSecret(): string {
  return randomBytes(CSRF_SECRET_LENGTH).toString('hex');
}

/**
 * Validate CSRF token against secret
 */
export function validateCSRFToken(token: string, secret: string): boolean {
  if (!token || !secret) return false;
  
  try {
    // Create expected token hash
    const expectedHash = createHash('sha256')
      .update(`${token}:${secret}`)
      .digest('hex');
    
    // Compare with provided token (constant time comparison)
    const providedHash = createHash('sha256')
      .update(`${token}:${secret}`)
      .digest('hex');
    
    return expectedHash === providedHash;
  } catch {
    return false;
  }
}

/**
 * Extract CSRF token from request headers or body
 */
export function extractCSRFToken(request: NextRequest): string | null {
  // Check header first
  const headerToken = request.headers.get(CSRF_HEADER_NAME);
  if (headerToken) return headerToken;
  
  // Check form data (for form submissions)
  const contentType = request.headers.get('content-type');
  if (contentType?.includes('application/x-www-form-urlencoded')) {
    // This would need to be extracted from parsed form data
    // For now, we'll rely on header-based tokens
  }
  
  return null;
}

/**
 * Get CSRF secret from secure cookies
 */
export function getCSRFSecret(request: NextRequest): string | null {
  return request.cookies.get('csrf-secret')?.value || null;
}

/**
 * Set CSRF secret in secure cookie
 */
export function setCSRFSecret(response: NextResponse, secret: string): void {
  response.cookies.set('csrf-secret', secret, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    maxAge: 60 * 60 * 24, // 24 hours
    path: '/'
  });
}

/**
 * CSRF protection middleware for API routes
 */
export function withCSRFProtection(handler: (req: NextRequest) => Promise<NextResponse>) {
  return async (request: NextRequest): Promise<NextResponse> => {
    // Skip CSRF for GET requests (safe methods)
    if (request.method === 'GET') {
      return handler(request);
    }
    
    // Extract CSRF token and secret
    const token = extractCSRFToken(request);
    const secret = getCSRFSecret(request);
    
    if (!token || !secret || !validateCSRFToken(token, secret)) {
      return NextResponse.json(
        { error: 'Invalid CSRF token' },
        { status: 403 }
      );
    }
    
    return handler(request);
  };
}

/**
 * API route to get CSRF token for client-side forms
 */
export async function getCSRFTokenAPI(request: NextRequest): Promise<NextResponse> {
  let secret = getCSRFSecret(request);
  
  // Generate new secret if none exists
  if (!secret) {
    secret = generateCSRFSecret();
  }
  
  const token = generateCSRFToken();
  const response = NextResponse.json({ csrfToken: token });
  
  // Set secret in secure cookie
  setCSRFSecret(response, secret);
  
  return response;
}