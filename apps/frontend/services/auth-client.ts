/**
 * Client-side authentication service
 * This service now uses server actions directly instead of API routes
 */

import { login, logout, register, getCurrentUser } from '@epsx/server-actions';

interface LoginRequest {
  email: string;
  password: string;
}

interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
}

interface BackendUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
  display_name?: string;
  photo_url?: string;
  phone_number?: string | null;
}

/**
 * Client-side auth wrapper that uses server actions
 */
export const authClient = {
  async getCurrentUser(): Promise<BackendUser | null> {
    try {
      return await getCurrentUser();
    } catch (error) {
      console.error('Error getting current user:', error);
      return null;
    }
  },

  async login(data: LoginRequest): Promise<BackendUser> {
    try {
      const result = await login(data);
      if (!result) {
        throw new Error('Invalid credentials');
      }
      return result;
    } catch (error) {
      console.error('Error during login:', error);
      throw error;
    }
  },

  async register(data: RegisterRequest): Promise<{
    user_id: string;
    email: string;
    verification_sent: boolean;
    message: string;
  }> {
    try {
      return await register(data);
    } catch (error) {
      console.error('Error during registration:', error);
      throw error;
    }
  },

  async logout(): Promise<void> {
    try {
      await logout();
    } catch (error) {
      console.error('Error during logout:', error);
      throw error;
    }
  },
};