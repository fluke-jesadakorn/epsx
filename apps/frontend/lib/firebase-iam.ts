import { initializeApp, getApps, getApp } from 'firebase/app';
import { getAuth, signInWithEmailAndPassword, createUserWithEmailAndPassword, signOut, onAuthStateChanged, type User } from 'firebase/auth';
import { getFirestore, doc, getDoc, setDoc, updateDoc, collection, query, where, getDocs } from 'firebase/firestore';
import { getStorage } from 'firebase/storage';

// Firebase configuration
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY || '',
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN || '',
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID || '',
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET || '',
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID || '',
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID || '',
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID || '',
};

// Initialize Firebase
const app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApp();
export const auth = getAuth(app);
export const db = getFirestore(app);
export const storage = getStorage(app);

// IAM Types
export interface UserRole {
  name: string;
  permissions: string[];
  description?: string;
}

export interface UserPermissions {
  uid: string;
  email: string;
  role: string;
  permissions: string[];
  customPermissions?: string[];
  createdAt: Date;
  updatedAt: Date;
}

export interface PermissionCheck {
  hasPermission: boolean;
  role: string;
  permissions: string[];
}

// Default roles and permissions
export const DEFAULT_ROLES: Record<string, UserRole> = {
  user: {
    name: 'user',
    permissions: ['read:own_data', 'write:own_data', 'read:public_content'],
    description: 'Basic user with access to own data and public content'
  },
  premium_user: {
    name: 'premium_user',
    permissions: [
      'read:own_data', 'write:own_data', 'read:public_content', 
      'read:premium_content', 'write:premium_content'
    ],
    description: 'Premium user with access to premium content'
  },
  moderator: {
    name: 'moderator',
    permissions: [
      'read:all', 'write:moderated', 'moderate:content', 
      'read:public_content', 'write:public_content'
    ],
    description: 'Moderator with content moderation capabilities'
  },
  admin: {
    name: 'admin',
    permissions: ['read:all', 'write:all', 'admin:access', 'manage:users', 'manage:roles'],
    description: 'Full administrative access'
  },
  super_admin: {
    name: 'super_admin',
    permissions: ['*'], // All permissions
    description: 'Super administrator with all system permissions'
  }
};

// IAM Service
export class FirebaseIAMService {
  private static instance: FirebaseIAMService;

  static getInstance(): FirebaseIAMService {
    if (!FirebaseIAMService.instance) {
      FirebaseIAMService.instance = new FirebaseIAMService();
    }
    return FirebaseIAMService.instance;
  }

  // Get user permissions from Firestore
  async getUserPermissions(uid: string): Promise<UserPermissions | null> {
    try {
      const userDoc = await getDoc(doc(db, 'user_permissions', uid));
      if (userDoc.exists()) {
        return userDoc.data() as UserPermissions;
      }
      return null;
    } catch (error) {
      console.error('Error getting user permissions:', error);
      return null;
    }
  }

  // Set user permissions
  async setUserPermissions(uid: string, role: string, customPermissions?: string[]): Promise<void> {
    try {
      const roleConfig = DEFAULT_ROLES[role];
      if (!roleConfig) {
        throw new Error(`Invalid role: ${role}`);
      }

      const permissions: UserPermissions = {
        uid,
        email: auth.currentUser?.email || '',
        role,
        permissions: [...roleConfig.permissions, ...(customPermissions || [])],
        customPermissions: customPermissions || [],
        createdAt: new Date(),
        updatedAt: new Date()
      };

      await setDoc(doc(db, 'user_permissions', uid), permissions);
    } catch (error) {
      console.error('Error setting user permissions:', error);
      throw error;
    }
  }

  // Update user role
  async updateUserRole(uid: string, newRole: string, customPermissions?: string[]): Promise<void> {
    try {
      const roleConfig = DEFAULT_ROLES[newRole];
      if (!roleConfig) {
        throw new Error(`Invalid role: ${newRole}`);
      }

      const updateData = {
        role: newRole,
        permissions: [...roleConfig.permissions, ...(customPermissions || [])],
        customPermissions: customPermissions || [],
        updatedAt: new Date()
      };

      await updateDoc(doc(db, 'user_permissions', uid), updateData);
    } catch (error) {
      console.error('Error updating user role:', error);
      throw error;
    }
  }

  // Check if user has specific permission
  async checkPermission(uid: string, permission: string): Promise<PermissionCheck> {
    try {
      const userPermissions = await this.getUserPermissions(uid);
      if (!userPermissions) {
        return {
          hasPermission: false,
          role: 'none',
          permissions: []
        };
      }

      // Check for wildcard permission
      if (userPermissions.permissions.includes('*')) {
        return {
          hasPermission: true,
          role: userPermissions.role,
          permissions: userPermissions.permissions
        };
      }

      // Check for specific permission
      const hasPermission = userPermissions.permissions.includes(permission);
      return {
        hasPermission,
        role: userPermissions.role,
        permissions: userPermissions.permissions
      };
    } catch (error) {
      console.error('Error checking permission:', error);
      return {
        hasPermission: false,
        role: 'error',
        permissions: []
      };
    }
  }

