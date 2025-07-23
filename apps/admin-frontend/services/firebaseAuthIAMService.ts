import { 
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  signOut,
  updateProfile,
  updatePassword,
  deleteUser,
  sendEmailVerification
} from 'firebase/auth';
import type { User } from 'firebase/auth';
import { 
  doc, 
  setDoc, 
  getDoc, 
  updateDoc, 
  deleteDoc,
  collection,
  query,
  where,
  getDocs,
  Timestamp
} from 'firebase/firestore';
import { auth, db } from '../lib/firebase';
// import { firebaseIAMService } from './firebaseIAMService'; // Service removed
import { PackageTier, SubscriptionStatus } from '../types/admin/iam-enhanced';
import type { UserWithPermissions } from '../types/admin/iam-enhanced';

// Placeholder for removed service
const firebaseIAMService = {
  createUser: async (...args: any[]) => {},
  updateUserPackageTier: async (...args: any[]) => {},
  getUser: async (...args: any[]) => null,
  getUserWithPermissions: async (...args: any[]): Promise<UserWithPermissions> => ({
    id: '',
    email: '',
    displayName: '',
    name: '',
    emailVerified: false,
    disabled: false,
    roles: [],
    groups: [],
    attachedPolicies: [],
    status: 'active',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    lastActivity: new Date().toISOString(),
    packageTier: PackageTier.FREE,
    customPermissions: [],
    effectivePermissions: [],
    packagePermissions: [],
    subscriptionStatus: SubscriptionStatus.PENDING
  }),
  applyPackagePermissions: async (...args: any[]) => {},
  setUserCustomPermissions: async (...args: any[]) => {},
  getUserCustomPermissions: async (...args: any[]) => [],
  updateUser: async (...args: any[]) => {},
  updateUserProfile: async (...args: any[]) => {},
  deleteUser: async (...args: any[]) => {},
  getUserEffectivePermissions: async (...args: any[]) => [],
  getUsersCount: async (...args: any[]) => 0,
  createAuditLog: async (...args: any[]) => {},
};

export interface CreateUserRequest {
  email: string;
  password: string;
  name: string;
  packageTier?: PackageTier;
  subscriptionStatus?: SubscriptionStatus;
  roles?: string[];
  groups?: string[];
}

export interface UpdateUserRequest {
  name?: string;
  packageTier?: PackageTier;
  subscriptionStatus?: SubscriptionStatus;
  roles?: string[];
  groups?: string[];
  disabled?: boolean;
}

export class FirebaseAuthIAMService {
  private readonly collections = {
    users: 'users',
    userProfiles: 'user_profiles'
  };

  /**
   * Create a new user with Firebase Auth and IAM profile
   */
  async createUser(userData: CreateUserRequest): Promise<{
    user: User;
    profile: UserWithPermissions;
  }> {
    try {
      // Create user in Firebase Auth
      const userCredential = await createUserWithEmailAndPassword(
        auth, 
        userData.email, 
        userData.password
      );
      
      const user = userCredential.user;

      // Update display name
      await updateProfile(user, {
        displayName: userData.name
      });

      // Send email verification
      await sendEmailVerification(user);

      // Create user profile in Firestore
      const profileData = {
        email: userData.email,
        name: userData.name,
        displayName: userData.name,
        emailVerified: user.emailVerified,
        disabled: false,
        roles: userData.roles || [],
        groups: userData.groups || [],
        attachedPolicies: [],
        status: 'active',
        packageTier: userData.packageTier || PackageTier.FREE,
        subscriptionStatus: userData.subscriptionStatus || SubscriptionStatus.PENDING,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now(),
        lastActivity: Timestamp.now(),
        authUid: user.uid
      };

      // Save to users collection
      await setDoc(doc(db, this.collections.users, user.uid), profileData);

      // Apply default package permissions
      await firebaseIAMService.applyPackagePermissions(user.uid, profileData.packageTier);

      // Create audit log
      await firebaseIAMService.createAuditLog({
        userId: user.uid,
        action: `User created with ${profileData.packageTier} package`,
        resource: 'user:create',
        performedBy: 'SYSTEM',
        timestamp: new Date(),
        metadata: { email: userData.email, packageTier: profileData.packageTier }
      });

      // Get full user profile with permissions
      const fullProfile = await firebaseIAMService.getUserWithPermissions(user.uid);

      return {
        user,
        profile: fullProfile
      };

    } catch (error) {
      console.error('Error creating user:', error);
      throw error;
    }
  }

