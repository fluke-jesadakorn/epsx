/**
 * NextAuth.js Admin Callback Route
 * Redirects to NextAuth.js callback handler for compatibility
 */

import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  
  console.log('🔐 Legacy admin callback - redirecting to NextAuth handler');
  
  // Forward all query parameters to NextAuth callback
  const nextAuthCallbackUrl = new URL('/api/auth/callback/epsx-backend', request.url);
  
  // Preserve all query parameters (code, state, error, etc.)
  searchParams.forEach((value, key) => {
    nextAuthCallbackUrl.searchParams.set(key, value);
  });
  
  return NextResponse.redirect(nextAuthCallbackUrl);
}