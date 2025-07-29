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
    console.log('[adminLoginAction] Backend response cookies:', cookieHeaders);
    console.log('[adminLoginAction] Backend response data:', { 
      hasSessionToken: !!data.session_token,
      hasUserId: !!data.user_id,
      hasRole: !!data.role 
    });
    
    const store = await cookies();
    
    if (cookieHeaders.length > 0) {
      for (const cookieHeader of cookieHeaders) {
        // Parse cookie string: "name=value; Path=/; HttpOnly; ..."
        const [cookiePair, ...attributes] = cookieHeader.split(';').map(s => s.trim());
        const [name, value] = cookiePair.split('=');
        
        console.log('[adminLoginAction] Processing cookie:', { name, value: value ? '***' : null, attributes });
        
        if (name && value) {
          // For admin session, use admin_sess_id
          if (name === 'sess_id') {
            console.log('[adminLoginAction] Setting admin_sess_id cookie');
            store.set('admin_sess_id', value, { 
              httpOnly: true, 
              secure: process.env.NODE_ENV === 'production',
              sameSite: 'lax',
              maxAge: 60 * 60 * 24 * 7 // 7 days
            });
          } else {
            console.log(`[adminLoginAction] Setting ${name} cookie`);
            store.set(name, value, { httpOnly: true });
          }
        }
      }
    } else {
      console.log('[adminLoginAction] No cookies received from backend, trying manual session token');
      
      // If no cookies from backend but we have session token in response, set it manually
      if (data.session_token) {
        console.log('[adminLoginAction] Setting admin_sess_id from response session_token');
        store.set('admin_sess_id', data.session_token, { 
          httpOnly: true, 
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 7 // 7 days
        });
      } else {
        console.log('[adminLoginAction] No session token in response data either');
      }
    }
    
    // Verify cookies were set
    const allCookies = store.getAll();
    console.log('[adminLoginAction] All cookies after login:', allCookies.map(c => ({ name: c.name, hasValue: !!c.value })));

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