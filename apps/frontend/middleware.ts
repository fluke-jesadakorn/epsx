/**
 * Enhanced JWT Middleware for Frontend (Trading Platform)
 * Uses JWT cookie verification with direct OAuth redirect
 */
import { NextRequest, NextResponse } from 'next/server';
import { verifyJWT } from '@/lib/auth-utils';
import { getAuthorizationUrl } from '@/lib/server/auth';

export default async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Public routes that don't require authentication
  const publicRoutes = [
    '/',
    '/register', 
    '/forgot-password',
    '/reset-password',
    '/verify-email',
    '/access-denied',
    '/unauthorized',
    '/terms',
    '/privacy',
    '/analytics',
    '/api/auth/signin',
    '/api/auth/signout', 
    '/api/auth/logout',
    '/api/auth/callback',
    '/api/auth/session',
  ];
  
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  // Create response with security headers
  const response = NextResponse.next();
  
  // Add security headers
  response.headers.set('x-pathname', pathname);
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  
  // Allow access to public routes
  if (isPublicRoute) {
    return response;
  }
  
  try {
    // Get JWT token from frontend-specific httpOnly cookie
    const jwtToken = request.cookies.get('epsx_frontend_jwt')?.value;
    
    // Redirect to backend Analytics login page if no token
    if (!jwtToken) {
      console.log('🔓 Frontend middleware: No JWT token found, redirecting to backend Analytics login');
      
      try {
        // Generate authorization URL with PKCE parameters for backend login
        const callbackUrl = pathname + request.nextUrl.search;
        const { url: authorizationUrl, codeVerifier, state } = await getAuthorizationUrl();
        
        console.log('✅ Frontend middleware: PKCE parameters generated, redirecting to backend Analytics login');
        
        // Redirect to backend Analytics login page instead of direct OAuth
        const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
        backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
        backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
        backendLoginUrl.searchParams.set('scope', 'openid profile email');
        backendLoginUrl.searchParams.set('response_type', 'code');
        backendLoginUrl.searchParams.set('state', state);
        backendLoginUrl.searchParams.set('code_challenge', authorizationUrl.split('code_challenge=')[1]?.split('&')[0] || '');
        backendLoginUrl.searchParams.set('code_challenge_method', 'S256');
        
        // Create redirect response to backend Analytics login
        const analyticsRedirect = NextResponse.redirect(backendLoginUrl.toString());
        
        // Set PKCE parameters in httpOnly cookies for callback processing
        analyticsRedirect.cookies.set('oauth_code_verifier', codeVerifier, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        analyticsRedirect.cookies.set('oauth_state', state, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        analyticsRedirect.cookies.set('oauth_callback_url', callbackUrl, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        console.log('✅ Frontend middleware: Redirecting to backend Analytics login page');
        return analyticsRedirect;
        
      } catch (error) {
        console.error('❌ Frontend middleware: Failed to redirect to backend Analytics login:', error);
        // Fallback to backend login page directly
        const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
        backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
        backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
        backendLoginUrl.searchParams.set('scope', 'openid profile email');
        backendLoginUrl.searchParams.set('response_type', 'code');
        backendLoginUrl.searchParams.set('error', 'oauth_generation_failed');
        return NextResponse.redirect(backendLoginUrl.toString());
      }
    }
    
    // Verify JWT token
    const payload = await verifyJWT(jwtToken);
    
    // Redirect to backend Analytics login if invalid token
    if (!payload) {
      console.log('🔓 Frontend middleware: Invalid JWT token, redirecting to backend Analytics login');
      
      try {
        // Generate authorization URL with PKCE parameters for backend login
        const callbackUrl = pathname + request.nextUrl.search;
        const { url: authorizationUrl, codeVerifier, state } = await getAuthorizationUrl();
        
        console.log('✅ Frontend middleware: PKCE parameters generated for invalid token, redirecting to backend Analytics login');
        
        // Redirect to backend Analytics login page
        const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
        backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
        backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
        backendLoginUrl.searchParams.set('scope', 'openid profile email');
        backendLoginUrl.searchParams.set('response_type', 'code');
        backendLoginUrl.searchParams.set('state', state);
        backendLoginUrl.searchParams.set('code_challenge', authorizationUrl.split('code_challenge=')[1]?.split('&')[0] || '');
        backendLoginUrl.searchParams.set('code_challenge_method', 'S256');
        
        // Create redirect response to backend Analytics login
        const analyticsRedirect = NextResponse.redirect(backendLoginUrl.toString());
        
        // Set PKCE parameters in httpOnly cookies
        analyticsRedirect.cookies.set('oauth_code_verifier', codeVerifier, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        analyticsRedirect.cookies.set('oauth_state', state, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        analyticsRedirect.cookies.set('oauth_callback_url', callbackUrl, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 10 * 60, // 10 minutes
          path: '/'
        });
        
        // Clear invalid JWT token
        analyticsRedirect.cookies.delete('epsx_frontend_jwt');
        
        console.log('✅ Frontend middleware: Redirecting to backend Analytics login for invalid token with cleaned cookies');
        return analyticsRedirect;
        
      } catch (error) {
        console.error('❌ Frontend middleware: Failed to redirect to backend Analytics login for invalid token:', error);
        // Fallback to backend login page directly
        const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
        backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
        backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
        backendLoginUrl.searchParams.set('scope', 'openid profile email');
        backendLoginUrl.searchParams.set('response_type', 'code');
        backendLoginUrl.searchParams.set('error', 'invalid_token');
        return NextResponse.redirect(backendLoginUrl.toString());
      }
    }
    
    // Add user info to headers for server components (non-sensitive data only)
    response.headers.set('x-user-id', payload.sub);
    response.headers.set('x-user-role', payload.role);
    response.headers.set('x-user-package-tier', payload.package_tier || 'FREE');
    
    console.log(`🔐 Frontend middleware: Authenticated user ${payload.email} accessing ${pathname}`);
    
    // Allow access to protected routes for authenticated users
    return response;
    
  } catch (error) {
    console.error('❌ Frontend middleware JWT verification failed:', error);
    
    try {
      // Redirect to backend Analytics login on JWT verification error
      const callbackUrl = pathname + request.nextUrl.search;
      const { url: authorizationUrl, codeVerifier, state } = await getAuthorizationUrl();
      
      console.log('✅ Frontend middleware: PKCE parameters generated for JWT error, redirecting to backend Analytics login');
      
      // Redirect to backend Analytics login page
      const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
      backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
      backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
      backendLoginUrl.searchParams.set('scope', 'openid profile email');
      backendLoginUrl.searchParams.set('response_type', 'code');
      backendLoginUrl.searchParams.set('state', state);
      backendLoginUrl.searchParams.set('code_challenge', authorizationUrl.split('code_challenge=')[1]?.split('&')[0] || '');
      backendLoginUrl.searchParams.set('code_challenge_method', 'S256');
      
      // Create redirect response to backend Analytics login
      const analyticsRedirect = NextResponse.redirect(backendLoginUrl.toString());
      
      // Set PKCE parameters in httpOnly cookies
      analyticsRedirect.cookies.set('oauth_code_verifier', codeVerifier, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 10 * 60, // 10 minutes
        path: '/'
      });
      
      analyticsRedirect.cookies.set('oauth_state', state, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 10 * 60, // 10 minutes
        path: '/'
      });
      
      analyticsRedirect.cookies.set('oauth_callback_url', callbackUrl, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 10 * 60, // 10 minutes
        path: '/'
      });
      
      // Clear any corrupted JWT token
      analyticsRedirect.cookies.delete('epsx_frontend_jwt');
      
      console.log('✅ Frontend middleware: Redirecting to backend Analytics login for JWT error with cleaned cookies');
      return analyticsRedirect;
      
    } catch (oauthError) {
      console.error('❌ Frontend middleware: Failed to redirect to backend Analytics login during error handling:', oauthError);
      // Ultimate fallback to backend login page
      const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
      backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
      backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
      backendLoginUrl.searchParams.set('scope', 'openid profile email');
      backendLoginUrl.searchParams.set('response_type', 'code');
      backendLoginUrl.searchParams.set('error', 'authentication_error');
      return NextResponse.redirect(backendLoginUrl.toString());
    }
  }
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes) 
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public (public assets)
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};