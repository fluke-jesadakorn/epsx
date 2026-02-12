'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { logger } from '../utils/logger';
import type { UserInfoResponse } from './client';
import { COOKIES, COOKIE_OPTIONS, HTTP_ONLY_COOKIES } from './cookies';

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
        logger.info('[AUTH] loginAction called with:', {
            hasAccessToken: Boolean(accessToken),
            tokenLength: accessToken.length,
            hasUser: Boolean(user),
            hasRefreshToken: Boolean(refreshToken),
            cookieName: COOKIES.access_token,
            env: process.env.NODE_ENV,
        });

        if (accessToken === '') {
            logger.error('[AUTH] Error: loginAction: Missing access_token');
            return { success: false, error: 'Missing access_token' };
        }

        const cookieStore = await cookies();

        // 1. Set Access Token (HttpOnly)
        // Log equivalent set-cookie header params for debugging
        logger.info('[AUTH] Setting access_token cookie:', {
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
        cookieStore.set(COOKIES.user, JSON.stringify(user), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.user,
        });

        // 3. Set Refresh Token (HttpOnly) if available
        if (refreshToken !== undefined && refreshToken !== '') {
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

        logger.info('[AUTH] loginAction: All cookies set successfully', {
            accessToken: COOKIES.access_token,
            user: COOKIES.user,
            authTime: COOKIES.auth_time,
            expiresAt: COOKIES.expires_at,
        });

        return { success: true };
    } catch (error) {
        logger.error('Login action failed:', error instanceof Error ? error.message : String(error));
        return { success: false, error: 'Internal server error' };
    }
}

/**
 * Server Action to clear the user session.
 */
export async function logoutAction() {
    try {
        const cookieStore = await cookies();

        // Get wallet address before clearing cookies
        const userCookie = cookieStore.get(COOKIES.user)?.value;
        let walletAddress = '';
        if (userCookie !== undefined && userCookie !== '') {
            const user = JSON.parse(userCookie) as UserInfoResponse;
            walletAddress = user.wallet_address ?? '';
        }

        logger.info('[AUTH] logoutAction: Starting logout', { walletAddress: walletAddress.slice(0, 8) });

        // Call backend logout endpoint if we have wallet address
        if (walletAddress !== '') {
            try {
                const { getBackendUrl } = await import('../utils/url-resolver');
                const backendUrl = getBackendUrl('server');

                await fetch(`${backendUrl}/api/auth/web3/logout`, {
                    method: 'DELETE',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ wallet_address: walletAddress }),
                    cache: 'no-store',
                });

                logger.info('[AUTH] logoutAction: Backend logout called');
            } catch (e) {
                // Best effort - continue clearing cookies even if backend call fails
                logger.warn('[AUTH] logoutAction: Backend logout failed (continuing)',
                    e instanceof Error ? e.message : String(e));
            }
        }

        // Clear all known cookies
        Object.entries(COOKIES).forEach(([key, cookieName]) => {
            // Determine options based on cookie type using HTTP_ONLY_COOKIES
            const isHttpOnly = (HTTP_ONLY_COOKIES as readonly string[]).includes(key);
            const options = isHttpOnly ? COOKIE_OPTIONS.httpOnly : COOKIE_OPTIONS.clientSide;

            // Explicitly delete with path and domain to ensure removal
            // Next.js cookies().delete() matches based on these
            cookieStore.delete({
                name: cookieName,
                path: options.path,
                domain: options.domain,
            });
        });

        logger.info('[AUTH] logoutAction: All cookies cleared');
        return { success: true };
    } catch (error) {
        logger.error('Logout action failed:', error instanceof Error ? error.message : String(error));
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

        if (refreshToken === undefined || refreshToken === '') {
            logger.warn('[AUTH] refreshSessionAction: No refresh token found');
            return { success: false, error: 'No refresh token available' };
        }

        // Import dynamically to avoid bundling issues
        const { getBackendUrl } = await import('../utils/url-resolver');
        const backendUrl = getBackendUrl('server');

        // Determine client_id from environment
        const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID ?? 'epsx-frontend';

        // Call backend refresh endpoint directly
        const response = await fetch(`${backendUrl}/api/auth/session/refresh`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: refreshToken, client_id: clientId }),
            cache: 'no-store',
        });

        if (!response.ok) {
            logger.error('[AUTH] refreshSessionAction: Backend refresh failed', String(response.status));
            return { success: false, error: 'Token refresh failed' };
        }

        const data = (await response.json()) as {
            access_token?: string;
            expires_in?: number;
            refresh_token?: string;
            user?: Record<string, unknown>;
        };

        if (data.access_token === undefined || data.access_token === '') {
            logger.error('[AUTH] refreshSessionAction: No access_token in response');
            return { success: false, error: 'Invalid refresh response' };
        }

        updateSessionCookies(cookieStore, data);

        logger.info('[AUTH] refreshSessionAction: Token refreshed successfully');
        return { success: true, access_token: data.access_token };
    } catch (error) {
        logger.error('[AUTH] refreshSessionAction error:', error instanceof Error ? error.message : String(error));
        return { success: false, error: 'Internal server error' };
    }
}

/**
 * Internal helper to update session cookies after token refresh
 */
function updateSessionCookies(
    cookieStore: Awaited<ReturnType<typeof cookies>>,
    data: { access_token?: string; expires_in?: number; refresh_token?: string; user?: Record<string, unknown> }
) {
    if (data.access_token === undefined || data.access_token === '') { return; }

    const accessToken = data.access_token;

    // 1. Update Access Token
    cookieStore.set(COOKIES.access_token, accessToken, {
        ...COOKIE_OPTIONS.httpOnly,
        maxAge: data.expires_in ?? COOKIE_OPTIONS.maxAge.access_token,
    });

    // 2. Update Refresh Token if provided
    if (data.refresh_token !== undefined && data.refresh_token !== '') {
        cookieStore.set(COOKIES.refresh_token, data.refresh_token, {
            ...COOKIE_OPTIONS.httpOnly,
            maxAge: COOKIE_OPTIONS.maxAge.refresh_token,
        });
    }

    // 3. Update User Data
    const existingUserCookie = cookieStore.get(COOKIES.user)?.value;
    if (existingUserCookie !== undefined && existingUserCookie !== '') {
        const existingUser = JSON.parse(existingUserCookie) as Record<string, unknown>;
        existingUser.access = accessToken;
        if (data.user) { Object.assign(existingUser, data.user); }

        cookieStore.set(COOKIES.user, JSON.stringify(existingUser), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.user,
        });
    } else if (data.user) {
        const newUser = { ...data.user, access: accessToken };
        cookieStore.set(COOKIES.user, JSON.stringify(newUser), {
            ...COOKIE_OPTIONS.clientSide,
            maxAge: COOKIE_OPTIONS.maxAge.user,
        });
    }

    // 4. Update expires_at
    const expiresAt = Date.now() + ((data.expires_in ?? COOKIE_OPTIONS.maxAge.access_token) * 1000);
    cookieStore.set(COOKIES.expires_at, expiresAt.toString(), {
        ...COOKIE_OPTIONS.clientSide,
        maxAge: COOKIE_OPTIONS.maxAge.expires_at,
    });
}

