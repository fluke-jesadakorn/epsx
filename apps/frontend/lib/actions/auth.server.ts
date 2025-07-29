'use server';

import { redirect } from 'next/navigation';
import { createApiClient, isApiError } from '@epsx/api-client';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const apiClient = createApiClient(BACKEND_URL);

export async function loginAction(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  try {
    const response = await apiClient.login({
      type: 'credentials',
      email,
      password,
    });

    if (isApiError(response)) {
      return { success: false, error: response.error || 'Login failed' };
    }

    return { success: true, user: response.data };
  } catch (error) {
    return { success: false, error: 'Login failed' };
  }
}

export async function registerAction(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  try {
    const response = await apiClient.register({
      email,
      password,
    });

    if (isApiError(response)) {
      return { success: false, error: response.error || 'Registration failed' };
    }

    return { success: true, user: response.data };
  } catch (error) {
    return { success: false, error: error instanceof Error ? error.message : 'Registration failed' };
  }
}

export async function logoutAction() {
  try {
    const response = await apiClient.logout();

    if (isApiError(response)) {
      return { success: false, error: response.error || 'Logout failed' };
    }

    redirect('/login');
  } catch (error) {
    return { success: false, error: 'Logout failed' };
  }
}

export async function clearSessionAction() {
  try {
    const response = await apiClient.logout();

    if (isApiError(response)) {
      return { success: false, error: response.error || 'Failed to clear session' };
    }

    return { success: true };
  } catch (error) {
    return { success: false, error: 'Failed to clear session' };
  }
}