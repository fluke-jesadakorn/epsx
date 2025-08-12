'use server';

// Admin OIDC Server Actions
// Server-side authentication flow for admin panel with enhanced security

import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';
import { getIronSession } from 'iron-session';
import { jwtVerify, importJWK } from 'jose';
import DOMPurify from 'dompurify';
import { JSDOM } from 'jsdom';

// Import discovery client and utilities from frontend (shared)
// Note: In a real implementation, these would be in a shared package
const getOIDCDiscoveryClient = () => {
  // Simplified implementation - would import from shared package
  return {
    async discoverConfiguration(tenantId?: string) {
      const baseUrl = process.env.BACKEND_URL || 'http://localhost:8080';
      const discoveryUrl = tenantId 
        ? `${baseUrl}/oauth/v2/${tenantId}/.well-known/openid-configuration`
        : `${baseUrl}/oauth/v2/.well-known/openid-configuration`;
      
      const response = await fetch(discoveryUrl);
      if (!response.ok) throw new Error('OIDC discovery failed');
      return response.json();
    },
    async getJWKS(tenantId?: string) {
      const config = await this.discoverConfiguration(tenantId);
      const response = await fetch(config.jwks_uri);
      if (!response.ok) throw new Error('JWKS fetch failed');
      return response.json();
    }
  };
};

// Admin session configuration with enhanced security
const adminSessionConfig = {
  password: process.env.ADMIN_SESSION_SECRET || process.env.SESSION_SECRET || 'admin_complex_password_at_least_32_characters_long!',
  cookieName: 'epsx_admin_oidc_session',
  cookieOptions: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 60 * 60 * 8, // 8 hours (shorter than regular users)
    sameSite: 'strict' as const, // Stricter than regular frontend
    path: '/',
  },
};

interface AdminOIDCSession {
  user?: {
    id: string;
    email: string;
    name?: string;
    role: string;
    permissions: string[];
    admin_level: 'moderator' | 'admin' | 'super_admin';
    tenant_id?: string;
    provider: string;
    last_activity: number;
  };
  tokens?: {
    access_token: string;
    id_token: string;
    refresh_token?: string;
    expires_at: number;
  };
  security?: {
    login_time: number;
    last_activity: number;
    ip_address?: string;
    user_agent?: string;
  };
  state?: string;
  code_verifier?: string;
  tenant_id?: string;
}

interface AdminLoginParams {
  tenant_id?: string;
  redirect_uri?: string;
  require_admin_level?: 'moderator' | 'admin' | 'super_admin';
}

interface AdminCallbackParams {
  code: string;
  state: string;
  tenant_id?: string;
}

interface AdminAuthResult {
  success: boolean;
  error?: string;
  redirect_url?: string;
  requires_elevation?: boolean;
}

// Admin role hierarchy
const ADMIN_ROLES = [
  'super_admin',
  'admin',
  'admin-full-004',
  'moderator-standard-003',
  'moderator',
  'system_administrator'
];

const ADMIN_LEVELS = {
  'super_admin': 3,
  'admin': 2,
  'admin-full-004': 2,
  'system_administrator': 3,
  'moderator': 1,
  'moderator-standard-003': 1
};

/**
 * Initialize admin OIDC login flow
 */
