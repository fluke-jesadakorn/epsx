import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin, getFirestoreAdmin } from '@/lib/firebase-admin';
import { ensureUserLevelInFirestore } from '@/lib/userLevelFirestore';
import type { UserLevel } from '@/types/admin/userLevels';

interface AdminUser {
  uid: string;
  email: string;
  emailVerified: boolean;
  displayName?: string;
  disabled: boolean;
  customClaims?: any;
  metadata: {
    creationTime?: string;
    lastSignInTime?: string;
    lastRefreshTime?: string | null;
  };
  // User level data from Firestore
  userLevel?: UserLevel;
  numericLevel?: number;
  levelAssignedBy?: string;
  levelAssignedAt?: string;
  levelUpdateReason?: string;
  maxTokens?: number;
  tokenMultiplier?: number;
  lastUpdated?: string;
}

export async function GET(
  _request: NextRequest,
  { params }: { params: { uid: string } }
) {
  try {
    const auth = getAuthAdmin();
    const db = getFirestoreAdmin();
    const { uid } = params;

    const userRecord = await auth.getUser(uid);
    
    // Get user data from Firestore
    const userDocRef = db.collection('users').doc(uid);
    const userDoc = await userDocRef.get();
    let firestoreData = userDoc.exists ? userDoc.data() : {};
    
    // If no user level in Firestore, create default entry
    if (!firestoreData?.userLevel) {
      const userData = await ensureUserLevelInFirestore(uid);
      firestoreData = { ...firestoreData, ...userData };
    }
    
    const user: AdminUser = {
      uid: userRecord.uid,
      email: userRecord.email || '',
      emailVerified: userRecord.emailVerified,
      displayName: userRecord.displayName,
      disabled: userRecord.disabled,
      customClaims: userRecord.customClaims,
      metadata: {
        creationTime: userRecord.metadata.creationTime,
        lastSignInTime: userRecord.metadata.lastSignInTime,
        lastRefreshTime: userRecord.metadata.lastRefreshTime,
      },
      
      // Firestore user level data
      userLevel: firestoreData.userLevel || 'BRONZE',
      numericLevel: firestoreData.numericLevel || 0,
      levelAssignedBy: firestoreData.levelAssignedBy || 'system',
      levelAssignedAt: firestoreData.levelAssignedAt || userRecord.metadata.creationTime,
      levelUpdateReason: firestoreData.levelUpdateReason || 'Default assignment',
      maxTokens: firestoreData.maxTokens || 1000,
      tokenMultiplier: firestoreData.tokenMultiplier || 1,
      lastUpdated: firestoreData.lastUpdated || userRecord.metadata.creationTime,
    };

    return NextResponse.json(user);
  } catch (error: any) {
    console.error(`Failed to get user ${params.uid}:`, error);
    return NextResponse.json(
      { error: error.message || 'Failed to fetch user' },
      { status: 500 }
    );
  }
}
