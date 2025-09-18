/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest } from 'next/server';
import { processOAuthCallback } from '../../../../../../../shared/auth/oauth-callback';
import { exchangeCodeForTokens } from '@/lib/server/auth';

export async function GET(request: NextRequest) {
  const config = {
    appType: 'frontend' as const,
    exchangeCodeForTokens,
    defaultRedirectUrl: '/'
  };
  
  return processOAuthCallback(request, config);
}