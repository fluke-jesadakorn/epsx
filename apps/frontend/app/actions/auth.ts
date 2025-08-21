'use server';

import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';
import { getAuthUser } from '@/lib/server/auth';
import { serverConfig } from '@/config/env';

/**
 * Sign in with OIDC - redirects to backend OAuth authorization
 */
export async function signIn(email: string, password: string) {
  try {
    // For credential-based login, redirect to backend login form
    const backendUrl = serverConfig.backendUrl;
    const callbackUrl = `${serverConfig.siteUrl}/api/auth/callback/epsx-backend`;
    
    // Create state with current URL for redirect after login
    const state = Buffer.from(JSON.stringify({ 
      redirectTo: '/',
      loginType: 'credentials',
      email: email 
    })).toString('base64url');
    
    const params = new URLSearchParams({
      client_id: 'epsx-frontend',
      response_type: 'code',
      scope: 'openid profile email',
      redirect_uri: callbackUrl,
      state: state,
      // Pass email as hint for pre-filling
      login_hint: email
    });
    
    const loginUrl = `${backendUrl}/oauth/authorize?${params.toString()}`;
    redirect(loginUrl);
    
  } catch (error) {
    console.error('❌ Sign in failed:', error);
    return { success: false, error: 'Login failed. Please try again.' };
  }
}

/**
 * Sign up - redirects to backend registration form
 */
export async function signUp(email: string, password: string, name: string) {
  try {
    const backendUrl = serverConfig.backendUrl;
    
    // Redirect to backend registration with pre-filled data
    const params = new URLSearchParams({
      email: email,
      name: name,
      redirect_to: `${serverConfig.siteUrl}/`
    });
    
    const registerUrl = `${backendUrl}/oauth/register?${params.toString()}`;
    redirect(registerUrl);
    
  } catch (error) {
    console.error('❌ Sign up failed:', error);
    return { success: false, error: 'Registration failed. Please try again.' };
  }
}

/**
 * Request password reset - redirects to backend forgot password
 */
export async function forgotPassword(email: string) {
  try {
    const backendUrl = serverConfig.backendUrl;
    
    // Redirect to backend password reset with email pre-filled
    const params = new URLSearchParams({
      email: email,
      redirect_to: `${serverConfig.siteUrl}/login`
    });
    
    const resetUrl = `${backendUrl}/oauth/reset-password?${params.toString()}`;
    redirect(resetUrl);
    
  } catch (error) {
    console.error('❌ Password reset request failed:', error);
    return { success: false, error: 'Password reset failed. Please try again.' };
  }
}

/**
 * Reset password with token - redirects to backend reset confirmation
 */
export async function resetPassword(token: string, password: string) {
  try {
    const backendUrl = serverConfig.backendUrl;
    
    // Redirect to backend password reset confirmation with token
    const params = new URLSearchParams({
      token: token,
      redirect_to: `${serverConfig.siteUrl}/login`
    });
    
    const resetUrl = `${backendUrl}/oauth/reset-password/confirm?${params.toString()}`;
    redirect(resetUrl);
    
  } catch (error) {
    console.error('❌ Password reset failed:', error);
    return { success: false, error: 'Password reset failed. Please try again.' };
  }
}

/**
 * Request password reset action (alias for forgotPassword)
 */
export async function requestPasswordResetAction(email: string) {
  return await forgotPassword(email);
}

/**
 * Reset password action (alias for resetPassword)
 */
export async function resetPasswordAction(token: string, password: string) {
  return await resetPassword(token, password);
}

/**
 * Require guest - redirect authenticated users to dashboard
 */
export async function requireGuest() {
  try {
    const user = await getAuthUser();
    
    if (user) {
      // User is authenticated, redirect to dashboard
      redirect('/dashboard');
    }
    
    return true;
  } catch (error) {
    console.error('❌ Guest check failed:', error);
    // If there's an error, assume not authenticated and allow access
    return true;
  }
}