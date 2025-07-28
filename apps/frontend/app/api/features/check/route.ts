import { checkFeatureAccess } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { featureId } = body;

    if (!featureId) {
      return NextResponse.json(
        { error: 'Feature ID is required' },
        { status: 400 }
      );
    }

    const result = await checkFeatureAccess(featureId);
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/features/check:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Feature check failed' },
      { status: 500 }
    );
  }
}