import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin, getFirestoreAdmin } from '@/lib/firebase-admin';

export async function GET(
  _request: NextRequest,
  { params }: { params: { uid: string } }
) {
  try {
    const { uid } = params;
    
    // Get user data from Firestore
    const db = getFirestoreAdmin();
    const userDocRef = db.collection('users').doc(uid);
    const userDoc = await userDocRef.get();
    
    if (!userDoc.exists) {
      // Fallback to Firebase Auth data if no Firestore document
      const auth = getAuthAdmin();
      const user = await auth.getUser(uid);
      
      const history = [
        {
          uid,
          userLevel: 'BRONZE',
          assignedBy: 'system',
          assignedAt: new Date(user.metadata.creationTime),
          reason: 'Default level',
          previousLevel: null
        }
      ];
      return NextResponse.json(history);
    }
    
    const userData = userDoc.data();
    const currentLevel = userData?.userLevel || 'BRONZE';
    
    const history = [
      {
        uid,
        userLevel: currentLevel,
        assignedBy: userData?.levelAssignedBy || 'system',
        assignedAt: new Date(userData?.levelAssignedAt || userData?.lastUpdated || Date.now()),
        reason: userData?.levelUpdateReason || 'Current level',
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