export async function initiateAdminOIDCLogin(params: AdminLoginParams): Promise<AdminAuthResult> {
  try {
    const { tenant_id, redirect_uri = '/dashboard', require_admin_level = 'moderator' } = params;
    const discoveryClient = getOIDCDiscoveryClient();
    
    // Get OIDC configuration - admin typically uses default tenant
    const config = await discoveryClient.discoverConfiguration(tenant_id);
    
    // Generate PKCE parameters with enhanced entropy
    const codeVerifier = generateSecureCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateSecureState();
    
    // Prepare authorization URL with admin-specific parameters
    const authUrl = new URL(config.authorization_endpoint);
    authUrl.searchParams.set('client_id', process.env.OIDC_ADMIN_CLIENT_ID || process.env.OIDC_CLIENT_ID || 'epsx-admin');
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('scope', 'openid profile email admin');
    authUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/auth/callback`);
    authUrl.searchParams.set('state', state);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    
    // Add admin-specific hints
    authUrl.searchParams.set('prompt', 'login'); // Always require fresh login for admin
    authUrl.searchParams.set('max_age', '0'); // No SSO for admin access
    
    if (tenant_id) {
      authUrl.searchParams.set('tenant_id', tenant_id);
    }
    
    // Store session data with security info
    const session = await getAdminSession();
    session.state = state;
    session.code_verifier = codeVerifier;
    session.tenant_id = tenant_id;
    session.security = {
      login_time: Date.now(),
      last_activity: Date.now(),
    };
    await session.save();
    
    console.log('🔐 Initiating admin OIDC login flow:', {
      tenant: tenant_id,
      issuer: config.issuer,
      require_admin_level
    });
    
    return {
      success: true,
      redirect_url: authUrl.toString()
    };
    
  } catch (error) {
    console.error('❌ Admin OIDC login initiation failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Admin login initiation failed'
    };
  }
}

/**
 * Handle admin OIDC callback
 */
export async function handleAdminOIDCCallback(params: AdminCallbackParams): Promise<AdminAuthResult> {
  try {
    const { code, state, tenant_id } = params;
    const discoveryClient = getOIDCDiscoveryClient();
    
    // Verify session and state
    const session = await getAdminSession();
    if (!session.state || session.state !== state) {
      throw new Error('Invalid state parameter - possible CSRF attack on admin panel');
    }
    
    if (!session.code_verifier) {
      throw new Error('Missing code verifier in admin session');
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
        redirect_uri: `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/auth/callback`,
        client_id: process.env.OIDC_ADMIN_CLIENT_ID || process.env.OIDC_CLIENT_ID || 'epsx-admin',
        code_verifier: session.code_verifier,
      }),
    });

    if (!tokenResponse.ok) {
      const errorData = await tokenResponse.text();
      throw new Error(`Admin token exchange failed: ${tokenResponse.status} ${errorData}`);
    }

    const tokens = await tokenResponse.json();
    
    // Verify and decode tokens with admin validation
    const userInfo = await verifyAndDecodeAdminTokens(tokens, config);
    
    // Validate admin permissions
    const adminValidation = validateAdminPermissions(userInfo);
    if (!adminValidation.valid) {
      console.warn('🚫 Non-admin user attempted to access admin panel:', userInfo.email);
      return {
        success: false,
        error: 'Access denied: Administrative privileges required'
      };
    }
    
    // Store authenticated admin session
    session.user = {
      ...userInfo,
      admin_level: adminValidation.admin_level!,
      last_activity: Date.now(),
    };
    session.tokens = {
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      refresh_token: tokens.refresh_token,
      expires_at: Date.now() + (tokens.expires_in * 1000),
    };
    
    // Update security info
    session.security = {
      ...session.security!,
      last_activity: Date.now(),
    };
    
    // Clear temporary data
    delete session.state;
    delete session.code_verifier;
    
    await session.save();
    
    console.log('✅ Admin OIDC callback processed successfully:', {
      email: userInfo.email,
      role: userInfo.role,
      admin_level: adminValidation.admin_level
    });
    
    return {
      success: true,
      redirect_url: '/dashboard'
    };
    
  } catch (error) {
    console.error('❌ Admin OIDC callback failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Admin authentication callback failed'
    };
  }
}

/**
 * Get current authenticated admin user
 */
export async function getCurrentAdminOIDCUser() {
  try {
    const session = await getAdminSession();
    
    // Check if session is valid and not expired
    if (!session.user || !session.tokens) {
      return null;
    }
    
    // Check session timeout (more aggressive for admin)
    const maxInactivity = 60 * 60 * 1000; // 1 hour inactivity timeout
    if (session.user.last_activity < Date.now() - maxInactivity) {
      console.log('🔒 Admin session timed out due to inactivity');
      await clearAdminOIDCSession();
      return null;
    }
    
    if (session.tokens.expires_at < Date.now()) {
      // Try to refresh token
      const refreshResult = await refreshAdminOIDCToken();
      if (!refreshResult.success) {
        await clearAdminOIDCSession();
        return null;
      }
      
      // Get updated session
      const updatedSession = await getAdminSession();
      return updatedSession.user || null;
    }
    
    // Update last activity
    session.user.last_activity = Date.now();
    await session.save();
    
    return session.user;
    
  } catch (error) {
    console.error('❌ Failed to get current admin OIDC user:', error);
    return null;
  }
}

/**
 * Refresh admin OIDC token
 */
export async function refreshAdminOIDCToken(): Promise<AdminAuthResult> {
  try {
    const session = await getAdminSession();
    
    if (!session.tokens?.refresh_token || !session.tenant_id) {
      throw new Error('No admin refresh token available');
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
        client_id: process.env.OIDC_ADMIN_CLIENT_ID || process.env.OIDC_CLIENT_ID || 'epsx-admin',
      }),
    });

    if (!tokenResponse.ok) {
      throw new Error(`Admin token refresh failed: ${tokenResponse.status}`);
    }

    const tokens = await tokenResponse.json();
    
    // Update session with new tokens
    session.tokens = {
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      refresh_token: tokens.refresh_token || session.tokens.refresh_token,
      expires_at: Date.now() + (tokens.expires_in * 1000),
    };
    
    if (session.security) {
      session.security.last_activity = Date.now();
    }
    
    await session.save();
    
    console.log('🔄 Admin OIDC token refreshed successfully');
    
    return { success: true };
    
  } catch (error) {
    console.error('❌ Admin OIDC token refresh failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Admin token refresh failed'
    };
  }
}

/**
 * Clear admin OIDC session and logout
 */
export async function logoutAdminOIDC(): Promise<void> {
  try {
    const session = await getAdminSession();
    
    // Log admin logout for security audit
    if (session.user) {
      console.log('🔒 Admin logout:', {
        email: session.user.email,
        role: session.user.role,
        session_duration: Date.now() - (session.security?.login_time || Date.now())
      });
    }
    
    // Clear session data
    await clearAdminOIDCSession();
    
    console.log('✅ Admin OIDC logout completed');
    
  } catch (error) {
    console.error('❌ Admin OIDC logout failed:', error);
    throw error;
  } finally {
    redirect('/login');
  }
}

/**
 * Clear admin session data
 */
export async function clearAdminOIDCSession(): Promise<void> {
  const session = await getAdminSession();
  session.destroy();
}

/**
 * Check if user has specific admin permission
 */
export async function checkAdminPermission(permission: string): Promise<boolean> {
  try {
    const adminUser = await getCurrentAdminOIDCUser();
    if (!adminUser) return false;
    
    return adminUser.permissions.includes(permission) || 
           adminUser.permissions.includes('admin:*') ||
           adminUser.role === 'super_admin';
  } catch {
    return false;
  }
}

/**
 * Get admin session
 */
async function getAdminSession() {
  const cookieStore = await cookies();
  return getIronSession<AdminOIDCSession>(cookieStore, adminSessionConfig);
}

/**
 * Verify and decode admin JWT tokens
 */
async function verifyAndDecodeAdminTokens(tokens: any, config: any) {
  const discoveryClient = getOIDCDiscoveryClient();
  
  // Get JWKS for verification
  const jwks = await discoveryClient.getJWKS();
  const key = jwks.keys?.[0];
  if (!key) throw new Error('No signing key found in JWKS');
  
  const publicKey = await importJWK(key);
  
  // Verify ID token
  const { payload: idToken } = await jwtVerify(tokens.id_token, publicKey, {
    issuer: config.issuer,
    audience: process.env.OIDC_ADMIN_CLIENT_ID || process.env.OIDC_CLIENT_ID || 'epsx-admin',
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
 * Validate admin permissions
 */
function validateAdminPermissions(userInfo: any): { valid: boolean; admin_level?: 'moderator' | 'admin' | 'super_admin' } {
  // Check if user has admin role
  if (!ADMIN_ROLES.includes(userInfo.role)) {
    return { valid: false };
  }
  
  // Determine admin level
  let admin_level: 'moderator' | 'admin' | 'super_admin' = 'moderator';
  const roleLevel = ADMIN_LEVELS[userInfo.role as keyof typeof ADMIN_LEVELS] || 0;
  
  if (roleLevel >= 3) {
    admin_level = 'super_admin';
  } else if (roleLevel >= 2) {
    admin_level = 'admin';
  }
  
  return { valid: true, admin_level };
}

/**
 * Generate secure PKCE code verifier for admin
 */
function generateSecureCodeVerifier(): string {
  const array = new Uint8Array(64); // Larger entropy for admin
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
 * Generate secure random state parameter for admin
 */
function generateSecureState(): string {
  const array = new Uint8Array(32); // Larger entropy for admin
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode.apply(null, Array.from(array)));
}