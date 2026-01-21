'use server';

import { cookies } from 'next/headers';
import { COOKIES, COOKIE_OPTIONS } from './cookies';

/**
 * Server Action to establish a user session by setting cookies.
 * This MUST be called from a Client Component or another Server Action.
 */
export async function loginAction(
    accessToken: string,
    user: any,
    refreshToken?: string
) {
    try {
        console.log('🔐 loginAction called with:', {
            hasAccessToken: !!accessToken,
            tokenLength: accessToken?.length,
            hasUser: !!user,
            hasRefreshToken: !!refreshToken,
            cookieName: COOKIES.access_token,
            env: process.env.NODE_ENV,
        });

        if (!accessToken) {
            console.error('❌ loginAction: Missing access_token');
            return { success: false, error: 'Missing access_token' };
        }

        const cookieStore = await cookies();

        // 1. Set Access Token (HttpOnly)
        // Log equivalent set-cookie header params for debugging
        console.log('📝 Setting access_token cookie:', {
            name: COOKIES.access_token,
            secure: COOKIE_OPTIONS.httpOnly.secure,
            sameSite: COOKIE_OPTIONS.httpOnly.sameSite,
            domain: COOKIE_OPTIONS.httpOnly.domain,
            path: COOKIE_OPTIONS.httpOnly.path,
        });

        cookieStore.set(COOKIES.access_token, accessToken, {
            ...COOKIE_OPTIONS.httpOnly,
            maxAge: COOKIE_OPTIONS.maxAge.access_token,
        });

        // 2. Set User Data (Client Accessible - but set from server)
        if (user) {
            cookieStore.set(COOKIES.user, JSON.stringify(user), {
                ...COOKIE_OPTIONS.clientSide,
                maxAge: COOKIE_OPTIONS.maxAge.user,
            });
        }

        // 3. Set Refresh Token (HttpOnly) if available
        if (refreshToken) {
            cookieStore.set(COOKIES.refresh_token, refreshToken, {
                ...COOKIE_OPTIONS.httpOnly,
                maxAge: COOKIE_OPTIONS.maxAge.refresh_token,
            });
        }

        // 4. Set Authentication Timestamp
        cookieStore.set(COOKIES.auth_time, Date.now().toString(), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.auth_time,
        });

        // 5. Set expires_at timestamp
        const expiresAt = Date.now() + (COOKIE_OPTIONS.maxAge.access_token * 1000);
        cookieStore.set(COOKIES.expires_at, expiresAt.toString(), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.expires_at,
        });

        console.log('✅ loginAction: All cookies set successfully', {
            accessToken: COOKIES.access_token,
            user: COOKIES.user,
            authTime: COOKIES.auth_time,
            expiresAt: COOKIES.expires_at,
        });

        return { success: true };
    } catch (error) {
        console.error('Login action failed:', error);
        return { success: false, error: 'Internal server error' };
    }
}

/**
 * Server Action to clear the user session.
 */
export async function logoutAction() {
    try {
        const cookieStore = await cookies();

        console.log('🚪 logoutAction: Clearing session cookies...');

        // Clear all known cookies
        Object.entries(COOKIES).forEach(([key, cookieName]) => {
            // Determine options based on cookie type
            // Check if this key corresponds to an HttpOnly cookie
            const isHttpOnly = ['access_token', 'refresh_token', 'id_token', 'state', 'nonce'].includes(key);
            const options = isHttpOnly ? COOKIE_OPTIONS.httpOnly : COOKIE_OPTIONS.clientSide;

            // Explicitly delete with path and domain to ensure removal
            // Next.js cookies().delete() matches based on these
            cookieStore.delete({
                name: cookieName,
                path: options.path,
                domain: options.domain,
            });
        });

        console.log('✅ logoutAction: All cookies cleared');
        return { success: true };
    } catch (error) {
        console.error('Logout action failed:', error);
        return { success: false, error: 'Internal server error' };
    }
}
