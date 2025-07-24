import { NextRequest, NextResponse } from 'next/server';
import { ApiCookies } from '@/lib/cookies';

export async function POST(request: NextRequest) {
  try {
    const response = NextResponse.json({ success: true, message: 'Cookies cleared' });
    
    // Clear all authentication cookies
    ApiCookies.clearAuthCookies(response);
    
    return response;
  } catch (error) {
    console.error('Error clearing cookies:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear cookies' },
      { status: 500 }
    );
  }
}