  // Check multiple permissions
  async checkPermissions(uid: string, permissions: string[]): Promise<Record<string, boolean>> {
    const results: Record<string, boolean> = {};
    
    for (const permission of permissions) {
      const check = await this.checkPermission(uid, permission);
      results[permission] = check.hasPermission;
    }
    
    return results;
  }

  // Get all users with a specific role
  async getUsersByRole(role: string): Promise<UserPermissions[]> {
    try {
      const q = query(collection(db, 'user_permissions'), where('role', '==', role));
      const querySnapshot = await getDocs(q);
      
      return querySnapshot.docs.map(doc => doc.data() as UserPermissions);
    } catch (error) {
      console.error('Error getting users by role:', error);
      return [];
    }
  }

  // Initialize user permissions on registration
  async initializeUserPermissions(user: User, role: string = 'user'): Promise<void> {
    try {
      await this.setUserPermissions(user.uid, role);
    } catch (error) {
      console.error('Error initializing user permissions:', error);
      throw error;
    }
  }

  // Get available roles
  getAvailableRoles(): UserRole[] {
    return Object.values(DEFAULT_ROLES);
  }

  // Get role by name
  getRole(roleName: string): UserRole | undefined {
    return DEFAULT_ROLES[roleName];
  }

  // Validate role exists
  isValidRole(roleName: string): boolean {
    return roleName in DEFAULT_ROLES;
  }
}

// Auth service with IAM integration
export class FirebaseAuthService {
  private iamService: FirebaseIAMService;

  constructor() {
    this.iamService = FirebaseIAMService.getInstance();
  }

  // Sign in with email and password
  async signIn(email: string, password: string): Promise<{ user: User; permissions: UserPermissions }> {
    try {
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      const permissions = await this.iamService.getUserPermissions(userCredential.user.uid);
      
      if (!permissions) {
        // Initialize default permissions if not exists
        await this.iamService.initializeUserPermissions(userCredential.user);
        const newPermissions = await this.iamService.getUserPermissions(userCredential.user.uid);
        return { user: userCredential.user, permissions: newPermissions! };
      }
      
      return { user: userCredential.user, permissions };
    } catch (error) {
      console.error('Sign in error:', error);
      throw error;
    }
  }

  // Sign up with email and password
  async signUp(email: string, password: string, role: string = 'user'): Promise<{ user: User; permissions: UserPermissions }> {
    try {
      const userCredential = await createUserWithEmailAndPassword(auth, email, password);
      await this.iamService.initializeUserPermissions(userCredential.user, role);
      
      const permissions = await this.iamService.getUserPermissions(userCredential.user.uid);
      return { user: userCredential.user, permissions: permissions! };
    } catch (error) {
      console.error('Sign up error:', error);
      throw error;
    }
  }

  // Sign out
  async signOut(): Promise<void> {
    try {
      await signOut(auth);
    } catch (error) {
      console.error('Sign out error:', error);
      throw error;
    }
  }

  // Get current user
  getCurrentUser(): User | null {
    return auth.currentUser;
  }

  // Listen for auth state changes
  onAuthStateChanged(callback: (user: User | null) => void): () => void {
    return onAuthStateChanged(auth, callback);
  }

  // Get current user permissions
  async getCurrentUserPermissions(): Promise<UserPermissions | null> {
    const user = this.getCurrentUser();
    if (!user) return null;
    
    return this.iamService.getUserPermissions(user.uid);
  }
}

// Export singleton instances
export const iamService = FirebaseIAMService.getInstance();
export const authService = new FirebaseAuthService();

// React hooks for IAM
export const useIAMServices = () => {
  return {
    iamService,
    authService,
    DEFAULT_ROLES
  };
};

// Permission checking utilities
export const hasPermission = async (uid: string, permission: string): Promise<boolean> => {
  const check = await iamService.checkPermission(uid, permission);
  return check.hasPermission;
};

export const hasAnyPermission = async (uid: string, permissions: string[]): Promise<boolean> => {
  const checks = await iamService.checkPermissions(uid, permissions);
  return Object.values(checks).some(hasPermission => hasPermission);
};

export const hasAllPermissions = async (uid: string, permissions: string[]): Promise<boolean> => {
  const checks = await iamService.checkPermissions(uid, permissions);
  return Object.values(checks).every(hasPermission => hasPermission);
};

// Role checking utilities
export const hasRole = async (uid: string, role: string): Promise<boolean> => {
  const permissions = await iamService.getUserPermissions(uid);
  return permissions?.role === role;
};

export const hasAnyRole = async (uid: string, roles: string[]): Promise<boolean> => {
  const permissions = await iamService.getUserPermissions(uid);
  return permissions ? roles.includes(permissions.role) : false;
};

export default {
  iamService,
  authService,
  DEFAULT_ROLES,
  hasPermission,
  hasAnyPermission,
  hasAllPermissions,
  hasRole,
  hasAnyRole
};
