/**
 * OIDC Callback Route for EPSX Backend
 * Handles OIDC authorization callback and sets OIDC tokens as HttpOnly cookies
 * OIDC Migration: Uses standard OIDC tokens instead of custom JWT
 */
import { NextRequest } from 'next/server';
import { processOAuthCallback } from '../../../../../../shared/auth/oauth-callback';
import { getUserInfo, exchangeCodeForTokens } from '@/lib/server/auth';

async function validateAdminPermissions(tokens: any) {
  try {
    const userinfo = await getUserInfo(tokens.accessToken);
    
    const hasAdminAccess = userinfo.permissions && userinfo.permissions.some((permission: string) => 
      permission === 'admin:*:*' || permission.startsWith('admin:')
    );
    
    if (!hasAdminAccess) {
      console.error('❌ Admin: User lacks admin permissions', {
        email: userinfo.email,
        permissions: userinfo.permissions
      });
      return {
        isValid: false,
        error: 'insufficient_admin_permissions'
      };
    }

    console.log('✅ Admin: User validated with admin permissions:', {
      email: userinfo.email,
      adminPermissions: userinfo.permissions.filter((p: string) => p.startsWith('admin:'))
    });

    return {
      isValid: true,
      userInfo: userinfo
    };
  } catch (error) {
    console.error('❌ Admin: Failed to validate permissions:', error);
    return {
      isValid: false,
      error: 'permission_validation_failed'
    };
  }
}

export async function GET(request: NextRequest) {
  const config = {
    appType: 'admin' as const,
    exchangeCodeForTokens,
    validateUserPermissions: validateAdminPermissions,
    defaultRedirectUrl: '/'
  };
  
  return processOAuthCallback(request, config);
}