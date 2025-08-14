/**
 * Manual Session Cookie Implementation
 * Direct cookie encryption/decryption for session data using Web Crypto API
 */
import { NextResponse } from 'next/server';

// Define SessionData interface directly to avoid circular imports
export interface SessionData {
  user?: {
    id: string;
    email: string;
    name: string;
    role: string;
    permissions: string[];
    package_tier: string;
    firebase_uid: string;
    hasPermission?: (permission: string) => boolean;
    hasPackageTier?: (tier: string) => boolean;
  };
  accessToken?: string;
  refreshToken?: string;
  isLoggedIn: boolean;
}

const SESSION_SECRET = process.env.SESSION_SECRET || process.env.NEXTAUTH_SECRET || 'complex-password-at-least-32-characters-long-for-iron-session-security';
const COOKIE_NAME = 'epsx-frontend-session';

/**
 * Generate key from password using PBKDF2 (Web Crypto API)
 */
async function deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const encoder = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    { name: 'PBKDF2' },
    false,
    ['deriveKey']
  );
  
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: salt,
      iterations: 100000,
      hash: 'SHA-256'
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  );
}

/**
 * Encrypt session data for cookie storage using Web Crypto API
 */
async function encryptSessionData(data: SessionData): Promise<string> {
  try {
    const jsonData = JSON.stringify(data);
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(jsonData);
    
    // Generate random salt and IV
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const iv = crypto.getRandomValues(new Uint8Array(12));
    
    // Derive key
    const key = await deriveKey(SESSION_SECRET, salt);
    
    // Encrypt data
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: iv },
      key,
      dataBuffer
    );
    
    // Combine salt, IV, and encrypted data
    const combined = new Uint8Array(salt.length + iv.length + encrypted.byteLength);
    combined.set(salt, 0);
    combined.set(iv, salt.length);
    combined.set(new Uint8Array(encrypted), salt.length + iv.length);
    
    // Convert to base64 using Array.from to handle the iteration properly
    const base64 = btoa(String.fromCharCode(...Array.from(combined)));
    return base64;
  } catch (error) {
    console.error('❌ Session encryption error:', error);
    throw new Error('Failed to encrypt session data');
  }
}

/**
 * Decrypt session data from cookie using Web Crypto API
 */
async function decryptSessionData(encryptedData: string): Promise<SessionData | null> {
  try {
    // Decode from base64
    const combined = new Uint8Array(
      atob(encryptedData)
        .split('')
        .map(char => char.charCodeAt(0))
    );
    
    // Extract salt, IV, and encrypted data
    const salt = combined.slice(0, 16);
    const iv = combined.slice(16, 28);
    const encrypted = combined.slice(28);
    
    // Derive key
    const key = await deriveKey(SESSION_SECRET, salt);
    
    // Decrypt data
    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: iv },
      key,
      encrypted
    );
    
    // Convert back to string and parse JSON
    const decoder = new TextDecoder();
    const jsonData = decoder.decode(decrypted);
    
    return JSON.parse(jsonData);
  } catch (error) {
    console.error('❌ Session decryption error:', error);
    return null;
  }
}

/**
 * Set session cookie manually in response
 */
export async function setSessionCookie(response: NextResponse, sessionData: SessionData): Promise<void> {
  try {
    console.log('🔧 Setting session cookie manually');
    console.log('🔧 Session data to encrypt:', {
      isLoggedIn: sessionData.isLoggedIn,
      userEmail: sessionData.user?.email,
      userId: sessionData.user?.id,
    });
    
    const encryptedData = await encryptSessionData(sessionData);
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
export async function getSessionFromCookie(cookieValue: string | undefined): Promise<SessionData> {
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
    const sessionData = await decryptSessionData(decodedValue);
    
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
      sessionData.user.hasPermission = (permission: string) => 
        sessionData.user?.permissions?.includes(permission) || false;
      sessionData.user.hasPackageTier = (tier: string) => 
        sessionData.user?.package_tier === tier || false;
    }
    
    return sessionData;
  } catch (error) {
    console.error('❌ Session cookie parsing error:', error);
    return { isLoggedIn: false };
  }
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
      package_tier: userinfo.package_tier || 'basic',
      firebase_uid: userinfo.firebase_uid || userinfo.sub,
      hasPermission: (permission: string) => 
        userinfo.permissions?.includes(permission) || false,
      hasPackageTier: (tier: string) => 
        userinfo.package_tier === tier || false,
    },
    accessToken,
    refreshToken,
    isLoggedIn: true,
  };
}