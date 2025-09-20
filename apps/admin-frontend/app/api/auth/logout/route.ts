/**
 * Admin Frontend Logout API Route - Wallet Authentication
 * Clears wallet session cookies and notifies backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { URL, URLContext, Service } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing wallet logout request');

    // Wallet Authentication: Get wallet session from cookies
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'Admin wallet session cleared successfully' 
    });

    // Wallet Authentication: Clear wallet session cookies
    response.cookies.delete('wallet_address');
    response.cookies.delete('wallet_nonce');
    response.cookies.delete('wallet_signature');
    response.cookies.delete('wallet_message');
    response.cookies.delete('wallet_expires_at');
    
    // Also clear legacy OIDC tokens
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    console.log('✅ Admin: Wallet session cookies cleared');

    // Notify backend of wallet logout
    if (walletAddress) {
      try {
        const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
        
        // Call Web3 logout endpoint
        const logoutResponse = await fetch(`${backendUrl}/api/auth/web3/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Admin-Context': 'true'
          },
          body: JSON.stringify({
            wallet_address: walletAddress,
            admin_context: true
          }),
          // Add timeout to prevent hanging
          signal: AbortSignal.timeout(5000),
        });

        if (logoutResponse.ok) {
          console.log('✅ Admin: Backend wallet logout successful');
        } else {
          console.warn('⚠️ Admin: Backend wallet logout failed, but cookies cleared');
        }
      } catch (backendError) {
        console.warn('⚠️ Admin: Backend wallet logout error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Admin: No wallet address found, skipping backend logout');
    }

    console.log('✅ Admin: Logout completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Admin: Wallet logout error:', error);

    // Still try to clear cookies even if there's an error
    const response = NextResponse.json({ 
      success: false,
      error: 'Wallet logout failed',
      message: 'An error occurred during admin wallet logout'
    }, { status: 500 });

    // Wallet Authentication: Clear wallet session cookies on error
    response.cookies.delete('wallet_address');
    response.cookies.delete('wallet_nonce');
    response.cookies.delete('wallet_signature');
    response.cookies.delete('wallet_message');
    response.cookies.delete('wallet_expires_at');
    
    // Also clear legacy OIDC tokens
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    return response;
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing wallet logout request (GET)');

    // Wallet Authentication: Get wallet session from cookies
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;

    // Create redirect response
    const loginUrl = new URL('/login', request.url);
    const response = NextResponse.redirect(loginUrl);

    // Wallet Authentication: Clear wallet session cookies
    response.cookies.delete('wallet_address');
    response.cookies.delete('wallet_nonce');
    response.cookies.delete('wallet_signature');
    response.cookies.delete('wallet_message');
    response.cookies.delete('wallet_expires_at');
    
    // Also clear legacy OIDC tokens
    response.cookies.delete('access_token');
    response.cookies.delete('id_token'); 
    response.cookies.delete('refresh_token');

    console.log('✅ Admin: Wallet session cookies cleared, redirecting to login');

    // Notify backend of wallet logout
    if (walletAddress) {
      try {
        const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
        
        // Call Web3 logout endpoint
        await fetch(`${backendUrl}/api/auth/web3/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Admin-Context': 'true'
          },
          body: JSON.stringify({
            wallet_address: walletAddress,
            admin_context: true
          }),
          signal: AbortSignal.timeout(5000),
        });
        console.log('✅ Admin: Backend wallet logout successful during GET');
      } catch (backendError) {
        console.warn('⚠️ Admin: Backend wallet logout error during GET:', backendError);
        // Continue with redirect even if backend fails
      }
    }

    return response;

  } catch (error) {
    console.error('❌ Admin: Wallet logout error (GET):', error);
    
    // Still redirect to login on error but clear cookies
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'wallet_logout_error');
    const response = NextResponse.redirect(loginUrl);
    
    // Wallet Authentication: Clear wallet session cookies on error
    response.cookies.delete('wallet_address');
    response.cookies.delete('wallet_nonce');
    response.cookies.delete('wallet_signature');
    response.cookies.delete('wallet_message');
    response.cookies.delete('wallet_expires_at');
    
    // Also clear legacy OIDC tokens
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    return response;
  }
}