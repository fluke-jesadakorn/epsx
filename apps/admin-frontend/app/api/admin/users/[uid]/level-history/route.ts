import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';

export async function GET(
  _request: NextRequest,
  { params }: { params: { uid: string } }
) {
  try {
    const { uid } = params;
    
    // In a real implementation, you would fetch from a database
    // For now, we'll return mock data based on the user's current level
    const auth = getAuthAdmin();
    const user = await auth.getUser(uid);
    const currentLevel = user.customClaims?.userLevel || 'BRONZE';
    
    const history = [
      {
        uid,
        userLevel: currentLevel,
        assignedBy: user.customClaims?.levelAssignedBy || 'system',
        assignedAt: new Date(user.customClaims?.levelAssignedAt || user.metadata.creationTime || Date.now()),
        reason: 'Current level',
        previousLevel: null
      }
    ];

    return NextResponse.json(history);
  } catch (error) {
    console.error('Failed to fetch user level history:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
