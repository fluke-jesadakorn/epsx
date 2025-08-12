'use server';

// OIDC Server Actions
// Server-side authentication flow for OpenID Connect integration with backend

import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';
import { getIronSession } from 'iron-session';
import { jwtVerify, importJWK } from 'jose';
import DOMPurify from 'dompurify';
import { JSDOM } from 'jsdom';
import { getOIDCDiscoveryClient, type OIDCConfiguration } from '@/lib/auth/oidc-discovery-client';
import { getTenantDetectionService } from '@/lib/auth/tenant-detection-service';

// Session configuration
const sessionConfig = {
  password: process.env.SESSION_SECRET || 'complex_password_at_least_32_characters_long!',
  cookieName: 'epsx_oidc_session',
  cookieOptions: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 60 * 60 * 24 * 7, // 1 week
    sameSite: 'lax' as const,
  },
};

interface OIDCSession {
  user?: {
    id: string;
    email: string;
    name?: string;
    role: string;
    permissions: string[];
    tenant_id?: string;
    provider: string;
  };
  tokens?: {
    access_token: string;
    id_token: string;
    refresh_token?: string;
    expires_at: number;
  };
  state?: string;
  code_verifier?: string;
  tenant_id?: string;
}

interface LoginParams {
  email?: string;
  tenant_id?: string;
  redirect_uri?: string;
}

interface CallbackParams {
  code: string;
  state: string;
  tenant_id?: string;
}

interface AuthResult {
  success: boolean;
  error?: string;
  redirect_url?: string;
}

/**
 * Initialize OIDC login flow
 */
export async function initiateOIDCLogin(params: LoginParams): Promise<AuthResult> {
  try {
    const { email, tenant_id, redirect_uri = '/dashboard' } = params;
    const discoveryClient = getOIDCDiscoveryClient();
    const tenantService = getTenantDetectionService();
    
    // Detect tenant if not provided
    let detectedTenantId = tenant_id;
    if (!detectedTenantId && email) {
      const detection = await tenantService.detectTenant(email);
      detectedTenantId = detection.tenant?.tenant_id;
    }

    // Get OIDC configuration
    const config = await discoveryClient.discoverConfiguration(detectedTenantId);
    
    // Generate PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateState();
    
    // Prepare authorization URL
    const authUrl = new URL(config.authorization_endpoint);
    authUrl.searchParams.set('client_id', process.env.OIDC_CLIENT_ID || 'epsx-frontend');
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('scope', 'openid profile email');
    authUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000'}/auth/callback`);
    authUrl.searchParams.set('state', state);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    
    // Add tenant hint if available
    if (detectedTenantId) {
      authUrl.searchParams.set('tenant_id', detectedTenantId);
    }
    
    // Store session data
    const session = await getSession();
    session.state = state;
    session.code_verifier = codeVerifier;
    session.tenant_id = detectedTenantId;
    await session.save();
    
    console.log('🚀 Initiating OIDC login flow:', {
      tenant: detectedTenantId,
      issuer: config.issuer,
      redirect_uri
    });
    
    return {
      success: true,
      redirect_url: authUrl.toString()
    };
    
  } catch (error) {
    console.error('❌ OIDC login initiation failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Login initiation failed'
    };
  }
}

/**
 * Handle OIDC callback
 */
