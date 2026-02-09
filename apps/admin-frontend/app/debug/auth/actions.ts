'use server';

import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';

/**
 * Get current server-side session status
 */
export async function getServerSessionStatus() {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get(COOKIES.access_token);
    const refreshToken = cookieStore.get(COOKIES.refresh_token);
    const user = cookieStore.get(COOKIES.user);

    return {
        hasAccessToken: Boolean(accessToken),
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        accessTokenValue: accessToken?.value ? `${accessToken.value.slice(0, 10)}...` : null,
        // accessTokenExp: accessToken?.maxAge, // maxAge not available on RequestCookie

        hasRefreshToken: Boolean(refreshToken),
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        refreshTokenValue: refreshToken?.value ? `${refreshToken.value.slice(0, 10)}...` : null,

        hasUser: Boolean(user),
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions, @typescript-eslint/no-unsafe-assignment
        userValue: user?.value ? JSON.parse(user.value) : null
    };
}

/**
 * Manually expire the access token (set to "EXPIRED")
 */
export async function expireAccessToken() {
    const cookieStore = await cookies();

    // Overwrite with invalid token
    cookieStore.set(COOKIES.access_token, 'EXPIRED_TEST_TOKEN', {
        httpOnly: true,
        path: '/',
        maxAge: 1 // Expire almost immediately
    });

    return { success: true, message: 'Access Token expired' };
}

/**
 * Manually expire the refresh token
 */
export async function expireRefreshToken() {
    const cookieStore = await cookies();

    // Overwrite with invalid token
    cookieStore.set(COOKIES.refresh_token, 'EXPIRED_TEST_TOKEN', {
        httpOnly: true,
        path: '/',
        maxAge: 1
    });

    return { success: true, message: 'Refresh Token expired' };
}
