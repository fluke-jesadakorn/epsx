import { NextRequest, NextResponse } from 'next/server';
import { getUserStats } from '@/app/actions/admin-server';

export async function GET(request: NextRequest) {
  try {
    const stats = await getUserStats();
    return NextResponse.json(stats);
  } catch (error) {
    console.error('API route error:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to fetch user statistics' },
      { status: 500 }
    );
  }
}