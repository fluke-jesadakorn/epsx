'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { UserRole } from '@/types/auth/roles';
import { TokenFeature, Permission } from '@/types/auth/features';
import { apiClient } from '@/lib/api-client';

interface AuthResponse {
  token: string;
  email: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  redirectUrl?: string;
}

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

export async function signInWithOAuth(providerId: string) {
  try {
    console.info('Initiating OAuth flow', { providerId });
    
    const { url } = await apiClient.auth.googleInit();
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

export async function handleOAuthCallback(params: URLSearchParams) {
  const code = params.get('code');
  const state = params.get('state');

  if (!code || !state) {
    throw new Error('Invalid OAuth callback parameters');
  }

  try {
    // Verify state starts with valid provider
    if (!state.startsWith('google_') && !state.startsWith('github_')) {
      throw new Error('Invalid state parameter');
    }

    const data: AuthResponse = await apiClient.auth.googleCallback(code, state);
    
    // Validate required fields in response
    if (!data.token || !data.email) {
      console.error('Invalid auth response:', data);
      throw new Error('Invalid authentication response');
    }

    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/',
      maxAge: 60 * 60 * 24 * 7, // 1 week
    };

    const cookieStore = await cookies();
    cookieStore.set('__session', data.token, cookieOptions);
    cookieStore.set('email', data.email, cookieOptions);
    cookieStore.set('role', data.role || UserRole.REGISTERED_USER, cookieOptions);
    cookieStore.set(
      'token_balance',
      (data.tokenBalance || 0).toString(),
      cookieOptions
    );
    cookieStore.set(
      'features', 
      JSON.stringify(data.features || []),
      cookieOptions
    );
    cookieStore.set(
      'permissions',
      JSON.stringify(data.permissions || []),
      cookieOptions
    );

    return data;
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

export async function signInWithEmail(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;
  const redirectTo = (formData.get('redirectTo') as string) || '/home';

  if (!email || !password) {
    throw new Error('Email and password are required');
  }

  try {
    const data: AuthResponse = await apiClient.auth.login({ email, password });

    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/',
      maxAge: 60 * 60 * 24 * 7, // 1 week
    };

    const cookieStore = await cookies();
    cookieStore.set('__session', data.token, cookieOptions);
    cookieStore.set('email', data.email, cookieOptions);
    cookieStore.set('role', data.role, cookieOptions);
    cookieStore.set(
      'token_balance',
      data.tokenBalance.toString(),
      cookieOptions
    );
    cookieStore.set('features', JSON.stringify(data.features), cookieOptions);
    cookieStore.set(
      'permissions',
      JSON.stringify(data.permissions),
      cookieOptions
    );

    redirect(redirectTo);
  } catch (error) {
    throw error;
  }
}
