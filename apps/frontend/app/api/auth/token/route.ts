import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const AUTH_COOKIE_NAME = 'frontend_bearer_token';

/**
 * GET /api/auth/token
 * Returns the bearer token from HTTP-only cookie for client-side API calls
 */
export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get(AUTH_COOKIE_NAME)?.value;

    if (!token) {
      return NextResponse.json({ error: 'No token found' }, { status: 401 });
    }

    return NextResponse.json({ token });
  } catch (error) {
    console.error('Token API error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}