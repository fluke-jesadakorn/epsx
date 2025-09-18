/**
 * Frontend Session API Route
 * OIDC Migration: Handles session management via OIDC-compliant cookies
 */
import { NextRequest } from 'next/server';
import { getSession, storeSession, clearSession, refreshSession } from '../../../../../../shared/auth/session';

export async function GET() {
  const config = {
    appType: 'frontend' as const,
    requireAdminPermissions: false
  };
  
  return getSession(config);
}

export async function POST(request: NextRequest) {
  const config = {
    appType: 'frontend' as const,
    requireAdminPermissions: false,
    legacyJwtCookie: 'epsx_frontend_jwt'
  };
  
  return storeSession(request, config);
}

export async function PUT() {
  const config = {
    appType: 'frontend' as const,
    requireAdminPermissions: false
  };
  
  return refreshSession(config);
}

export async function DELETE() {
  const config = {
    appType: 'frontend' as const,
    requireAdminPermissions: false
  };
  
  return clearSession(config);
}