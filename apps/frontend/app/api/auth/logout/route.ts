/**
 * Frontend Logout API Route
 * OIDC Migration: Properly clears OIDC cookies and revokes backend tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { getBackendUrl, getFrontendUrl } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Processing OIDC logout request');

    // OIDC Migration: Get tokens from OIDC cookies before clearing
    const accessToken = request.cookies.get('access_token')?.value;
    const idToken = request.cookies.get('id_token')?.value;
    const refreshToken = request.cookies.get('refresh_token')?.value;

    console.log('🔍 OIDC Logout - Token Check:', {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing',
      refreshToken: refreshToken ? 'present' : 'missing'
    });

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'OIDC logout completed successfully',
      tokenType: 'oidc'
    });

    // OIDC Migration: Clear all OIDC authentication cookies
    response.cookies.delete('access_token');
    response.cookies.delete('id_token'); 
    response.cookies.delete('refresh_token');
    
    // Also clear legacy JWT cookie for backwards compatibility
    response.cookies.delete('epsx_frontend_jwt');

    console.log('✅ Frontend: All OIDC cookies cleared');

    // Call backend OIDC token revocation endpoint
    if (refreshToken || accessToken) {
      try {
        const backendUrl = getBackendUrl('server');
        
        // Revoke refresh token (preferred) or access token
        const tokenToRevoke = refreshToken || accessToken;
        const tokenTypeHint = refreshToken ? 'refresh_token' : 'access_token';
        
        console.log(`🔄 Revoking ${tokenTypeHint} with backend...`);
        
        const revokeResponse = await fetch(`${backendUrl}/api/v1/oidc/token/revoke`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          body: new URLSearchParams([
            ['token', tokenToRevoke!],
            ['token_type_hint', tokenTypeHint]
          ]).toString(),
          signal: AbortSignal.timeout(5000), // 5 second timeout
        });

        if (revokeResponse.ok) {
          console.log('✅ Frontend: OIDC token revocation successful');
        } else {
          console.warn('⚠️ Frontend: OIDC token revocation failed, but cookies cleared');
        }
        
      } catch (backendError) {
        console.warn('⚠️ Frontend: OIDC token revocation error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Frontend: No OIDC tokens found, skipping revocation');
    }

    // Standard OIDC RP-Initiated Logout (if ID token available)
    if (idToken) {
      try {
        const backendUrl = getBackendUrl('server');
        const frontendUrl = getFrontendUrl('server');
        
        console.log('🔄 Initiating OIDC RP-Initiated Logout...');
        
        // Standard OpenID Connect RP-Initiated Logout
        const logoutParams = new URLSearchParams({
          id_token_hint: idToken,
          post_logout_redirect_uri: `${frontendUrl}/`,
          state: 'frontend-oidc-logout'
        });
        
        const logoutResponse = await fetch(`${backendUrl}/api/v1/oidc/logout?${logoutParams.toString()}`, {
          method: 'GET',
          signal: AbortSignal.timeout(5000),
        });

        if (logoutResponse.ok) {
          console.log('✅ Frontend: OIDC RP-Initiated logout successful');
        } else {
          console.warn('⚠️ Frontend: OIDC RP-Initiated logout failed, but tokens revoked');
        }
        
      } catch (logoutError) {
        console.warn('⚠️ Frontend: OIDC RP-Initiated logout error:', logoutError);
      }
    }

    console.log('✅ Frontend: OIDC logout completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Frontend: OIDC Logout error:', error);

    // Still try to clear cookies even if there's an error
    const response = NextResponse.json({ 
      success: false,
      error: 'OIDC logout failed',
      message: 'An error occurred during OIDC logout'
    }, { status: 500 });

    // Emergency cookie cleanup
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');
    response.cookies.delete('epsx_frontend_jwt');

    return response;
  }
}

// Also support GET method for simple logout links
export async function GET(request: NextRequest) {
  return POST(request);
}