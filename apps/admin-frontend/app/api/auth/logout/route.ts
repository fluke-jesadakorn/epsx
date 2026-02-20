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
        cookieStore.delete(cookieName);
    });

    return NextResponse.redirect(new URL(returnUrl, request.url));
}
