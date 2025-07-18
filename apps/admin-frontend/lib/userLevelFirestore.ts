import { getFirestoreAdmin } from '@/lib/firebase-admin';
import { USER_LEVEL_CONFIGS, UserLevel } from '@/types/admin/userLevels';

export async function ensureUserLevelInFirestore(uid: string, defaultLevel: UserLevel = UserLevel.BRONZE) {
  try {
    const db = getFirestoreAdmin();
    const userDocRef = db.collection('users').doc(uid);
    const userDoc = await userDocRef.get();

    if (!userDoc.exists) {
      // Create a new user document with default level
      const levelConfig = USER_LEVEL_CONFIGS[defaultLevel];
      const userData = {
        userLevel: defaultLevel,
        numericLevel: levelConfig.priority,
        levelAssignedBy: 'system',
        levelAssignedAt: new Date().toISOString(),
        levelUpdateReason: 'New user default level',
        maxTokens: levelConfig.maxTokens,
        tokenMultiplier: levelConfig.tokenMultiplier,
        lastUpdated: new Date().toISOString(),
        createdAt: new Date().toISOString(),
      };

      await userDocRef.set(userData, { merge: true });
      console.log(`Created user level document for ${uid} with level ${defaultLevel}`);
      return userData;
    }

    // If document exists but doesn't have user level, add it
    const userData = userDoc.data();
    if (!userData?.userLevel) {
      const levelConfig = USER_LEVEL_CONFIGS[defaultLevel];
      const updateData = {
        userLevel: defaultLevel,
        numericLevel: levelConfig.priority,
        levelAssignedBy: 'system',
        levelAssignedAt: new Date().toISOString(),
        levelUpdateReason: 'Added missing user level',
        maxTokens: levelConfig.maxTokens,
        tokenMultiplier: levelConfig.tokenMultiplier,
        lastUpdated: new Date().toISOString(),
      };

      await userDocRef.update(updateData);
      console.log(`Added user level to existing document for ${uid} with level ${defaultLevel}`);
      return { ...userData, ...updateData };
    }

    return userData;
  } catch (error) {
    console.error(`Failed to ensure user level in Firestore for ${uid}:`, error);
    throw error;
  }
}

export async function getUserLevelFromFirestore(uid: string): Promise<{
  userLevel: UserLevel;
  numericLevel: number;
  levelAssignedBy?: string;
  levelAssignedAt?: string;
  levelUpdateReason?: string;
  maxTokens?: number;
  tokenMultiplier?: number;
  lastUpdated?: string;
} | null> {
  try {
    const db = getFirestoreAdmin();
    const userDocRef = db.collection('users').doc(uid);
    const userDoc = await userDocRef.get();

    if (!userDoc.exists) {
      return null;
    }

    const userData = userDoc.data();
    return {
      userLevel: userData?.userLevel || UserLevel.BRONZE,
      numericLevel: userData?.numericLevel || 0,
      levelAssignedBy: userData?.levelAssignedBy,
      levelAssignedAt: userData?.levelAssignedAt,
      levelUpdateReason: userData?.levelUpdateReason,
      maxTokens: userData?.maxTokens,
      tokenMultiplier: userData?.tokenMultiplier,
      lastUpdated: userData?.lastUpdated,
    };
  } catch (error) {
    console.error(`Failed to get user level from Firestore for ${uid}:`, error);
    return null;
  }
}
