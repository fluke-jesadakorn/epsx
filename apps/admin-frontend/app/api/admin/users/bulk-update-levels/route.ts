import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';
import { USER_LEVEL_CONFIGS } from '@/types/admin/userLevels';

export async function POST(request: NextRequest) {
  try {
    const auth = getAuthAdmin();
    const body = await request.json();
    const { updates } = body;

    if (!Array.isArray(updates)) {
      return NextResponse.json(
        { error: 'Updates must be an array' },
        { status: 400 }
      );
    }

    const results = [];
    
    for (const update of updates) {
      try {
        const { uid, userLevel, reason } = update;
        
        // Determine if uid is actually an email or a UID
        let actualUid = uid;
        let currentUser;
        
        if (uid.includes('@')) {
          // It's an email, need to get the user by email first
          try {
            currentUser = await auth.getUserByEmail(uid);
            actualUid = currentUser.uid;
          } catch (emailError) {
            throw new Error(`User not found with email: ${uid}`);
          }
        } else {
          // It's a UID, get user directly
          currentUser = await auth.getUser(uid);
        }
        
        const currentClaims = currentUser.customClaims || {};
        const levelConfig = USER_LEVEL_CONFIGS[userLevel as keyof typeof USER_LEVEL_CONFIGS];
        
        const updatedClaims = {
          ...currentClaims,
          userLevel: userLevel,
          maxTokens: levelConfig.maxTokens,
          levelAssignedBy: 'admin', // Should be the actual admin ID
          levelAssignedAt: new Date().toISOString(),
          lastUpdated: Date.now(),
        };

        await auth.setCustomUserClaims(actualUid, updatedClaims);
        
        results.push({ 
          uid: actualUid, 
          email: currentUser.email,
          success: true,
          userLevel,
          reason
        });
        
        // Log the level assignment
        console.log(`Bulk user level updated: ${actualUid} (${currentUser.email}) -> ${userLevel}`, {
          reason: reason || 'Bulk assignment',
          assignedBy: 'admin',
          assignedAt: new Date().toISOString()
        });
        
      } catch (error: any) {
        console.error(`Failed to update user ${update.uid}:`, error);
        results.push({ 
          uid: update.uid, 
          success: false, 
          error: error.message 
        });
      }
    }

    return NextResponse.json({ results });
  } catch (error: any) {
    console.error('Bulk update failed:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
