/**
 * Web3 Permissions API Route
 * Fetches wallet permissions and user data
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
    
    if (!wallet_address) {
      return NextResponse.json({
        success: false,
        error: 'Wallet address is required'
      }, { status: 400 });
    }

    console.log('🔍 Fetching Web3 permissions for wallet:', wallet_address.slice(0, 8) + '...');

    // Get Bearer token from cookies for authentication
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Include Bearer token if available
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    // Fetch Web3 permissions from backend
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/permissions?wallet_address=${encodeURIComponent(wallet_address)}`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      console.warn(`Web3 permissions API returned ${response.status} for wallet ${wallet_address}`);
      
      // Return default values for failed API calls
      return NextResponse.json({
        wallet_address,
        permissions: [],
        tier: 'basic',
        has_access: false,
        error: 'Permissions API unavailable'
      });
    }

    const permissionsData = await response.json();
    
    // Return standardized format matching backend response
    return NextResponse.json({
      wallet_address: permissionsData.wallet_address,
      permissions: permissionsData.permissions || [],
      tier: permissionsData.tier || 'basic',
      has_access: permissionsData.has_access || false,
      user_id: permissionsData.user_id,
      expires_at: permissionsData.expires_at
    });
    
  } catch (error) {
    console.error('❌ Web3 permissions fetch error:', error);
    
    // Return safe defaults on error
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    
    return NextResponse.json({
      wallet_address,
      permissions: [],
      tier: 'basic',
      has_access: false,
      error: 'Failed to fetch permissions data'
    });
  }
}