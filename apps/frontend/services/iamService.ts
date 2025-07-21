import { initializeApp, getApps, getApp } from 'firebase/app';
import { 
  getAuth, 
  signInWithEmailAndPassword, 
  signOut, 
  onAuthStateChanged,
  type User as FirebaseUser,
  createUserWithEmailAndPassword,
  sendPasswordResetEmail,
  updateProfile
} from 'firebase/auth';
import { 
  getFirestore, 
  doc, 
  getDoc, 
  setDoc, 
  updateDoc, 
  collection, 
  getDocs,
  serverTimestamp
} from 'firebase/firestore';
import { 
  getStorage
} from 'firebase/storage';
import { 
  DEFAULT_ROLES, 
  DEFAULT_PERMISSIONS 
} from '../config/iam/default-roles';
import type { PermissionDocument, RoleDocument } from '../config/iam/types';

// Firebase configuration
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
};

// Initialize Firebase
const app = !getApps().length ? initializeApp(firebaseConfig) : getApp();
const auth = getAuth(app);
const db = getFirestore(app);
const storage = getStorage(app);

// Types
export interface User {
  uid: string;
  email: string;
  displayName?: string;
  photoURL?: string;
  role: string;
  permissions: string[];
  packagePermissions: string[];
  companyId?: string;
  partnerId?: string;
  isActive: boolean;
  lastLoginAt?: Date;
  createdAt: Date;
  updatedAt: Date;
  metadata?: Record<string, any>;
}

export interface UserRole {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
  permissions: string[];
  isSystem: boolean;
  isActive: boolean;
}

export interface UserPermission {
  id: string;
  name: string;
  description: string;
  category: string;
  action: string;
  resource: string;
  scope: string;
  isSystem: boolean;
  tags: string[];
}

export interface IAMContext {
  user: User | null;
  role: UserRole | null;
  permissions: UserPermission[];
  loading: boolean;
  error: string | null;
}

// IAM Service
class IAMService {
  private static instance: IAMService;
  private currentUser: User | null = null;
  private currentRole: UserRole | null = null;
  private currentPermissions: UserPermission[] = [];

  public static getInstance(): IAMService {
    if (!IAMService.instance) {
      IAMService.instance = new IAMService();
    }
    return IAMService.instance;
  }

  // Authentication Methods
  async signIn(email: string, password: string): Promise<User> {
    try {
      const userCredential = await signInWithEmailAndPassword(auth, email, password);
      const user = await this.getUser(userCredential.user.uid);
      
      if (!user) {
        throw new Error('User not found in IAM system');
      }

      if (!user.isActive) {
        throw new Error('Account is deactivated');
      }

      // Update last login
      await updateDoc(doc(db, 'users', user.uid), {
        lastLoginAt: serverTimestamp()
      });

      this.currentUser = user;
      await this.loadUserRoleAndPermissions(user.role);
      
      return user;
    } catch (error) {
      console.error('Sign in error:', error);
      throw error;
    }
  }

  async signUp(email: string, password: string, displayName: string): Promise<User> {
    try {
      const userCredential = await createUserWithEmailAndPassword(auth, email, password);
      
      // Create user profile
      const user: User = {
        uid: userCredential.user.uid,
        email,
        displayName,
        role: 'user', // Default role
        permissions: [],
        packagePermissions: [],
        isActive: true,
        createdAt: new Date(),
        updatedAt: new Date()
      };

      await setDoc(doc(db, 'users', user.uid), user);
      
      // Update Firebase Auth profile
      await updateProfile(userCredential.user, { displayName });
      
      this.currentUser = user;
      await this.loadUserRoleAndPermissions(user.role);
      
      return user;
    } catch (error) {
      console.error('Sign up error:', error);
      throw error;
    }
  }

  async signOut(): Promise<void> {
    try {
      await signOut(auth);
      this.currentUser = null;
      this.currentRole = null;
      this.currentPermissions = [];
    } catch (error) {
      console.error('Sign out error:', error);
      throw error;
    }
  }

  async resetPassword(email: string): Promise<void> {
    try {
      await sendPasswordResetEmail(auth, email);
    } catch (error) {
      console.error('Password reset error:', error);
      throw error;
    }
  }

  // User Management
  async getUser(uid: string): Promise<User | null> {
    try {
      const userDoc = await getDoc(doc(db, 'users', uid));
      if (userDoc.exists()) {
        return userDoc.data() as User;
      }
      return null;
    } catch (error) {
      console.error('Get user error:', error);
      throw error;
    }
  }

  async updateUser(uid: string, updates: Partial<User>): Promise<void> {
    try {
      await updateDoc(doc(db, 'users', uid), {
        ...updates,
        updatedAt: serverTimestamp()
      });
      
      if (uid === this.currentUser?.uid) {
        this.currentUser = { ...this.currentUser, ...updates };
      }
    } catch (error) {
      console.error('Update user error:', error);
      throw error;
    }
  }

  async assignRole(uid: string, roleId: string): Promise<void> {
    try {
      await updateDoc(doc(db, 'users', uid), {
        role: roleId,
        updatedAt: serverTimestamp()
      });
      
      if (uid === this.currentUser?.uid) {
        this.currentUser.role = roleId;
        await this.loadUserRoleAndPermissions(roleId);
      }
    } catch (error) {
      console.error('Assign role error:', error);
      throw error;
    }
  }