  /**
   * Sign in user and sync profile
   */
  async signInUser(email: string, password: string): Promise<{
    user: User;
    profile: UserWithPermissions;
  }> {
    try {
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      const user = userCredential.user;

      // Update last activity
      await this.updateLastActivity(user.uid);

      // Get user profile with permissions
      const profile = await firebaseIAMService.getUserWithPermissions(user.uid);

      return {
        user,
        profile
      };

    } catch (error) {
      console.error('Error signing in user:', error);
      throw error;
    }
  }

  /**
   * Sign out current user
   */
  async signOutUser(): Promise<void> {
    try {
      await signOut(auth);
    } catch (error) {
      console.error('Error signing out:', error);
      throw error;
    }
  }

  /**
   * Update user profile
   */
  async updateUser(userId: string, updates: UpdateUserRequest, updatedBy: string): Promise<UserWithPermissions> {
    try {
      const userRef = doc(db, this.collections.users, userId);
      
      // Prepare update data
      const updateData: any = {
        updatedAt: Timestamp.now(),
        updatedBy
      };

      if (updates.name !== undefined) {
        updateData.name = updates.name;
        updateData.displayName = updates.name;
      }
      if (updates.roles !== undefined) updateData.roles = updates.roles;
      if (updates.groups !== undefined) updateData.groups = updates.groups;
      if (updates.disabled !== undefined) updateData.disabled = updates.disabled;
      if (updates.subscriptionStatus !== undefined) updateData.subscriptionStatus = updates.subscriptionStatus;

      // Update Firestore profile
      await updateDoc(userRef, updateData);

      // Handle package tier change
      if (updates.packageTier && updates.packageTier !== undefined) {
        await firebaseIAMService.updateUserPackageTier(userId, updates.packageTier, updatedBy);
      }

      // Update Firebase Auth display name if name changed
      if (updates.name) {
        const user = auth.currentUser;
        if (user && user.uid === userId) {
          await updateProfile(user, { displayName: updates.name });
        }
      }

      // Create audit log
      await firebaseIAMService.createAuditLog({
        userId,
        action: 'User profile updated',
        resource: 'user:update',
        performedBy: updatedBy,
        timestamp: new Date(),
        metadata: updates
      });

      // Return updated profile
      return await firebaseIAMService.getUserWithPermissions(userId);

    } catch (error) {
      console.error('Error updating user:', error);
      throw error;
    }
  }

  /**
   * Delete user (Auth + Firestore)
   */
  async deleteUser(userId: string, deletedBy: string): Promise<void> {
    try {
      // Get user profile first for audit
      const profile = await firebaseIAMService.getUserWithPermissions(userId);

      // Delete from Firestore first
      await deleteDoc(doc(db, this.collections.users, userId));

      // Clean up IAM permissions
      // Note: In production, you'd want to clean up all related IAM data
      
      // Delete from Firebase Auth (requires the user to be signed in)
      const user = auth.currentUser;
      if (user && user.uid === userId) {
        await deleteUser(user);
      }

      // Create audit log
      await firebaseIAMService.createAuditLog({
        userId,
        action: 'User deleted',
        resource: 'user:delete',
        performedBy: deletedBy,
        timestamp: new Date(),
        metadata: { email: profile.email, name: profile.name }
      });

    } catch (error) {
      console.error('Error deleting user:', error);
      throw error;
    }
  }

  /**
   * Update user password
   */
  async updateUserPassword(newPassword: string): Promise<void> {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('No authenticated user');
      }

      await updatePassword(user, newPassword);

