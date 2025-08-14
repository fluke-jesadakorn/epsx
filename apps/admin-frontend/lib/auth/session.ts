/**
 * Manual Session Cookie Implementation
 * Direct cookie encryption/decryption for session data
 */
import crypto from 'crypto';
import { NextResponse, NextRequest } from 'next/server';
import { cookies } from 'next/headers';

// Define SessionData interface directly to avoid circular imports
export interface SessionData {
  user?: {
    id: string;
    email: string;
    name: string;
    role: string;
    permissions: string[];
    admin_modules: string[];
    firebase_uid: string;
    hasAdminModule?: (module: string) => boolean;
    isSystemAdmin?: () => boolean;
  };
  accessToken?: string;
  refreshToken?: string;
  isLoggedIn: boolean;
  expiresAt?: number;
}

const SESSION_SECRET = process.env.SESSION_SECRET || process.env.NEXTAUTH_SECRET || 'complex-password-at-least-32-characters-long-for-iron-session-security';
const COOKIE_NAME = 'epsx-admin-session';

/**
 * Encrypt session data for cookie storage
 */
function encryptSessionData(data: SessionData): string {
  try {
    const jsonData = JSON.stringify(data);
    const key = crypto.scryptSync(SESSION_SECRET, 'salt', 32);
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
    
    let encrypted = cipher.update(jsonData, 'utf8', 'hex');
    encrypted += cipher.final('hex');
    
    // Combine IV and encrypted data
    const combined = iv.toString('hex') + ':' + encrypted;
    return Buffer.from(combined).toString('base64');
  } catch (error) {
    console.error('❌ Session encryption error:', error);
    throw new Error('Failed to encrypt session data');
  }
}

/**
 * Decrypt session data from cookie
 */
function decryptSessionData(encryptedData: string): SessionData | null {
  try {
    const combined = Buffer.from(encryptedData, 'base64').toString('utf8');
    const [ivHex, encrypted] = combined.split(':');
    
    if (!ivHex || !encrypted) {
      return null;
    }
    
    const key = crypto.scryptSync(SESSION_SECRET, 'salt', 32);
    const iv = Buffer.from(ivHex, 'hex');
    const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
    
    let decrypted = decipher.update(encrypted, 'hex', 'utf8');
    decrypted += decipher.final('utf8');
    
    return JSON.parse(decrypted);
  } catch (error) {
    console.error('❌ Session decryption error:', error);
    return null;
  }
}

/**
 * Set session cookie manually in response
 */
export function setSessionCookie(response: NextResponse, sessionData: SessionData): void {
  try {
    console.log('🔧 Setting session cookie manually');
    console.log('🔧 Session data to encrypt:', {
      isLoggedIn: sessionData.isLoggedIn,
      userEmail: sessionData.user?.email,
      userId: sessionData.user?.id,
    });
    
    const encryptedData = encryptSessionData(sessionData);
    console.log('🔧 Encrypted session data length:', encryptedData.length);
    
    response.cookies.set(COOKIE_NAME, encryptedData, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 7, // 7 days
      path: '/',
    });
    
    console.log('✅ Session cookie set manually with name:', COOKIE_NAME);
  } catch (error) {
    console.error('❌ Failed to set session cookie:', error);
    throw error;
  }
}

/**
 * Get session data from cookie
 */
export function getSessionFromCookie(cookieValue: string | undefined): SessionData {
  if (!cookieValue) {
    console.log('🔧 No session cookie found');
    return { isLoggedIn: false };
  }
  
  try {
    console.log('🔧 Processing session cookie, length:', cookieValue.length);
    
    // Handle URL encoding - Next.js might encode the cookie value
    let decodedValue = cookieValue;
    if (cookieValue.includes('%')) {
      console.log('🔧 Cookie appears to be URL encoded, decoding...');
      try {
        decodedValue = decodeURIComponent(cookieValue);
        console.log('🔧 Decoded cookie length:', decodedValue.length);
      } catch (decodeError) {
        console.log('🔧 URL decode failed, using raw value');
        decodedValue = cookieValue;
      }
    }
    
    console.log('🔧 Decrypting session cookie...');
    const sessionData = decryptSessionData(decodedValue);
    
    if (!sessionData) {
      console.log('🔧 Session cookie decryption failed');
      return { isLoggedIn: false };
    }
    
    console.log('🔧 Session decrypted successfully:', {
      isLoggedIn: sessionData.isLoggedIn,
      userEmail: sessionData.user?.email,
    });
    
    // Add convenience methods to user object if present
    if (sessionData.user) {
      sessionData.user.hasAdminModule = (module: string) => 
        sessionData.user?.admin_modules?.includes(module) || false;
      sessionData.user.isSystemAdmin = () => 
        sessionData.user?.admin_modules?.includes('system_admin') || false;
    }
    
    return sessionData;
  } catch (error) {
    console.error('❌ Session cookie parsing error:', error);
    return { isLoggedIn: false };
  }
}

/**
 * Get session from Next.js server-side cookies (for server components)
 */
export async function getSession(): Promise<SessionData> {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get(COOKIE_NAME);
    return getSessionFromCookie(sessionCookie?.value);
  } catch (error) {
    console.error('❌ Failed to get session from server:', error);
    return { isLoggedIn: false };
  }
}

/**
 * Get session from NextRequest (for middleware/API routes)
 */
export async function getSessionFromRequest(request: NextRequest): Promise<SessionData> {
  try {
    const sessionCookie = request.cookies.get(COOKIE_NAME);
    return getSessionFromCookie(sessionCookie?.value);
  } catch (error) {
    console.error('❌ Failed to get session from request:', error);
    return { isLoggedIn: false };
  }
}

/**
 * Refresh session if needed (placeholder for token refresh logic)
 */
export async function refreshSessionIfNeeded(session: SessionData): Promise<SessionData> {
  // For now, just return the session as-is
  // In the future, implement token refresh logic here
  return session;
}

/**
 * Remove session cookie from response
 */
export function removeSessionCookie(response: NextResponse): void {
  try {
    console.log('🔧 Removing session cookie');
    
    response.cookies.set(COOKIE_NAME, '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0, // Expire immediately
      path: '/',
    });
    
    console.log('✅ Session cookie removed with name:', COOKIE_NAME);
  } catch (error) {
    console.error('❌ Failed to remove session cookie:', error);
    throw error;
  }
}

/**
 * Create a user session from userinfo and tokens
 */
export function createUserSession(userinfo: any, accessToken: string, refreshToken?: string): SessionData {
  return {
    user: {
      id: userinfo.sub || userinfo.id,
      email: userinfo.email,
      name: userinfo.name || userinfo.display_name,
      role: userinfo.role || 'user',
      permissions: userinfo.permissions || [],
      admin_modules: userinfo.admin_modules || [],
      firebase_uid: userinfo.firebase_uid || userinfo.sub,
      hasAdminModule: (module: string) => 
        userinfo.admin_modules?.includes(module) || false,
      isSystemAdmin: () => 
        userinfo.admin_modules?.includes('system_admin') || false,
    },
    accessToken,
    refreshToken,
    isLoggedIn: true,
    expiresAt: userinfo.exp || Date.now() + (60 * 60 * 1000), // 1 hour default
  };
}