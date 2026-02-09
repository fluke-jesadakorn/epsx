'use server';

import { getServerAuthToken } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';

/**
 * Server Action to verify if the client has a valid server-side session.
 * Used by the Auth Page to prevent redirect loops where client thinks it's auth'd
 * but server doesn't have the token.
 */
export async function verifySessionAction() {
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);

    return {
        valid: Boolean(token)
    };
}
