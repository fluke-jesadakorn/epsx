'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { COOKIES } from '@/shared/auth/cookies';

const BACKEND_URL = process.env.BACKEND_URL ?? 'http://127.0.0.1:8080';

export async function verifyTurnstileAction(
    token: string,
    from: string
): Promise<{ error: string } | undefined> {
    const safeFrom = typeof from === 'string' && from.startsWith('/') ? from : '/';

    try {
        const res = await fetch(`${BACKEND_URL}/api/public/verify-turnstile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ token }),
            cache: 'no-store',
        });

        if (!res.ok) {
            return { error: 'Verification service unavailable. Please try again.' };
        }

        const data = (await res.json()) as { success?: boolean };
        if (data.success !== true) {
            return { error: 'Verification failed. Please try again.' };
        }
    } catch {
        return { error: 'Verification service unavailable. Please try again.' };
    }

    const isProd = process.env.NODE_ENV === 'production';
    const cookieStore = await cookies();
    cookieStore.set(COOKIES.turnstile, '1', {
        httpOnly: true,
        secure: isProd,
        sameSite: 'lax',
        path: '/',
        maxAge: 86400, // 24 hours
    });

    redirect(safeFrom);
}
