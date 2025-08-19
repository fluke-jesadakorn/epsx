/**
 * Server-side OAuth utilities for admin-frontend
 */

/**
 * Generate PKCE code verifier
 */
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Generate PKCE code challenge
 */
async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(digest));
}

/**
 * Generate random string
 */
function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Base64 URL encode
 */
function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Generate authorization URL with PKCE parameters
 */
export async function getAuthorizationUrl() {
  // Use consolidated auth config
  const { authConfig } = await import('../../config/env');
  
  console.log('🔄 Admin: Generating PKCE parameters for OAuth authorization...');
  
  // Generate PKCE parameters (server-side only)
  const codeVerifier = generateCodeVerifier();
  console.log('✅ Admin: Code verifier generated successfully');
  
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  console.log('✅ Admin: Code challenge generated successfully');
  
  const state = generateRandomString(32);
  console.log('✅ Admin: State parameter generated successfully');
  
  // Build authorization URL using consolidated config
  const authorizationEndpoint = authConfig.authorizationEndpoint;
  const clientId = authConfig.clientId;
  const redirectUri = authConfig.callbackUrl;
  
  console.log('🔧 Admin: OAuth configuration:', {
    authorizationEndpoint,
    clientId,
    redirectUri,
    scope: 'openid profile email admin_modules'
  });
  
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'openid profile email admin_modules',
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });
  
  const url = `${authorizationEndpoint}?${params.toString()}`;
  console.log('✅ Admin: Authorization URL generated successfully:', url);
  
  return {
    url,
    codeVerifier,
    state,
  };
}

/**
 * Get server session (mock implementation for build)
 */
export async function getServerSession() {
  // TODO: Implement actual session retrieval
  return null;
}

/**
 * Get auth user (mock implementation for build)
 */
export async function getAuthUser() {
  // TODO: Implement actual user retrieval
  return null;
}

/**
 * Fetch user info from OAuth userinfo endpoint
 */
export async function getUserInfo(accessToken: string) {
  // Use consolidated auth config
  const { authConfig } = await import('../../config/env');
  const apiUrl = authConfig.apiUrl;
  
  console.log('🔄 Admin: Fetching user info from backend userinfo endpoint');
  const response = await fetch(`${apiUrl}/oauth/userinfo`, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error('❌ Admin: UserInfo fetch failed:', response.status, response.statusText, errorText);
    throw new Error(`UserInfo fetch failed: ${response.status} ${response.statusText} - ${errorText}`);
  }

  const userinfo = await response.json();
  console.log('✅ Admin: Successfully received user info');
  return userinfo;
}