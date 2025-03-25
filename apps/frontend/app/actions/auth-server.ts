'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import type { AuthResponse, TokenFeature, Permission, AuthActionResponse } from '@/types/auth';
import { UserRole } from '@/types/auth';
import { apiClient } from '@/lib/api-client';

export async function verifyAuth() {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get('__session');
  const email = cookieStore.get('email');
  const role = cookieStore.get('role');
  const tokenBalance = cookieStore.get('token_balance');
  const features = cookieStore.get('features');
  const permissions = cookieStore.get('permissions');

  if (!sessionToken || !email || !role) {
    return {
      isLoggedIn: false,
      userEmail: null,
      role: UserRole.GUEST,
      tokenBalance: 0,
      features: [],
      permissions: [],
      isAdmin: false,
    };
  }

  return {
    isLoggedIn: true,
    userEmail: email.value,
    role: role.value as UserRole,
    tokenBalance: tokenBalance ? parseInt(tokenBalance.value, 10) : 0,
    features: features ? (JSON.parse(features.value) as TokenFeature[]) : [],
    permissions: permissions
      ? (JSON.parse(permissions.value) as Permission[])
      : [],
    isAdmin: role.value === UserRole.ADMINISTRATOR,
  };
}

export async function signInWithOAuth(
  providerId: string, 
  redirectUrl?: string,
  oauthRedirectUri?: string
) {
  try {
    console.info('Initiating OAuth flow', { providerId, redirectUrl, oauthRedirectUri });
    
    const { url } = await apiClient.auth.googleInit({
      redirectUrl,
      oauthRedirectUri
    });
    console.info('Received OAuth init response:', { url });
    
    if (!url) {
      console.error('OAuth init response missing URL');
      throw new Error('OAuth initialization response missing redirect URL');
    }

    console.info('OAuth init successful, returning URL:', url);
    return { redirectUrl: url };
  } catch (error) {
    console.error('OAuth initialization error:', error);
    // Log additional error details if available
    if (error instanceof Error) {
      console.error('Error details:', {
        message: error.message,
        stack: error.stack
      });
    }
    throw error;
  }
}

export async function signUpWithEmailPassword({ email, password }: { email: string; password: string }) {
  if (!email || !password) {
    throw new Error('Email and password are required');
  }

  try {
    return await apiClient.auth.register({ email, password });
  } catch (error) {
    console.error('Registration error:', error);
    throw error;
  }
}

export async function signOut() {
  try {
    await apiClient.auth.logout();
    
    const cookieStore = await cookies();
    cookieStore.delete('__session');
    cookieStore.delete('email');
    cookieStore.delete('role');
    cookieStore.delete('token_balance');
    cookieStore.delete('features');
    cookieStore.delete('permissions');
    cookieStore.delete('oauth_state');

    redirect('/login');
  } catch (error) {
    console.error('Sign out error:', error);
    throw error;
  }
}

export async function handleOAuthCallback(code: string, state: string): Promise<AuthActionResponse> {
  if (!code || !state) {
    throw new Error('Invalid OAuth callback parameters');
  }

  try {
    // Verify state starts with valid provider
    if (!state.startsWith('google_') && !state.startsWith('github_')) {
      throw new Error('Invalid state parameter');
    }

    console.debug('OAuth callback request started with:', { code, state });
    
    const response = await apiClient.auth.googleCallback(code, state);
    console.debug('OAuth callback response:', { response });
    
    // Check cookies after callback
    const cookieStore = await cookies();
    // Log individual cookies
    console.debug('Cookies after OAuth callback:', {
      session: cookieStore.get('__session'),
      email: cookieStore.get('email'),
      role: cookieStore.get('role'),
      tokenBalance: cookieStore.get('token_balance'),
      features: cookieStore.get('features'),
      permissions: cookieStore.get('permissions'),
    });

    // Return the redirect URL to be handled by the client component
    if (response && 'redirect' in response) {
      return { redirect: response.redirect };
    }
    return null;
  } catch (error) {
    console.error('OAuth callback error:', error);
    throw error;
  }
}

export async function listUsers() {
  try {
    const users = await apiClient.auth.roles();
    return users.map(user => ({
      userId: user.uid,
      email: user.email,
      role: user.role as UserRole,
    }));
  } catch (error) {
    console.error('Error fetching users:', error);
    throw error;
  }
}

export async function signInWithEmail(formData: FormData): Promise<AuthActionResponse> {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;
  const redirectTo = (formData.get('redirectTo') as string) || '/home';

  if (!email || !password) {
    throw new Error('Email and password are required');
  }

  try {
    console.debug('Email login request started with:', { email });
    
    const response = await apiClient.auth.login({ email, password });
    console.debug('Email login response:', { response });
    
    // Check cookies after login
    const cookieStore = await cookies();
    // Log individual cookies
    console.debug('Cookies after login:', {
      session: cookieStore.get('__session'),
      email: cookieStore.get('email'),
      role: cookieStore.get('role'),
      tokenBalance: cookieStore.get('token_balance'),
      features: cookieStore.get('features'),
      permissions: cookieStore.get('permissions'),
    });

    if (response && 'redirect' in response) {
      return { redirect: response.redirect };
    }
    return null;
  } catch (error) {
    throw error;
  }
}
