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
    const res = await fetch(`${URL}/api/v1/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        type: "admin",
        email,
        password,
        admin_token: null
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
    const cookieHeaders = res.headers.getSetCookie?.() || [];
    if (cookieHeaders.length > 0) {
      const store = await cookies();
      
      for (const cookieHeader of cookieHeaders) {
        // Parse cookie string: "name=value; Path=/; HttpOnly; ..."
        const [cookiePair, ...attributes] = cookieHeader.split(';').map(s => s.trim());
        const [name, value] = cookiePair.split('=');
        
        if (name && value) {
          // For admin session, use admin_sess_id
          if (name === 'sess_id') {
            store.set('admin_sess_id', value, { 
              httpOnly: true, 
              secure: process.env.NODE_ENV === 'production',
              sameSite: 'lax',
              maxAge: 60 * 60 * 24 * 7 // 7 days
            });
          } else {
            store.set(name, value, { httpOnly: true });
          }
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
    const store = await cookies();
    const header = store.toString();

    await fetch(`${URL}/api/v1/admin/logout`, {
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