import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';
import { USER_LEVEL_CONFIGS, type UserLevel } from '@/types/admin/userLevels';

// Interface for server-side operations
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
}

interface UserListResult {
  users: AdminUser[];
  pageToken?: string;
}

interface UpdateUserLevelData {
  userLevel: UserLevel;
  reason?: string;
}

export async function GET(request: NextRequest) {
  try {
    const auth = getAuthAdmin();
    const { searchParams } = new URL(request.url);
    const maxResults = parseInt(searchParams.get('maxResults') || '100');
    const pageToken = searchParams.get('pageToken') || undefined;

    const result = await auth.listUsers(maxResults, pageToken);
    
    const users: AdminUser[] = result.users.map((userRecord) => ({
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
    }));

    const response: UserListResult = {
      users,
      pageToken: result.pageToken,
    };

    return NextResponse.json(response);
  } catch (error: any) {
    console.error('Failed to list users:', error);
    return NextResponse.json(
      { error: error.message || 'Failed to fetch users' },
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
        // Get current user claims
        const currentUserForLevel = await auth.getUser(uid);
        const currentClaims = currentUserForLevel.customClaims || {};
        const levelData = data as UpdateUserLevelData;
        const levelConfig = USER_LEVEL_CONFIGS[levelData.userLevel];
        
        const updatedClaimsForLevel = {
          ...currentClaims,
          userLevel: levelData.userLevel,
          maxTokens: levelConfig.maxTokens,
          levelAssignedBy: 'admin', // Should be the actual admin ID
          levelAssignedAt: new Date().toISOString(),
          lastUpdated: Date.now(),
        };

        await auth.setCustomUserClaims(uid, updatedClaimsForLevel);
        
        // Log the level assignment (you might want to store this in a database)
        console.log(`User level updated: ${uid} -> ${levelData.userLevel}`, {
          reason: levelData.reason,
          assignedBy: 'admin',
          assignedAt: new Date().toISOString()
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
