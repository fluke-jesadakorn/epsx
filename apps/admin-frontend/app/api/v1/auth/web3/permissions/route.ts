/**
 * Admin Web3 Permissions API Route
 * Fetches wallet permissions for admin users
 * Aligns with backend /api/v1/auth/web3/permissions
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    
    // Verify admin session
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!accessToken || !sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    // Use session wallet if no specific wallet requested
    const targetWallet = wallet_address || sessionWalletAddress;

    console.log('🔍 Admin: Fetching Web3 permissions for wallet:', targetWallet.slice(0, 8) + '...');

    // Forward to backend with Bearer token authentication
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/permissions?wallet_address=${encodeURIComponent(targetWallet)}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
    });

    if (!response.ok) {
      console.error(`❌ Admin: Backend permissions API failed: ${response.status}`);
      return NextResponse.json(
        { error: 'Failed to fetch permissions' },
        { status: response.status }
      );
    }

    const permissionsData = await response.json();
    
    console.log('✅ Admin: Retrieved permissions from backend for wallet:', targetWallet.slice(0, 8) + '...');

    return NextResponse.json({
      wallet_address: permissionsData.wallet_address,
      permissions: permissionsData.permissions || [],
      tier: permissionsData.tier || 'basic',
      has_access: permissionsData.has_access || false,
      user_id: permissionsData.user_id,
      expires_at: permissionsData.expires_at
    });

  } catch (error) {
    console.error('❌ Admin: Web3 permissions fetch error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}