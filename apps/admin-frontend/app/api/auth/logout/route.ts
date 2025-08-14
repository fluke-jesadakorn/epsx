/**
 * Logout API Route
 * Clears user session and logs out
 */
import { NextRequest, NextResponse } from 'next/server';
import { clearSession } from '@/lib/auth/session';

export async function POST(request: NextRequest) {
  try {
    console.log('🚪 User logout requested');

    // Clear user session
    await clearSession();

    console.log('✅ User logged out successfully');

    // Return success response
    return NextResponse.json({ 
      success: true,
      message: 'Logged out successfully' 
    });
  } catch (error) {
    console.error('❌ Logout error:', error);
    
    return NextResponse.json(
      { success: false, error: 'Logout failed' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🚪 User logout requested (GET)');

    // Clear user session
    await clearSession();

    console.log('✅ User logged out successfully');

    // Redirect to login page
    const loginUrl = new URL('/login', request.url);
    return NextResponse.redirect(loginUrl);
  } catch (error) {
    console.error('❌ Logout error:', error);
    
    // Still redirect to login on error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'logout_error');
    return NextResponse.redirect(loginUrl);
  }
}