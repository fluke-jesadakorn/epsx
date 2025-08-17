/**
 * Clear shared authentication cookies
 * Useful for testing and forcing re-authentication
 */
import { NextResponse } from 'next/server';

export async function POST() {
  try {
    const response = NextResponse.json({ success: true, message: 'All cookies cleared' });
    
    // Clear shared JWT cookies
    response.cookies.delete('epsx_jwt');
    response.cookies.delete('epsx_refresh');
    
    // Clear OAuth cookies
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');
    
    console.log('✅ Shared authentication cookies cleared');
    return response;
    
  } catch (error) {
    console.error('❌ Failed to clear cookies:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear cookies' },
      { status: 500 }
    );
  }
}