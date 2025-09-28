/**
 * Admin Web3 Verify API Route
 * Verifies SIWE signatures for admin authentication using unified client
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3AdminClient } from '@/shared/utils/web3-api-client';
import { hasAdminPermissions } from '@/shared/types/wallet-auth';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, signature, nonce, message } = body;

    if (!wallet_address || !signature || !nonce || !message) {
      return NextResponse.json(
        { error: 'Missing required fields: wallet_address, signature, nonce, message' },
        { status: 400 }
      );
    }

    console.log('🔄 Admin: Verifying Web3 signature for wallet:', wallet_address.slice(0, 8) + '...');

    // Create Web3 admin client for server-side verification
    const web3Client = createWeb3AdminClient({ serverSide: true });
    
    // Verify signature using typed client
    const authData = await web3Client.verifySignature({
      wallet_address,
      signature,
      nonce,
      message
    });

    // Verify this is an admin by checking permissions
    const hasAdminPerms = hasAdminPermissions(authData.permissions || []);
    
    if (!hasAdminPerms) {
      console.error('❌ Admin: Wallet lacks admin permissions:', wallet_address.slice(0, 8) + '...');
      return NextResponse.json(
        { error: 'Wallet does not have admin permissions' },
        { status: 403 }
      );
    }
    
    const expiresIn = 3600; // 1 hour for admin sessions
    
    // Set admin session cookies
    if (authData.access_token) {
      const cookieStore = await cookies();
      
      const cookieOptions = {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        maxAge: expiresIn,
        path: '/',
      };

      // Set Bearer token for backend API calls
      cookieStore.set('access_token', authData.access_token, cookieOptions);
      
      // Set admin session marker
      cookieStore.set('admin_session', '1', cookieOptions);
      cookieStore.set('wallet_address', wallet_address, cookieOptions);
      
      // Clean up any legacy session cookies
      cookieStore.delete('wallet_nonce');
      cookieStore.delete('wallet_signature'); 
      cookieStore.delete('wallet_message');
    }
    
    console.log('✅ Admin: Wallet verification successful for admin:', wallet_address.slice(0, 8) + '...');
    
    // Return admin user data
    return NextResponse.json({
      success: true,
      wallet_address: authData.wallet_address,
      user_id: authData.user_id || wallet_address,
      permissions: authData.permissions,
      admin_level: 'admin',
      expires_at: Date.now() + (expiresIn * 1000)
    });

  } catch (error) {
    console.error('❌ Admin: Web3 verify API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}