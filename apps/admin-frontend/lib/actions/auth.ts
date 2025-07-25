'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { adminLogger } from '../logger';
import { config } from '../config';

const BACKEND_URL = config.getBackendUrl();

export async function adminLoginAction(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  try {
    const response = await fetch(`${BACKEND_URL}/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        type: "admin",
        email,
        password,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: 'Login failed' }));
      return { 
        success: false, 
        error: errorData.message || 'Invalid credentials' 
      };
    }

    const userData = await response.json();
    
    // Set cookies from backend response
    const setCookieHeader = response.headers.get('set-cookie');
    if (setCookieHeader) {
      const cookieStore = cookies();
      const cookiePairs = setCookieHeader.split(';');
      
      for (const cookiePair of cookiePairs) {
        const [name, value] = cookiePair.trim().split('=');
        if (name && value) {
          cookieStore.set(name, value, { httpOnly: true });
        }
      }
    }

    const user = {
      uid: userData.user_id,
      email: userData.email,
      roles: [userData.role],
      isAdmin: userData.role === 'admin' || userData.role === 'super_admin',
      customClaims: {
        role: userData.role.toUpperCase(),
      },
    };

    return { success: true, user };
  } catch (error) {
    adminLogger.error('Admin login error', { 
      error: error instanceof Error ? error.message : String(error) 
    });
    return { success: false, error: 'Internal server error' };
  }
}

export async function adminLogoutAction() {
  try {
    const cookieStore = cookies();
    const cookieHeader = cookieStore.toString();

    await fetch(`${BACKEND_URL}/admin/logout`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
    });

    // Clear all cookies
    cookieStore.getAll().forEach(cookie => {
      cookieStore.delete(cookie.name);
    });

    redirect('/admin/login');
  } catch (error) {
    adminLogger.error('Admin logout error', { 
      error: error instanceof Error ? error.message : String(error) 
    });
    return { success: false, error: 'Logout failed' };
  }
}