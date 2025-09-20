import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { SiweMessage } from 'siwe';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

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

    console.log('🔄 Admin: Verifying Web3 signature for wallet:', wallet_address);

    // Validate SIWE message format
    try {
      const siweMessage = new SiweMessage(message);
      
      // Verify the message contains admin context
      if (!siweMessage.statement?.includes('Admin') && !siweMessage.requestId?.includes('admin')) {
        console.warn('⚠️ Admin: SIWE message missing admin context');
      }
      
      // Verify wallet address matches
      if (siweMessage.address.toLowerCase() !== wallet_address.toLowerCase()) {
        return NextResponse.json(
          { error: 'Wallet address mismatch in SIWE message' },
          { status: 400 }
        );
      }
    } catch (error) {
      console.error('❌ Admin: Invalid SIWE message format:', error);
      return NextResponse.json(
        { error: 'Invalid SIWE message format' },
        { status: 400 }
      );
    }

    // Forward to backend Web3 verify endpoint with admin context
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Admin-Context': 'true', // Mark as admin authentication request
      },
      body: JSON.stringify({ 
        wallet_address, 
        signature, 
        nonce, 
        message,
        admin_context: true // Request admin permission validation
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Verification failed' }));
      console.error('❌ Admin: Backend verification failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    
    // Verify admin permissions exist
    if (!data.permissions || !Array.isArray(data.permissions)) {
      console.error('❌ Admin: No permissions returned from backend');
      return NextResponse.json(
        { error: 'No permissions found for wallet' },
        { status: 403 }
      );
    }
    
    // Check for admin permissions
    const hasAdminPerms = data.permissions.some((permission: string) => 
      permission === 'admin:*:*' || 
      permission.startsWith('admin:') ||
      permission === 'epsx:admin:*'
    );
    
    if (!hasAdminPerms) {
      console.error('❌ Admin: Wallet lacks admin permissions:', wallet_address);
      return NextResponse.json(
        { error: 'Wallet does not have admin permissions' },
        { status: 403 }
      );
    }
    
    console.log('✅ Admin: Wallet verification successful for admin:', wallet_address);
    
    // Set wallet session cookies
    const cookieStore = await cookies();
    const expiresIn = 3600; // 1 hour
    const expiresAt = Date.now() + (expiresIn * 1000);
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: expiresIn,
      path: '/',
    };

    cookieStore.set('wallet_address', wallet_address, cookieOptions);
    cookieStore.set('wallet_nonce', nonce, cookieOptions);
    cookieStore.set('wallet_signature', signature, cookieOptions);
    cookieStore.set('wallet_message', message, cookieOptions);
    cookieStore.set('wallet_expires_at', expiresAt.toString(), cookieOptions);
    
    // Clear any legacy OIDC tokens
    cookieStore.delete('access_token');
    cookieStore.delete('id_token');
    cookieStore.delete('refresh_token');
    
    // Return success response without sensitive data
    return NextResponse.json({
      success: true,
      wallet_address: data.wallet_address,
      user_id: data.user_id,
      email: data.email,
      permissions: data.permissions,
      admin_level: data.admin_level || 'admin',
      expires_at: expiresAt
    });

  } catch (error) {
    console.error('❌ Admin: Web3 verify API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}