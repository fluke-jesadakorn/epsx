import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  console.log('🔍 SIMPLE GET REQUEST - TESTING ROUTE');
  
  const { searchParams } = new URL(request.url);
  const wallet_address = searchParams.get('wallet_address');
  
  console.log('📱 Wallet address received:', wallet_address);
  
  return NextResponse.json({
    success: true,
    wallet_address,
    permissions: [],
    user_tier: 'free',
    has_api_access: false,
    message: 'GET method is working!'
  });
}

export async function POST(request: NextRequest) {
  console.log('🔍 SIMPLE POST REQUEST - TESTING ROUTE');
  
  const body = await request.json();
  const { wallet_address } = body;
  
  console.log('📱 Wallet address received:', wallet_address);
  
  return NextResponse.json({
    success: true,
    wallet_address,
    permissions: [],
    user_tier: 'free',
    has_api_access: false,
    message: 'POST method is working!'
  });
}