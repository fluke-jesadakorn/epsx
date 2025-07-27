'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { adminLogger } from '../logger';
import { config } from '../config';

const URL = config.getBackendUrl();

export async function adminLoginAction(form: FormData) {
  const email = form.get('email') as string;
  const password = form.get('password') as string;

  try {
    const res = await fetch(`${URL}/login`, {
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

    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Login failed' }));
      return { 
        success: false, 
        error: err.message || 'Invalid credentials' 
      };
    }

    const data = await res.json();
    
    // Set cookies from backend response
    const cookieHeader = res.headers.get('set-cookie');
    if (cookieHeader) {
      const store = cookies();
      const pairs = cookieHeader.split(';');
      
      for (const pair of pairs) {
        const [name, value] = pair.trim().split('=');
        if (name && value) {
          store.set(name, value, { httpOnly: true });
        }
      }
    }

    const user = {
      uid: data.user_id,
      email: data.email,
      roles: [data.role],
      isAdmin: data.role === 'admin' || data.role === 'super_admin' || data.role === 'SuperAdmin',
      customClaims: {
        role: data.role.toUpperCase(),
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
    const store = cookies();
    const header = store.toString();

    await fetch(`${URL}/admin/logout`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Cookie': header,
      },
    });

    // Clear all cookies
    store.getAll().forEach(cookie => {
      store.delete(cookie.name);
    });

    redirect('/admin/login');
  } catch (error) {
    adminLogger.error('Admin logout error', { 
      error: error instanceof Error ? error.message : String(error) 
    });
    return { success: false, error: 'Logout failed' };
  }
}