  // Role Management
  async getRole(roleId: string): Promise<UserRole | null> {
    try {
      // Return null if roleId is undefined or empty
      if (!roleId) {
        console.warn('No role ID provided to getRole method');
        return null;
      }

      const roleDoc = await getDoc(doc(db, 'roles', roleId));
      if (roleDoc.exists()) {
        return roleDoc.data() as UserRole;
      }
      return null;
    } catch (error) {
      console.error('Get role error:', error);
      throw error;
    }
  }

  async getAllRoles(): Promise<UserRole[]> {
    try {
      const rolesSnapshot = await getDocs(collection(db, 'roles'));
      return rolesSnapshot.docs.map(doc => doc.data() as UserRole);
    } catch (error) {
      console.error('Get all roles error:', error);
      throw error;
    }
  }

  async createRole(role: Omit<UserRole, 'id'>): Promise<string> {
    try {
      const roleId = doc(collection(db, 'roles')).id;
      const newRole = { ...role, id: roleId };
      
      await setDoc(doc(db, 'roles', roleId), newRole);
      return roleId;
    } catch (error) {
      console.error('Create role error:', error);
      throw error;
    }
  }

  // Permission Management
  async getPermission(permissionId: string): Promise<UserPermission | null> {
    try {
      const permissionDoc = await getDoc(doc(db, 'permissions', permissionId));
      if (permissionDoc.exists()) {
        return permissionDoc.data() as UserPermission;
      }
      return null;
    } catch (error) {
      console.error('Get permission error:', error);
      throw error;
    }
  }

  async getAllPermissions(): Promise<UserPermission[]> {
    try {
      const permissionsSnapshot = await getDocs(collection(db, 'permissions'));
      return permissionsSnapshot.docs.map(doc => doc.data() as UserPermission);
    } catch (error) {
      console.error('Get all permissions error:', error);
      throw error;
    }
  }

  // Authorization
  hasPermission(permissionId: string): boolean {
    if (!this.currentUser || !this.currentRole) return false;
    
    // Super admin has all permissions
    if (this.currentRole.id === 'super_admin') return true;
    
    // Check role permissions
    if (this.currentRole.permissions.includes('*')) return true;
    if (this.currentRole.permissions.includes(permissionId)) return true;
    
    // Check user-specific permissions
    if (this.currentUser.permissions.includes(permissionId)) return true;
    
    return false;
  }

  hasAnyPermission(permissionIds: string[]): boolean {
    return permissionIds.some(id => this.hasPermission(id));
  }

  hasAllPermissions(permissionIds: string[]): boolean {
    return permissionIds.every(id => this.hasPermission(id));
  }

  getUserPermissions(): UserPermission[] {
    return this.currentPermissions;
  }

  getUserRole(): UserRole | null {
    return this.currentRole;
  }

  getCurrentUser(): User | null {
    return this.currentUser;
  }

  // Private methods
  private async loadUserRoleAndPermissions(roleId: string): Promise<void> {
    try {
      // Skip if roleId is undefined or empty
      if (!roleId) {
        console.warn('No role ID provided, skipping role and permissions loading');
        this.currentRole = null;
        this.currentPermissions = [];
        return;
      }

      const role = await this.getRole(roleId);
      if (role) {
        this.currentRole = role;
        
        // Load permissions for the role
        const permissions = await Promise.all(
          role.permissions
            .filter(permId => permId !== '*')
            .map(permId => this.getPermission(permId))
        );
        
        this.currentPermissions = permissions.filter(Boolean) as UserPermission[];
      }
    } catch (error) {
      console.error('Load user role and permissions error:', error);
    }
  }

  // Initialize IAM system
  async initializeIAM(): Promise<void> {
    try {
      // Check if roles and permissions exist
      const roles = await this.getAllRoles();
      const permissions = await this.getAllPermissions();
      
      if (roles.length === 0) {
        // Seed default roles
        for (const role of DEFAULT_ROLES) {
          await setDoc(doc(db, 'roles', role.id), role);
        }
      }
      
      if (permissions.length === 0) {
        // Seed default permissions
        for (const permission of DEFAULT_PERMISSIONS) {
          await setDoc(doc(db, 'permissions', permission.id), permission);
        }
      }
    } catch (error) {
      console.error('Initialize IAM error:', error);
      throw error;
    }
  }

  // Auth state listener
  onAuthStateChange(callback: (user: User | null) => void): () => void {
    return onAuthStateChanged(auth, async (firebaseUser: FirebaseUser | null) => {
      if (firebaseUser) {
        const user = await this.getUser(firebaseUser.uid);
        if (user) {
          this.currentUser = user;
          await this.loadUserRoleAndPermissions(user.role);
          callback(user);
        } else {
          callback(null);
        }
      } else {
        this.currentUser = null;
        this.currentRole = null;
        this.currentPermissions = [];
        callback(null);
      }
    });
  }
}

// Export singleton instance
export const iamService = IAMService.getInstance();
