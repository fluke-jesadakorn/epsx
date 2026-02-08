import { cookies } from 'next/headers';
import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

import { COOKIES } from '@/shared/auth/cookies';

/**
 * Server-side logout handler
 * Clears all authentication cookies (including HttpOnly ones) and redirects
 */
export async function GET(request: NextRequest) {
    const cookieStore = await cookies();
    const searchParams = request.nextUrl.searchParams;
    const returnUrl = searchParams.get('return_url') ?? '/auth';

    // Clear all known EPSX cookies
    Object.values(COOKIES).forEach((cookieName) => {
        // Delete with path / to ensure global clearance
        cookieStore.delete(cookieName);

        // Also try to delete with potentially different domain configurations if needed
        // But since we use __Host- prefix in production which restricts to path=/, simple delete should work
    });

    // Create redirect response
    const response = NextResponse.redirect(new URL(returnUrl, request.url));

    // Double ensure cookies are cleared by setting headers manually if needed
    // (NextResponse.redirect usually handles this if we modified cookieStore)

    return response;
}
