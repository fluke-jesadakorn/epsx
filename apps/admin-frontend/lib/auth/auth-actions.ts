'use server';

import { logger } from '@/lib/logger';
import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';

import type { Web3SessionData } from './auth';

/**
 * Server Action to set Web3 session data in secure cookies
 */
export async function setWeb3SessionAction(sessionData: Web3SessionData): Promise<void> {
    const cookieStore = await cookies();

    const cookieOptions = {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        maxAge: Math.floor((sessionData.expiresAt - Date.now()) / 1000), // Convert to seconds
        path: '/'
    };

    try {
        cookieStore.set(COOKIES.user, JSON.stringify({
            wallet_address: sessionData.walletAddress,
            sub: sessionData.walletAddress,
            auth_time: Date.now(),
            permissions: [],
            groups: ['admin'],
            isAdmin: true,
            expires_at: sessionData.expiresAt
        }), cookieOptions);
    } catch (error) {
        logger.auth.error('Failed to set Web3 session cookie', { error });
        throw error; // Re-throw for set as it's critical
    }
}

/**
 * Server Action to clear all Web3 session data
 */
export async function clearWeb3SessionAction(): Promise<void> {
    const cookieStore = await cookies();

    // Clear unified EPSX session cookies
    try {
        cookieStore.delete(COOKIES.user);
        cookieStore.delete(COOKIES.access_token);
        cookieStore.delete(COOKIES.id_token);
        cookieStore.delete(COOKIES.refresh_token);
        cookieStore.delete(COOKIES.sid);
    } catch (error) {
        // Find a way to handle this gracefully - usually happens if called during render
        logger.warn('Failed to clear cookies in clearWeb3SessionAction (likely called during render)', { error });
    }

}

/**
 * Server Action for logout
 */
export async function logoutAction(returnUrl?: string): Promise<void> {
    await clearWeb3SessionAction();

    if (returnUrl != null && returnUrl !== '') {
        const cookieStore = await cookies();
        cookieStore.set(COOKIES.return_url, returnUrl, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            path: '/',
            maxAge: 300,
        });
    }
}

/**
 * Server Action to retrieve and clear the return URL
 */
export async function getAndClearReturnUrlAction(): Promise<string> {
    const cookieStore = await cookies();
    const returnUrl = cookieStore.get(COOKIES.return_url)?.value ?? '/';

    // Clear the cookie after retrieving it
    if (cookieStore.has(COOKIES.return_url)) {
        cookieStore.delete(COOKIES.return_url);
    }

    // Validate return URL - reject invalid paths
    const invalidPrefixes = [
        '/.well-known',
        '/_next',
        '/api',
        '/favicon',
        '/static',
        '/access-denied',
        '/auth',
        '/login',
        '/unauthorized',
    ];

    // Check for invalid paths or external URLs
    const isInvalidPath = invalidPrefixes.some(prefix => returnUrl.startsWith(prefix));
    const isExternalUrl = returnUrl.startsWith('http://') || returnUrl.startsWith('https://') || returnUrl.startsWith('//');

    if (isInvalidPath || isExternalUrl || !returnUrl.startsWith('/')) {
        logger.warn('[AUTH] Invalid return URL detected, defaulting to home', { returnUrl });
        return '/';
    }

    return returnUrl;
}
