import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin, getFirestoreAdmin } from '@/lib/firebase-admin';
import { USER_LEVEL_CONFIGS, type UserLevel } from '@/types/admin/userLevels';
import { ensureUserLevelInFirestore } from '@/lib/userLevelFirestore';

interface UpdateUserLevelData {
  userLevel: UserLevel;
  reason?: string;
}

export async function GET() {
  try {
    const auth = getAuthAdmin();
    const db = getFirestoreAdmin();
    
    // Get all users from Firebase Auth
    const listUsersResult = await auth.listUsers();
    const users = [];
    
    for (const userRecord of listUsersResult.users) {
      try {
        // Get user data from Firestore
        const userDocRef = db.collection('users').doc(userRecord.uid);
        const userDoc = await userDocRef.get();
        
        let firestoreData = userDoc.exists ? userDoc.data() : {};
        
        // If no user level in Firestore, create default entry
        if (!firestoreData?.userLevel) {
          const userData = await ensureUserLevelInFirestore(userRecord.uid);
          firestoreData = { ...firestoreData, ...userData };
        }
        
        // Merge Firebase Auth data with Firestore data
        const userData = {
          uid: userRecord.uid,
          email: userRecord.email || '',
          emailVerified: userRecord.emailVerified,
          displayName: userRecord.displayName,
          disabled: userRecord.disabled,
          customClaims: userRecord.customClaims || {},
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
        
        users.push(userData);
      } catch (error) {
        console.error(`Failed to fetch Firestore data for user ${userRecord.uid}:`, error);
        // Fallback to Firebase Auth data only
        users.push({
          uid: userRecord.uid,
          email: userRecord.email || '',
          emailVerified: userRecord.emailVerified,
          displayName: userRecord.displayName,
          disabled: userRecord.disabled,
          customClaims: userRecord.customClaims || {},
          metadata: {
            creationTime: userRecord.metadata.creationTime,
            lastSignInTime: userRecord.metadata.lastSignInTime,
            lastRefreshTime: userRecord.metadata.lastRefreshTime,
          },
          userLevel: 'BRONZE',
          numericLevel: 0,
        });
      }
    }
    
    return NextResponse.json({ users });
  } catch (error) {
    console.error('Failed to fetch users:', error);
    return NextResponse.json(
      { error: 'Failed to fetch users' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const auth = getAuthAdmin();
    const body = await request.json();
    const { action, uid, data } = body;

    switch (action) {
      case 'updateRole':
        // Get current user claims and update role
        const currentUser = await auth.getUser(uid);
        const updatedClaims = {
          ...currentUser.customClaims,
          role: data.role,
          lastUpdated: Date.now(),
        };
        await auth.setCustomUserClaims(uid, updatedClaims);
        return NextResponse.json({ success: true });

      case 'updateUserLevel':
        // Get level data
        const levelData = data as UpdateUserLevelData;
        const levelConfig = USER_LEVEL_CONFIGS[levelData.userLevel];
        
        // Update user level in Firestore instead of custom claims
        const db = getFirestoreAdmin();
        const userDocRef = db.collection('users').doc(uid);
        const userDoc = await userDocRef.get();
        const existingData = userDoc.exists ? userDoc.data() : {};
        
        const userLevelData = {
          ...existingData,
          // User level information
          userLevel: levelData.userLevel,
          numericLevel: levelConfig.priority || 1,
          
          // Level assignment metadata
          levelAssignedBy: 'admin', // Should be the actual admin ID
          levelAssignedAt: new Date().toISOString(),
          levelUpdateReason: levelData.reason || 'Individual assignment',
          
          // Level configuration data
          maxTokens: levelConfig.maxTokens,
          tokenMultiplier: levelConfig.tokenMultiplier,
          
          // Update metadata
          lastUpdated: new Date().toISOString(),
        };

        await userDocRef.set(userLevelData, { merge: true });
        
        // Log the level assignment
        console.log(`User level updated: ${uid} -> ${levelData.userLevel}`, {
          reason: levelData.reason,
          assignedBy: 'admin',
          assignedAt: new Date().toISOString(),
          storage: 'firestore'
        });
        
        return NextResponse.json({ success: true });

      case 'updateStatus':
        await auth.updateUser(uid, { disabled: data.disabled });
        return NextResponse.json({ success: true });

      case 'deleteUser':
        await auth.deleteUser(uid);
        return NextResponse.json({ success: true });

      case 'sendPasswordReset':
        const link = await auth.generatePasswordResetLink(data.email);
        console.log('Password reset link generated:', link);
        return NextResponse.json({ success: true, link });

      default:
        return NextResponse.json(
          { error: 'Invalid action' },
          { status: 400 }
        );
    }
  } catch (error: any) {
    console.error('User management operation failed:', error);
    return NextResponse.json(
      { error: error.message || 'Operation failed' },
      { status: 500 }
    );
  }
}
