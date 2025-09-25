/**
 * Web3 Logout API Route
 * Handles Web3 session logout
 * Aligns with backend /api/v1/auth/web3/logout
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function DELETE(request: NextRequest) {
  try {
    console.log('🔄 Processing Web3 logout request');

    // Get Bearer token from cookies
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;

    // Notify backend about logout if we have a token
    if (accessToken) {
      try {
        await fetch(`${BACKEND_URL}/api/v1/auth/web3/logout`, {
          method: 'DELETE',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${accessToken}`,
          },
        });
      } catch (error) {
        console.warn('Failed to notify backend about logout:', error);
        // Continue with local logout even if backend fails
      }
    }

    // Clear all auth-related cookies
    cookieStore.delete('access_token');
    cookieStore.delete('refresh_token');
    cookieStore.delete('web3_session');

    console.log('✅ Web3 logout completed successfully');

    return NextResponse.json({
      success: true,
      message: 'Logout successful'
    });

  } catch (error) {
    console.error('❌ Web3 logout error:', error);
    
    // Even if there's an error, clear cookies as fallback
    const cookieStore = await cookies();
    cookieStore.delete('access_token');
    cookieStore.delete('refresh_token');
    cookieStore.delete('web3_session');

    return NextResponse.json({
      success: true,
      message: 'Logout completed with warnings'
    });
  }
}

// Support POST for compatibility
export async function POST(request: NextRequest) {
  return DELETE(request);
}