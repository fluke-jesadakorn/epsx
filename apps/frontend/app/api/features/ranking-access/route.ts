import { checkRankingAccess } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const result = await checkRankingAccess();
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/features/ranking-access:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Ranking access check failed' },
      { status: 500 }
    );
  }
}