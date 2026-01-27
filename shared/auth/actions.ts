'use server';

import { cookies } from 'next/headers';
import { COOKIES, COOKIE_OPTIONS, HTTP_ONLY_COOKIES } from './cookies';
import type { UserInfoResponse } from './client';

/**
 * Server Action to establish a user session by setting cookies.
 * This MUST be called from a Client Component or another Server Action.
 */
export async function loginAction(
    accessToken: string,
    user: UserInfoResponse | Record<string, unknown>,
    refreshToken?: string
) {
    try {
        console.log('[AUTH] loginAction called with:', {
            hasAccessToken: !!accessToken,
            tokenLength: accessToken?.length,
            hasUser: !!user,
            hasRefreshToken: !!refreshToken,
            cookieName: COOKIES.access_token,
            env: process.env.NODE_ENV,
        });

        if (!accessToken) {
            console.error('[AUTH] Error: loginAction: Missing access_token');
            return { success: false, error: 'Missing access_token' };
        }

        const cookieStore = await cookies();

        // 1. Set Access Token (HttpOnly)
        // Log equivalent set-cookie header params for debugging
        console.log('[AUTH] Setting access_token cookie:', {
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

        console.log('[AUTH] loginAction: All cookies set successfully', {
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

        console.log('[AUTH] logoutAction: Clearing session cookies...');

        // Clear all known cookies
        Object.entries(COOKIES).forEach(([key, cookieName]) => {
            // Determine options based on cookie type using HTTP_ONLY_COOKIES
            const isHttpOnly = HTTP_ONLY_COOKIES.includes(key as any);
            const options = isHttpOnly ? COOKIE_OPTIONS.httpOnly : COOKIE_OPTIONS.clientSide;

            // Explicitly delete with path and domain to ensure removal
            // Next.js cookies().delete() matches based on these
            cookieStore.delete({
                name: cookieName,
                path: options.path,
                domain: options.domain,
            });
        });

        console.log('[AUTH] logoutAction: All cookies cleared');
        return { success: true };
    } catch (error) {
        console.error('Logout action failed:', error);
        return { success: false, error: 'Internal server error' };
    }
}

/**
 * Server Action to refresh the access token using the refresh token.
 * This eliminates the need for the /api/proxy route for token refresh.
 */
export async function refreshSessionAction() {
    try {
        const cookieStore = await cookies();
        const refreshToken = cookieStore.get(COOKIES.refresh_token)?.value;

        if (!refreshToken) {
            console.warn('[AUTH] refreshSessionAction: No refresh token found');
            return { success: false, error: 'No refresh token available' };
        }

        // Import dynamically to avoid bundling issues
        const { getBackendUrl } = await import('../utils/url-resolver');
        const backendUrl = getBackendUrl('server');

        // Call backend refresh endpoint directly
        const response = await fetch(`${backendUrl}/api/auth/session/refresh`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: refreshToken }),
            cache: 'no-store',
        });

        if (!response.ok) {
            console.error('[AUTH] refreshSessionAction: Backend refresh failed', response.status);
            return { success: false, error: 'Token refresh failed' };
        }

        const data = await response.json();

        if (!data.access_token) {
            console.error('[AUTH] refreshSessionAction: No access_token in response');
            return { success: false, error: 'Invalid refresh response' };
        }

        // Update cookies with new tokens
        cookieStore.set(COOKIES.access_token, data.access_token, {
            ...COOKIE_OPTIONS.httpOnly,
            maxAge: data.expires_in || COOKIE_OPTIONS.maxAge.access_token,
        });

        if (data.refresh_token) {
            cookieStore.set(COOKIES.refresh_token, data.refresh_token, {
                ...COOKIE_OPTIONS.httpOnly,
                maxAge: COOKIE_OPTIONS.maxAge.refresh_token,
            });
        }

        // Update user data if provided
        if (data.user) {
            const existingUserCookie = cookieStore.get(COOKIES.user)?.value;
            let existingUser = existingUserCookie ? JSON.parse(existingUserCookie) : {};
            cookieStore.set(COOKIES.user, JSON.stringify({ ...existingUser, ...data.user }), {
                ...COOKIE_OPTIONS.clientSide,
                maxAge: COOKIE_OPTIONS.maxAge.user,
            });
        }

        // Update expires_at
        const expiresAt = Date.now() + ((data.expires_in || COOKIE_OPTIONS.maxAge.access_token) * 1000);
        cookieStore.set(COOKIES.expires_at, expiresAt.toString(), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.expires_at,
        });

        console.log('[AUTH] refreshSessionAction: Token refreshed successfully');
        return { success: true, access_token: data.access_token };
    } catch (error) {
        console.error('[AUTH] refreshSessionAction error:', error);
        return { success: false, error: 'Internal server error' };
    }
}
