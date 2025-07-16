import { NextRequest, NextResponse } from 'next/server';

export async function userAccessMiddleware(request: NextRequest) {
  const response = NextResponse.next();
  
  // Check if user access cookies are already set
  const userLevel = request.cookies.get('userLevel');
  const isExpired = request.cookies.get('isExpired');
  
  // If not set, set default values
  if (!userLevel) {
    response.cookies.set('userLevel', 'BASIC', {
      httpOnly: false,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
  }
  
  if (!isExpired) {
    response.cookies.set('isExpired', 'true', {
      httpOnly: false,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
  }
  
  return response;
}

// Helper function to update user access cookies when user status changes
export function updateUserAccessCookies(userLevel: string, isExpired: boolean) {
  if (typeof document !== 'undefined') {
    document.cookie = `userLevel=${userLevel}; path=/; max-age=${60 * 60 * 24 * 7}`;
    document.cookie = `isExpired=${isExpired}; path=/; max-age=${60 * 60 * 24 * 7}`;
  }
}
