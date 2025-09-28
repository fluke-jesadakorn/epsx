import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

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

    // Forward verification request to backend
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        wallet_address,
        signature,
        nonce,
        message
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Backend verification failed' 
      }));
      console.error('❌ Admin: Backend Web3 verification failed:', response.status);
      return NextResponse.json(errorData, { status: response.status });
    }

    const authData = await response.json();
    
    // Verify this is an admin by checking permissions
    const permissions = authData.permissions || [];
    const hasAdminPerms = permissions.some((p: string) => 
      p === 'admin:*:*' || 
      p.startsWith('admin:') ||
      p === 'epsx:admin:*' ||
      p === 'epsx:*:*'
    );
    
    if (!hasAdminPerms) {
      console.error('❌ Admin: Wallet lacks admin permissions:', wallet_address);
      return NextResponse.json(
        { error: 'Wallet does not have admin permissions' },
        { status: 403 }
      );
    }
    
    console.log('✅ Admin: Web3 verification successful for admin:', wallet_address);
    
    // Set admin session cookies using backend tokens
    const cookieStore = await cookies();
    const expiresIn = 3600; // 1 hour
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: expiresIn,
      path: '/',
    };

    // Set Bearer token cookies from backend
    if (authData.access_token) {
      cookieStore.set('access_token', authData.access_token, cookieOptions);
    }
    
    if (authData.refresh_token) {
      cookieStore.set('refresh_token', authData.refresh_token, cookieOptions);
    }

    // Set admin session marker
    cookieStore.set('admin_session', 'true', cookieOptions);
    cookieStore.set('wallet_address', wallet_address, cookieOptions);
    
    // Return success response
    return NextResponse.json({
      success: true,
      wallet_address: authData.wallet_address,
      user_id: authData.user_id || authData.wallet_address,
      permissions: authData.permissions,
      admin_level: 'admin',
      expires_at: authData.expires_at
    });

  } catch (error) {
    console.error('❌ Admin: Web3 verify API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}