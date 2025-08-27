/**
 * PKCE (Proof Key for Code Exchange) utility functions
 * RFC 7636 compliant implementation
 */

import { randomBytes } from 'crypto';

/**
 * Generate a cryptographically secure code verifier
 */
export function generateCodeVerifier(): string {
  return randomBytes(32)
    .toString('base64url')
    .replace(/[+/]/g, '')
    .replace(/=+$/, '');
}

/**
 * Generate code challenge from code verifier using SHA256
 */
export async function generateCodeChallenge(codeVerifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(codeVerifier);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return btoa(String.fromCharCode(...new Uint8Array(hash)))
    .replace(/[+/]/g, (match) => (match === '+' ? '-' : '_'))
    .replace(/=+$/, '');
}

/**
 * Generate a cryptographically secure state parameter
 */
export function generateState(): string {
  return randomBytes(16).toString('base64url');
}