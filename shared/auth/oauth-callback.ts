/**
 * Shared OAuth Callback Processing Utilities
 * Consolidates OAuth authorization callback logic
 * Used by both frontend and admin-frontend applications
 */

import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { setOIDCTokens, clearPKCECookies, clearLegacyCookies, PKCE_COOKIES } from './cookies';

export interface OAuthCallbackConfig {
  appType: 'frontend' | 'admin';
  exchangeCodeForTokens: (code: string, codeVerifier: string, state: string) => Promise<any>;
  validateUserPermissions?: (tokens: any) => Promise<{ isValid: boolean; userInfo?: any; error?: string }>;
  defaultRedirectUrl?: string;
}

export interface CallbackValidationResult {
  success: boolean;
  tokens?: any;
  userInfo?: any;
  redirectUrl?: string;
  error?: string;
  status?: number;
}

export async function processOAuthCallback(
  request: NextRequest,
  config: OAuthCallbackConfig
): Promise<NextResponse> {
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  console.log(`🚨 ${appLabel.toUpperCase()} CALLBACK ROUTE CALLED - START`);
  console.log(`🚨 Request URL: ${request.url}`);
  console.log(`🚨 Time: ${new Date().toISOString()}`);
  
  try {
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    console.log(`🔄 ${appLabel}: EPSX Backend OAuth callback received:`, {
      code: code ? 'present' : 'missing',
      state: state ? 'present' : 'missing',
      error,
      errorDescription,
    });

    // Handle OAuth errors
    if (error) {
      console.error(`❌ ${appLabel}: OAuth callback error:`, error, errorDescription);
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', error);
      if (errorDescription) {
        loginUrl.searchParams.set('error_description', errorDescription);
      }
      return NextResponse.redirect(loginUrl);
    }

    // Validate required parameters
    if (!code || !state) {
      console.error(`❌ ${appLabel}: Missing required OAuth parameters`);
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_parameters');
      return NextResponse.redirect(loginUrl);
    }

    // Get validation result
    const validationResult = await validateCallback(request, { code, state }, config);
    
    if (!validationResult.success) {
      console.error(`❌ ${appLabel}: Callback validation failed:`, validationResult.error);
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', validationResult.error || 'callback_validation_failed');
      return NextResponse.redirect(loginUrl);
    }

    // Create redirect response
    const redirectUrl = new URL(validationResult.redirectUrl || '/', request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // Set OIDC tokens
    setOIDCTokens(response, validationResult.tokens);
    
    // Clean up cookies
    clearPKCECookies(response);
    clearLegacyCookies(response);

    console.log(`✅ ${appLabel}: Callback completed successfully with OIDC cookies`);
    console.log(`🚨 ${appLabel.toUpperCase()} CALLBACK ROUTE COMPLETED - END`);
    return response;

  } catch (error) {
    console.error(`❌ ${appLabel}: OAuth callback processing error:`, error);
    
    if (error instanceof Error) {
      console.error(`❌ ${appLabel}: Error details:`, {
        message: error.message,
        stack: error.stack,
      });
    }

    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    console.log(`🚨 ${appLabel.toUpperCase()} CALLBACK ROUTE CALLED - END (ERROR)`);
    return NextResponse.redirect(loginUrl);
  }
}

async function validateCallback(
  request: NextRequest,
  params: { code: string; state: string },
  config: OAuthCallbackConfig
): Promise<CallbackValidationResult> {
  const { code, state } = params;
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  // Retrieve PKCE parameters from cookies
  const cookieStore = await cookies();
  let storedCodeVerifier = cookieStore.get(PKCE_COOKIES.CODE_VERIFIER)?.value;
  let storedState = cookieStore.get(PKCE_COOKIES.STATE)?.value;
  const storedCallbackUrl = cookieStore.get(PKCE_COOKIES.CALLBACK_URL)?.value || 
                           cookieStore.get(PKCE_COOKIES.REDIRECT_TO)?.value;

  // Admin-specific: Check backup cookies if primary missing
  if (config.appType === 'admin') {
    if (!storedCodeVerifier) {
      storedCodeVerifier = cookieStore.get('pkce_verifier_backup')?.value || 
                          cookieStore.get('admin_oauth_verifier')?.value;
      if (storedCodeVerifier) {
        console.log('🔄 Using backup code verifier');
      }
    }
    
    if (!storedState) {
      storedState = cookieStore.get('pkce_state_backup')?.value ||
                   cookieStore.get('admin_oauth_state')?.value;
      if (storedState) {
        console.log('🔄 Using backup state');
      }
    }
  }

  // Comprehensive cookie debugging for admin
  if (config.appType === 'admin') {
    const allCookies = cookieStore.getAll();
    const authCookies = allCookies.filter(cookie => 
      cookie.name.includes('oauth') || 
      cookie.name.includes('pkce') || 
      cookie.name.includes('admin')
    );
    console.log(`🔍 ${appLabel} All auth-related cookies:`, authCookies.map(c => ({
      name: c.name,
      value: c.value ? `${c.value.slice(0,10)}...` : 'empty'
    })));
  }

  console.log(`🔍 ${appLabel} Cookie debug info:`, {
    oauth_code_verifier: storedCodeVerifier ? `present (${storedCodeVerifier.slice(0,10)}...)` : 'missing',
    oauth_state: storedState ? `present (${storedState.slice(0,10)}...)` : 'missing',
    callback_url: storedCallbackUrl || 'missing'
  });

  // Final fallback: If still missing and in development, provide helpful error
  if (!storedCodeVerifier || !storedState) {
    if (config.appType === 'admin') {
      console.error('❌ Admin: All PKCE parameter sources failed:', {
        primaryCookies: 'missing',
        backupCookies: 'missing', 
        additionalCookies: 'missing',
        suggestion: 'Try logging out completely and starting fresh OAuth flow'
      });
    }
    
    return {
      success: false,
      error: 'missing_pkce_parameters'
    };
  }

  // Validate state parameter
  if (state !== storedState) {
    return {
      success: false,
      error: 'state_mismatch'
    };
  }
  
  console.log(`✅ ${appLabel}: State validation passed`);

  // Exchange authorization code for OIDC tokens
  console.log(`🔄 ${appLabel}: Exchanging authorization code for OIDC tokens...`);
  const tokens = await config.exchangeCodeForTokens(code, storedCodeVerifier, state);
  
  console.log(`✅ ${appLabel}: Successfully received OIDC tokens:`, {
    accessToken: tokens.accessToken ? 'present' : 'missing',
    refreshToken: tokens.refreshToken ? 'present' : 'missing', 
    idToken: tokens.idToken ? 'present' : 'missing'
  });

  // Validate that we received all required OIDC tokens
  if (!tokens.accessToken || !tokens.refreshToken || !tokens.idToken) {
    return {
      success: false,
      error: 'incomplete_oidc_tokens'
    };
  }

  // Optional user permission validation (for admin)
  if (config.validateUserPermissions) {
    console.log(`🔄 ${appLabel}: Validating user permissions...`);
    const permissionResult = await config.validateUserPermissions(tokens);
    
    if (!permissionResult.isValid) {
      return {
        success: false,
        error: permissionResult.error || 'insufficient_permissions',
        status: 403
      };
    }
    
    console.log(`✅ ${appLabel}: User validated with required permissions`);
  }

  // Get callback URL
  const redirectUrl = storedCallbackUrl || config.defaultRedirectUrl || '/';
  
  return {
    success: true,
    tokens,
    redirectUrl
  };
}