      // Create audit log
      await firebaseIAMService.createAuditLog({
        userId: user.uid,
        action: 'Password updated',
        resource: 'user:password',
        performedBy: user.uid,
        timestamp: new Date()
      });

    } catch (error) {
      console.error('Error updating password:', error);
      throw error;
    }
  }

  /**
   * Get current authenticated user with full IAM profile
   */
  async getCurrentUserProfile(): Promise<UserWithPermissions | null> {
    try {
      const user = auth.currentUser;
      if (!user) return null;

      await this.updateLastActivity(user.uid);
      return await firebaseIAMService.getUserWithPermissions(user.uid);

    } catch (error) {
      console.error('Error getting current user profile:', error);
      return null;
    }
  }

  /**
   * Search users by email or name
   */
  async searchUsers(searchTerm: string): Promise<UserWithPermissions[]> {
    try {
      const usersRef = collection(db, this.collections.users);
      
      // Search by email
      const emailQuery = query(usersRef, where('email', '>=', searchTerm), where('email', '<=', searchTerm + '\uf8ff'));
      const emailSnapshot = await getDocs(emailQuery);
      
      // Search by name
      const nameQuery = query(usersRef, where('name', '>=', searchTerm), where('name', '<=', searchTerm + '\uf8ff'));
      const nameSnapshot = await getDocs(nameQuery);
      
      const userIds = new Set<string>();
      emailSnapshot.forEach(doc => userIds.add(doc.id));
      nameSnapshot.forEach(doc => userIds.add(doc.id));
      
      const users: UserWithPermissions[] = [];
      for (const userId of userIds) {
        try {
          const user = await firebaseIAMService.getUserWithPermissions(userId);
          users.push(user);
        } catch (error) {
          console.warn(`Failed to get user ${userId}:`, error);
        }
      }
      
      return users;

    } catch (error) {
      console.error('Error searching users:', error);
      throw error;
    }
  }

  /**
   * Disable/Enable user account
   */
  async setUserDisabled(userId: string, disabled: boolean, updatedBy: string): Promise<void> {
    try {
      await updateDoc(doc(db, this.collections.users, userId), {
        disabled,
        updatedAt: Timestamp.now(),
        updatedBy
      });

      // Create audit log
      await firebaseIAMService.createAuditLog({
        userId,
        action: disabled ? 'User disabled' : 'User enabled',
        resource: 'user:status',
        performedBy: updatedBy,
        timestamp: new Date(),
        metadata: { disabled }
      });

    } catch (error) {
      console.error('Error updating user disabled status:', error);
      throw error;
    }
  }

  /**
   * Sync Firebase Auth user with Firestore profile
   */
  async syncUserProfile(user: User): Promise<UserWithPermissions> {
    try {
      const userRef = doc(db, this.collections.users, user.uid);
      const userDoc = await getDoc(userRef);

      if (!userDoc.exists()) {
        // Create profile if it doesn't exist
        const profileData = {
          email: user.email || '',
          name: user.displayName || '',
          displayName: user.displayName || '',
          emailVerified: user.emailVerified,
          disabled: false,
          roles: [],
          groups: [],
          attachedPolicies: [],
          status: 'active',
          packageTier: PackageTier.FREE,
          subscriptionStatus: SubscriptionStatus.PENDING,
          createdAt: Timestamp.now(),
          updatedAt: Timestamp.now(),
          lastActivity: Timestamp.now(),
          authUid: user.uid
        };

        await setDoc(userRef, profileData);
        await firebaseIAMService.applyPackagePermissions(user.uid, PackageTier.FREE);
      } else {
        // Update existing profile with latest auth info
        await updateDoc(userRef, {
          emailVerified: user.emailVerified,
          lastActivity: Timestamp.now()
        });
      }

      return await firebaseIAMService.getUserWithPermissions(user.uid);

    } catch (error) {
      console.error('Error syncing user profile:', error);
      throw error;
    }
  }

  /**
   * Update last activity timestamp
   */
  private async updateLastActivity(userId: string): Promise<void> {
    try {
      await updateDoc(doc(db, this.collections.users, userId), {
        lastActivity: Timestamp.now()
      });
    } catch (error) {
      console.warn('Failed to update last activity:', error);
      // Don't throw - this is not critical
    }
  }

  /**
   * Upgrade user package and handle payment integration
   */
  async upgradeUserPackage(
    userId: string, 
    newTier: PackageTier, 
    transactionId: string,
    performedBy: string
  ): Promise<void> {
    try {
      // Update package tier
      await firebaseIAMService.updateUserPackageTier(userId, newTier, performedBy);

      // Update subscription status
      await updateDoc(doc(db, this.collections.users, userId), {
        subscriptionStatus: SubscriptionStatus.ACTIVE,
        lastPaymentDate: Timestamp.now(),
        updatedAt: Timestamp.now()
      });

      // Create audit log with transaction details
      await firebaseIAMService.createAuditLog({
        userId,
        action: `Package upgraded to ${newTier}`,
        resource: 'user:package_upgrade',
        performedBy,
        timestamp: new Date(),
        metadata: { newTier, transactionId }
      });

    } catch (error) {
      console.error('Error upgrading user package:', error);
      throw error;
    }
  }
}

export const firebaseAuthIAMService = new FirebaseAuthIAMService();
