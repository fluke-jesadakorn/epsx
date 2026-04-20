import { COOKIES, getServerAuthToken } from '@/shared/auth/cookies';
import {
  DESIGN_BYPASS_COOKIE,
  DESIGN_BYPASS_HEADER,
  DESIGN_BYPASS_QUERY_PARAM,
  isDesignBypassRequestEnabled,
  isDesignBypassTruthy,
} from '@/shared/utils/design-bypass';
import { getBackendUrl } from '@/shared/utils/url-resolver';
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

const PUBLIC_ROUTES = [
  '/login',
  '/auth',
  '/api/auth',
  '/api/public',
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico',
  '/robots.txt',
  '/sitemap.xml',
  '/manifest.json',
  '/screenshots',
];

const LOGIN_PATH = '/auth';
const BACKEND_URL = getBackendUrl('server');
const AUTH_COOKIE_NAMES = [
  COOKIES.access_token,
  COOKIES.refresh_token,
  COOKIES.id_token,
  COOKIES.user,
  COOKIES.sid,
  COOKIES.auth_time,
  COOKIES.expires_at,
];

function isPublicRoute(pathname: string): boolean {
  return PUBLIC_ROUTES.some(route => pathname.startsWith(route));
}

function clearAuthCookies(response: NextResponse): NextResponse {
  const isProd = process.env.NODE_ENV === 'production';

  AUTH_COOKIE_NAMES.forEach(name => {
    response.cookies.set(name, '', {
      maxAge: 0,
      secure: isProd,
      sameSite: 'lax',
      path: '/',
    });
  });

  return response;
}

function createNextResponse(requestHeaders?: Headers): NextResponse {
  if (requestHeaders === undefined) {
    return NextResponse.next();
  }

  return NextResponse.next({
    request: {
      headers: requestHeaders,
    },
  });
}

function handleDesignBypass(
  request: NextRequest,
  requestHeaders: Headers,
  bypassEnabled: boolean
): NextResponse | null {
  if (!request.nextUrl.searchParams.has(DESIGN_BYPASS_QUERY_PARAM)) {
    if (!bypassEnabled) {
      return null;
    }

    const response = createNextResponse(requestHeaders);
    response.headers.set(DESIGN_BYPASS_HEADER, '1');
    return response;
  }

  const raw = request.nextUrl.searchParams.get(DESIGN_BYPASS_QUERY_PARAM);
  const response = createNextResponse(requestHeaders);
  const shouldEnableBypass = isDesignBypassTruthy(raw);

  response.cookies.set(DESIGN_BYPASS_COOKIE, shouldEnableBypass ? '1' : '', {
    httpOnly: true,
    secure: DESIGN_BYPASS_COOKIE.startsWith('__Host-'),
    sameSite: 'lax',
    path: '/',
    maxAge: shouldEnableBypass ? 60 * 60 : 0,
  });

  if (bypassEnabled) {
    response.headers.set(DESIGN_BYPASS_HEADER, '1');
  }

  return response;
}

function handleLoginRoute(
  request: NextRequest,
  hasToken: boolean
): NextResponse | null {
  if (request.nextUrl.pathname !== LOGIN_PATH) {
    return null;
  }

  // Clear expired session cookies and redirect to gate
  const params = request.nextUrl.searchParams;
  if (params.get('reason') === 'no-session' || params.has('clear')) {
    return clearAuthCookies(NextResponse.redirect(new URL('/', request.url)));
  }

  // No token: pass through — page.tsx redirects to / which shows gate
  if (!hasToken) {
    return NextResponse.next();
  }

  // Has token: redirect authenticated users away from /auth
  return NextResponse.redirect(new URL('/', request.url));
}

async function extractErrorDetail(res: Response): Promise<string> {
  try {
    const text = await res.text();
    if (!text) {
      return '';
    }
    const data = JSON.parse(text) as Record<string, unknown>;
    if ('message' in data && typeof data.message === 'string') {
      return `&detail=${encodeURIComponent(data.message)}`;
    }
    if (
      'error' in data &&
      typeof data.error === 'object' &&
      data.error !== null
    ) {
      const errObj = data.error as Record<string, unknown>;
      if ('message' in errObj && typeof errObj.message === 'string') {
        return `&detail=${encodeURIComponent(errObj.message)}`;
      }
    }
  } catch (_e) {
    // Ignore parse errors
  }
  return '';
}

async function verifyAdminAccess(
  request: NextRequest,
  token: string
): Promise<NextResponse | null> {
  try {
    const res = await fetch(`${BACKEND_URL}/api/admin/me`, {
      method: 'GET',
      headers: { Authorization: `Bearer ${token}` },
      signal: AbortSignal.timeout(5000),
    });

    if (res.ok) {
      return null;
    }

    if (res.status === 401) {
      return clearAuthCookies(NextResponse.redirect(new URL('/auth', request.url)));
    }

    const detail = await extractErrorDetail(res);
    const reason =
      res.status === 403 ? 'insufficient_permissions' : 'backend_error';
    return NextResponse.redirect(
      new URL(`/access-denied?reason=${reason}${detail}`, request.url)
    );
  } catch {
    return NextResponse.redirect(
      new URL('/access-denied?reason=backend_unavailable', request.url)
    );
  }
}

/**
 * Admin-frontend middleware — authentication + authorization.
 * Verifies admin permissions by probing the backend; no JWT decoding in frontend.
 */
export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  const bypassEnabled = isDesignBypassRequestEnabled(
    request.nextUrl.searchParams,
    request.cookies.get(DESIGN_BYPASS_COOKIE)?.value
  );

  const requestHeaders = new Headers(request.headers);
  if (bypassEnabled) {
    requestHeaders.set(DESIGN_BYPASS_HEADER, '1');
  }

  const bypassResponse = handleDesignBypass(
    request,
    requestHeaders,
    bypassEnabled
  );
  if (bypassResponse !== null) {
    return bypassResponse;
  }

  // Logout via middleware — clears __Host- cookies at HTTP level
  if (request.nextUrl.searchParams.has('logout')) {
    const cleanUrl = new URL(pathname, request.url);
    return clearAuthCookies(NextResponse.redirect(cleanUrl));
  }

  const token = getServerAuthToken(request.cookies);
  const hasToken = token !== null;

  // Handle /auth (LOGIN_PATH) before public route check so authenticated users are redirected
  const loginResp = handleLoginRoute(request, hasToken);
  if (loginResp !== null) {
    return loginResp;
  }

  if (isPublicRoute(pathname)) {
    return NextResponse.next();
  }

  // No token: pass through — layout gate shows auth modal inline
  if (!hasToken) {
    return NextResponse.next();
  }

  // Verify admin permission by calling the backend — backend is the sole authority
  // Fail-closed approach: any non-2xx response rejects access
  const verifyResponse = await verifyAdminAccess(request, token);
  if (verifyResponse !== null) {
    return verifyResponse;
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico|api/health).*)'],
};
