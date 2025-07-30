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
    
    // Get cookies from backend response headers
    const cookieHeaders = res.headers.getSetCookie?.() || [];
    console.log('[adminLoginAction] Backend response cookies:', cookieHeaders);
    console.log('[adminLoginAction] Backend response data:', { 
      hasSessionToken: !!data.session_token,
      hasUserId: !!data.user_id,
      hasRole: !!data.role 
    });
    
    // Handle cookies properly in server action context
    let sessionId: string | null = null;
    
    if (cookieHeaders.length > 0) {
      for (const cookieHeader of cookieHeaders) {
        // Parse cookie string: "name=value; Path=/; HttpOnly; ..."
        const [cookiePair] = cookieHeader.split(';').map(s => s.trim());
        const [name, value] = cookiePair.split('=');
        
        console.log('[adminLoginAction] Processing cookie:', { name, value: value ? '***' : null });
        
        if (name === 'sess_id' && value) {
          sessionId = value;
          console.log('[adminLoginAction] Found sess_id cookie');
        }
      }
    }
    
    // Set the session cookie properly using Next.js cookies API
    // Use 'sess_id' to match backend expectations for unified session handling
    if (sessionId) {
      const cookieStore = await cookies();
      cookieStore.set('sess_id', sessionId, { 
        httpOnly: true, 
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 7, // 7 days
        path: '/'
      });
      console.log('[adminLoginAction] Set sess_id cookie successfully');
    } else {
      console.log('[adminLoginAction] No session ID found in backend response');
      return { success: false, error: 'No session created' };
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

    console.log('[adminLoginAction] Login successful, redirecting to dashboard');
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

    await fetch(`${URL}/api/admin/auth/logout`, {
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