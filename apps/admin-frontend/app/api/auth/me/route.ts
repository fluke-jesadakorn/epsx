/**
 * Modern Auth Me Endpoint - Admin Module System
 * Returns current user with admin module data
 */

import { NextResponse } from 'next/server'
import { ModernAuthService } from '@/lib/auth/modern-auth-service'

export async function GET() {
  try {
    const user = await ModernAuthService.getCurrentUser()
    
    if (!user) {
      return NextResponse.json({ error: 'Not authenticated' }, { status: 401 })
    }

    return NextResponse.json(user)
  } catch (error) {
    console.error('Auth me endpoint error:', error)
    return NextResponse.json(
      { error: 'Authentication failed' },
      { status: 500 }
    )
  }
}