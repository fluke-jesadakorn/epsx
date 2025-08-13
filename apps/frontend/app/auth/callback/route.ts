/**
 * Auth.js OIDC Callback Route
 * 
 * Note: This file is kept for compatibility but Auth.js handles
 * the actual OIDC callback automatically via /api/auth/callback/epsx-backend
 * 
 * Redirects to Auth.js callback handler for proper OIDC flow
 */

import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  // Redirect to Auth.js OIDC callback handler
  const { searchParams } = new URL(request.url);
  const authCallbackUrl = new URL('/api/auth/callback/epsx-backend', request.url);
  
  // Pass through all parameters to Auth.js callback
  for (const [key, value] of searchParams.entries()) {
    authCallbackUrl.searchParams.set(key, value);
  }
  
  console.log('🔐 Redirecting to Auth.js OIDC callback:', authCallbackUrl.toString());
  
  return NextResponse.redirect(authCallbackUrl);
}