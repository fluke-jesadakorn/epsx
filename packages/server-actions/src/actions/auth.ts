'use server';

import { serverGet, serverPost } from '../core/request';

// Authentication Actions
export async function login(credentials: {
  email: string;
  password: string;
}) {
  try {
    return await serverPost('/api/auth/login', credentials);
  } catch (error) {
    console.error('Error during login:', error);
    throw error;
  }
}

export async function logout() {
  try {
    return await serverPost('/api/auth/logout');
  } catch (error) {
    console.error('Error during logout:', error);
    throw error;
  }
}

export async function getCurrentUser() {
  try {
    return await serverGet('/api/auth/me');
  } catch (error) {
    console.error('Error fetching current user:', error);
    return null;
  }
}

export async function refreshToken() {
  try {
    return await serverPost('/api/auth/refresh');
  } catch (error) {
    console.error('Error refreshing token:', error);
    throw error;
  }
}

// Admin Authentication
export async function adminLogin(credentials: {
  email: string;
  password: string;
}) {
  try {
    return await serverPost('/api/auth/admin/login', credentials);
  } catch (error) {
    console.error('Error during admin login:', error);
    throw error;
  }
}

export async function checkAdminPermission(permission: string) {
  try {
    return await serverGet('/api/auth/admin/check-permission', { permission });
  } catch (error) {
    console.error('Error checking admin permission:', error);
    return { allowed: false };
  }
}

export async function getAdminSession() {
  try {
    return await serverGet('/api/auth/admin/session');
  } catch (error) {
    console.error('Error fetching admin session:', error);
    return null;
  }
}

// User Profile Actions
export async function updateProfile(data: {
  name?: string;
  email?: string;
  preferences?: any;
}) {
  try {
    return await serverPost('/api/auth/profile/update', data);
  } catch (error) {
    console.error('Error updating profile:', error);
    throw error;
  }
}

export async function changePassword(data: {
  currentPassword: string;
  newPassword: string;
}) {
  try {
    return await serverPost('/api/auth/password/change', data);
  } catch (error) {
    console.error('Error changing password:', error);
    throw error;
  }
}

export async function register(data: {
  email: string;
  password: string;
  name?: string;
}) {
  try {
    return await serverPost('/api/auth/register', data);
  } catch (error) {
    console.error('Error during registration:', error);
    throw error;
  }
}

// Feature Access
export async function checkFeatureAccess(featureId: string) {
  try {
    return await serverGet('/api/auth/features/check', { featureId });
  } catch (error) {
    console.error('Error checking feature access:', error);
    return { allowed: false };
  }
}

export async function getUserFeatures() {
  try {
    return await serverGet('/api/auth/features');
  } catch (error) {
    console.error('Error fetching user features:', error);
    return [];
  }
}

// Password Reset Actions
export async function requestPasswordReset(email: string) {
  try {
    return await serverPost('/api/auth/password/reset/request', { email });
  } catch (error) {
    console.error('Error requesting password reset:', error);
    throw error;
  }
}

export async function resetPassword(data: {
  token: string;
  newPassword: string;
}) {
  try {
    return await serverPost('/api/auth/password/reset/confirm', data);
  } catch (error) {
    console.error('Error resetting password:', error);
    throw error;
  }
}