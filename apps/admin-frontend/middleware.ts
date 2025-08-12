/**
 * Modern Next.js Middleware - Admin Module Based Authentication
 * Completely replaces legacy role-based middleware
 */

import { NextRequest, NextResponse } from 'next/server'
import { ModernAuthService } from '@/lib/auth/modern-auth-service'

// Routes that require authentication
const protectedRoutes = [
  '/dashboard',
  '/users',
  '/analytics', 
  '/billing',
  '/settings',
  '/admin',
  '/modules',
  '/permissions'
]

// Routes that require specific admin modules
const adminModuleRoutes: Record<string, string> = {
  '/users': 'user_operations',
  '/analytics': 'analytics_specialist', 
  '/billing': 'billing_admin',
  '/settings': 'system_admin',
  '/permissions': 'permission_admin',
  '/modules': 'module_coordinator'
}

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/auth/callback',
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico',
  '/api/auth'
]

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl

  // Skip middleware for public routes
  if (publicRoutes.some(route => pathname.startsWith(route))) {
    return NextResponse.next()
  }

  // Check if route requires authentication
  const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route))
  
  if (!isProtectedRoute) {
    return NextResponse.next()
  }

  // Use modern auth service for authentication
  const authResult = await ModernAuthService.authMiddleware(request)
  
  // If auth service returned a response (redirect), use it
  if (authResult && authResult !== NextResponse.next()) {
    return authResult
  }

  // Check for admin module requirements
  const requiredModule = Object.entries(adminModuleRoutes).find(([route]) => 
    pathname.startsWith(route)
  )?.[1]

  if (requiredModule) {
    // Get token and validate admin module access
    const token = request.cookies.get('modern_admin_token')?.value

    if (token) {
      try {
        // Validate admin module access with backend
        const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:8080'}/api/v1/auth/me`, {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
          }
        })

        if (response.ok) {
          const userData = await response.json()
          
          // Check if user has required admin module
          if (!userData.admin_modules?.includes(requiredModule)) {
            const accessDeniedUrl = new URL('/access-denied', request.url)
            accessDeniedUrl.searchParams.set('required_module', requiredModule)
            return NextResponse.redirect(accessDeniedUrl)
          }
        }
      } catch (error) {
        console.error('Admin module validation error:', error)
        return NextResponse.redirect(new URL('/login', request.url))
      }
    }
  }

  return NextResponse.next()
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api/public (public API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api/public|_next/static|_next/image|favicon.ico).*)',
  ],
}