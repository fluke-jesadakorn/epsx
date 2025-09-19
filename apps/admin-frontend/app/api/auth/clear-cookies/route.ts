/**
 * Clear shared authentication cookies
 * Useful for testing and forcing re-authentication
 */
import { NextResponse } from 'next/server';

export async function POST() {
  try {
    const response = NextResponse.json({ success: true, message: 'All cookies cleared' });
    
    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');
    
    // Clear OAuth cookies
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');
    
    // Clear backup cookies
    response.cookies.delete('pkce_verifier_backup');
    response.cookies.delete('pkce_state_backup');
    response.cookies.delete('admin_oauth_verifier');
    response.cookies.delete('admin_oauth_state');
    
    console.log('✅ OIDC authentication and OAuth cookies cleared');
    return response;
    
  } catch (error) {
    console.error('❌ Failed to clear cookies:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear cookies' },
      { status: 500 }
    );
  }
}