export async function handleOIDCCallback(params: CallbackParams): Promise<AuthResult> {
  try {
    const { code, state, tenant_id } = params;
    const discoveryClient = getOIDCDiscoveryClient();
    
    // Verify session and state
    const session = await getSession();
    if (!session.state || session.state !== state) {
      throw new Error('Invalid state parameter - possible CSRF attack');
    }
    
    if (!session.code_verifier) {
      throw new Error('Missing code verifier in session');
    }
    
    const effectiveTenantId = tenant_id || session.tenant_id;
    const config = await discoveryClient.discoverConfiguration(effectiveTenantId);
    
    // Exchange code for tokens
    const tokenResponse = await fetch(config.token_endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Accept': 'application/json',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: `${process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000'}/auth/callback`,
        client_id: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
        code_verifier: session.code_verifier,
      }),
    });

    if (!tokenResponse.ok) {
      const errorData = await tokenResponse.text();
      throw new Error(`Token exchange failed: ${tokenResponse.status} ${errorData}`);
    }

    const tokens = await tokenResponse.json();
    
    // Verify and decode tokens
    const userInfo = await verifyAndDecodeTokens(tokens, config);
    
    // Store authenticated session
    session.user = userInfo;
    session.tokens = {
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      refresh_token: tokens.refresh_token,
      expires_at: Date.now() + (tokens.expires_in * 1000),
    };
    
    // Clear temporary data
    delete session.state;
    delete session.code_verifier;
    
    await session.save();
    
    console.log('✅ OIDC callback processed successfully:', userInfo.email);
    
    return {
      success: true,
      redirect_url: '/dashboard'
    };
    
  } catch (error) {
    console.error('❌ OIDC callback failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Authentication callback failed'
    };
  }
}

/**
 * Get current authenticated user
 */
export async function getCurrentOIDCUser() {
  try {
    const session = await getSession();
    
    // Check if session is valid and not expired
    if (!session.user || !session.tokens) {
      return null;
    }
    
    if (session.tokens.expires_at < Date.now()) {
      // Try to refresh token
      const refreshResult = await refreshOIDCToken();
      if (!refreshResult.success) {
        await clearOIDCSession();
        return null;
      }
      
      // Get updated session
      const updatedSession = await getSession();
      return updatedSession.user || null;
    }
    
    return session.user;
    
  } catch (error) {
    console.error('❌ Failed to get current OIDC user:', error);
    return null;
  }
}

/**
 * Refresh OIDC token
 */
export async function refreshOIDCToken(): Promise<AuthResult> {
  try {
    const session = await getSession();
    
    if (!session.tokens?.refresh_token || !session.tenant_id) {
      throw new Error('No refresh token available');
    }
    
    const discoveryClient = getOIDCDiscoveryClient();
    const config = await discoveryClient.discoverConfiguration(session.tenant_id);
    
    const tokenResponse = await fetch(config.token_endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Accept': 'application/json',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: session.tokens.refresh_token,
        client_id: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
      }),
    });

    if (!tokenResponse.ok) {
      throw new Error(`Token refresh failed: ${tokenResponse.status}`);
    }

    const tokens = await tokenResponse.json();
    
    // Update session with new tokens
    session.tokens = {
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      refresh_token: tokens.refresh_token || session.tokens.refresh_token,
      expires_at: Date.now() + (tokens.expires_in * 1000),
    };
    
    await session.save();
    
    console.log('🔄 OIDC token refreshed successfully');
    
    return { success: true };
    
  } catch (error) {
    console.error('❌ OIDC token refresh failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Token refresh failed'
    };
  }
}

/**
 * Clear OIDC session and logout
 */
export async function logoutOIDC(): Promise<void> {
  try {
    const session = await getSession();
    
    // Clear session data
    await clearOIDCSession();
    
    console.log('✅ OIDC logout completed');
    
  } catch (error) {
    console.error('❌ OIDC logout failed:', error);
    throw error;
  } finally {
    redirect('/login');
  }
}

/**
 * Clear session data
 */
export async function clearOIDCSession(): Promise<void> {
  const session = await getSession();
  session.destroy();
}

/**
 * Get iron session
 */
async function getSession() {
  const cookieStore = await cookies();
  return getIronSession<OIDCSession>(cookieStore, sessionConfig);
}

/**
 * Verify and decode JWT tokens
 */
async function verifyAndDecodeTokens(tokens: any, config: OIDCConfiguration) {
  const discoveryClient = getOIDCDiscoveryClient();
  
  // Get JWKS for verification
  const jwks = await discoveryClient.getJWKS();
  
  // Find appropriate key
  const key = jwks.keys?.[0];
  if (!key) {
    throw new Error('No signing key found in JWKS');
  }
  
  const publicKey = await importJWK(key);
  
  // Verify ID token
  const { payload: idToken } = await jwtVerify(tokens.id_token, publicKey, {
    issuer: config.issuer,
    audience: process.env.OIDC_CLIENT_ID || 'epsx-frontend',
  });
  
  // Sanitize user data
  const window = new JSDOM('').window;
  const purify = DOMPurify(window);
  
  return {
    id: String(idToken.sub),
    email: purify.sanitize(String(idToken.email || '')),
    name: purify.sanitize(String(idToken.name || '')),
    role: String(idToken.role || 'user'),
    permissions: Array.isArray(idToken.permissions) ? idToken.permissions : [],
    tenant_id: String(idToken.tenant_id || ''),
    provider: 'oidc',
  };
}

/**
 * Generate PKCE code verifier
 */
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode.apply(null, Array.from(array)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Generate PKCE code challenge
 */
async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return btoa(String.fromCharCode.apply(null, Array.from(new Uint8Array(digest))))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Generate random state parameter
 */
function generateState(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode.apply(null, Array.from(array)));
}