/**
 * Modern Auth Login Endpoint - Admin Module System
 * Handles both OIDC redirects and direct login
 */

import { NextRequest, NextResponse } from 'next/server'
import { ModernAuthService } from '@/lib/auth/modern-auth-service'
import { cookies } from 'next/headers'

/**
 * GET: OIDC Authorization Redirect
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams
  const callbackUrl = searchParams.get('callbackUrl') || '/dashboard'
  
  // Store callback URL for after authentication
  const cookieStore = await cookies()
  cookieStore.set('auth_callback_url', callbackUrl, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 60 * 10, // 10 minutes
    path: '/'
  })
  
  // Build modern OIDC authorization URL
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080'
  const clientId = 'epsx-admin'
  const redirectUri = `${process.env.NEXTAUTH_URL || 'http://localhost:3001'}/api/auth/callback`
  const state = generateSecureState()
  const nonce = generateSecureNonce()
  
  // Store security parameters
  cookieStore.set('oidc_state', state, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production', 
    sameSite: 'lax',
    maxAge: 60 * 10,
    path: '/'
  })
  
  cookieStore.set('oidc_nonce', nonce, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax', 
    maxAge: 60 * 10,
    path: '/'
  })
  
  // Modern OIDC authorization URL with admin scope
  const authUrl = new URL(`${backendUrl}/oauth/authorize`)
  authUrl.searchParams.set('client_id', clientId)
  authUrl.searchParams.set('redirect_uri', redirectUri)
  authUrl.searchParams.set('response_type', 'code')
  authUrl.searchParams.set('scope', 'openid profile email admin_modules')
  authUrl.searchParams.set('state', state)
  authUrl.searchParams.set('nonce', nonce)
  
  console.log('🚀 Modern OIDC Authorization:', {
    url: authUrl.toString(),
    callback: callbackUrl,
    timestamp: new Date().toISOString()
  })
  
  return NextResponse.redirect(authUrl.toString())
}

/**
 * POST: Direct Login (for form-based authentication)
 */
export async function POST(request: NextRequest) {
  try {
    const { email, password } = await request.json()

    if (!email || !password) {
      return NextResponse.json(
        { error: 'Email and password are required' },
        { status: 400 }
      )
    }

    const result = await ModernAuthService.login(email, password)
    
    if (result.success) {
      return NextResponse.json({ success: true })
    } else {
      return NextResponse.json(
        { error: result.error || 'Invalid credentials' },
        { status: 401 }
      )
    }
  } catch (error) {
    console.error('Login endpoint error:', error)
    return NextResponse.json(
      { error: 'Authentication failed' },
      { status: 500 }
    )
  }
}

function generateSecureState(): string {
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    const array = new Uint8Array(32)
    crypto.getRandomValues(array)
    return btoa(String.fromCharCode(...array))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '')
  }
  
  return Math.random().toString(36).substring(2) + 
         Math.random().toString(36).substring(2) +
         Date.now().toString(36)
}

function generateSecureNonce(): string {
  return generateSecureState()
}