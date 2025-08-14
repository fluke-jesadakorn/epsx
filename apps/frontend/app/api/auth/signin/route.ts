/**
 * Frontend Main Sign In Route
 * Redirects to EPSX backend OAuth flow
 */
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  // Get callback URL from query parameters
  const { searchParams } = new URL(request.url);
  const callbackUrl = searchParams.get('callbackUrl') || '/dashboard';
  
  // Redirect to EPSX backend signin with callback URL
  const backendSigninUrl = new URL('/api/auth/signin/epsx-backend', request.url);
  if (callbackUrl) {
    backendSigninUrl.searchParams.set('callbackUrl', callbackUrl);
  }
  
  return NextResponse.redirect(backendSigninUrl);
}

// Allow POST method as well
export async function POST(request: NextRequest) {
  return GET(request);
}