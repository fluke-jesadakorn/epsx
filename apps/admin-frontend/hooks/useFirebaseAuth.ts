import { useState, useEffect } from 'react';
import { onAuthStateChanged } from 'firebase/auth';
import type { User } from 'firebase/auth';
import { auth } from '../lib/firebase';
import { firebaseAuthIAMService } from '../services/firebaseAuthIAMService';
import type { UserWithPermissions } from '../types/admin/iam-enhanced';

export interface AuthState {
  user: User | null;
  profile: UserWithPermissions | null;
  loading: boolean;
  error: string | null;
}

export const useFirebaseAuth = () => {
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    profile: null,
    loading: true,
    error: null
  });

  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      try {
        setAuthState(prev => ({ ...prev, loading: true, error: null }));

        if (user) {
          // User is signed in, get their full profile
          const profile = await firebaseAuthIAMService.syncUserProfile(user);
          setAuthState({
            user,
            profile,
            loading: false,
            error: null
          });
        } else {
          // User is signed out
          setAuthState({
            user: null,
            profile: null,
            loading: false,
            error: null
          });
        }
      } catch (error) {
        console.error('Auth state change error:', error);
        setAuthState({
          user,
          profile: null,
          loading: false,
          error: error instanceof Error ? error.message : 'Authentication error'
        });
      }
    });

    return () => unsubscribe();
  }, []);

  const signIn = async (email: string, password: string) => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));
      const result = await firebaseAuthIAMService.signInUser(email, password);
      
      setAuthState({
        user: result.user,
        profile: result.profile,
        loading: false,
        error: null
      });
      
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Sign in failed';
      setAuthState(prev => ({ 
        ...prev, 
        loading: false, 
        error: errorMessage 
      }));
      throw error;
    }
  };

  const signOut = async () => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));
      await firebaseAuthIAMService.signOutUser();
      // Auth state will be updated by the listener
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Sign out failed';
      setAuthState(prev => ({ 
        ...prev, 
        loading: false, 
        error: errorMessage 
      }));
      throw error;
    }
  };

  const createUser = async (userData: Parameters<typeof firebaseAuthIAMService.createUser>[0]) => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));
      const result = await firebaseAuthIAMService.createUser(userData);
      
      setAuthState({
        user: result.user,
        profile: result.profile,
        loading: false,
        error: null
      });
      
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'User creation failed';
      setAuthState(prev => ({ 
        ...prev, 
        loading: false, 
        error: errorMessage 
      }));
      throw error;
    }
  };

  const updateProfile = async (updates: Parameters<typeof firebaseAuthIAMService.updateUser>[1]) => {
    try {
      if (!authState.user) throw new Error('No authenticated user');
      
      setAuthState(prev => ({ ...prev, loading: true, error: null }));
      const updatedProfile = await firebaseAuthIAMService.updateUser(
        authState.user.uid, 
        updates, 
        authState.user.uid
      );
      
      setAuthState(prev => ({
        ...prev,
        profile: updatedProfile,
        loading: false,
        error: null
      }));
      
      return updatedProfile;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Profile update failed';
      setAuthState(prev => ({ 
        ...prev, 
        loading: false, 
        error: errorMessage 
      }));
      throw error;
    }
  };

  const hasFeatureAccess = (featureId: string): boolean => {
    if (!authState.profile) return false;
    
    // Check package permissions
    const hasPackageAccess = authState.profile.packagePermissions.some(
      perm => perm.featureId === featureId
    );
    
    // Check custom permissions
    const hasCustomAccess = authState.profile.customPermissions.some(
      perm => perm.featureId === featureId && perm.isActive
    );
    
    return hasPackageAccess || hasCustomAccess;
  };

  const hasRole = (role: string): boolean => {
    return authState.profile?.roles.includes(role) || false;
  };

  const refreshProfile = async () => {
    try {
      if (!authState.user) return;
      
      setAuthState(prev => ({ ...prev, loading: true }));
      const profile = await firebaseAuthIAMService.getCurrentUserProfile();
      
      setAuthState(prev => ({
        ...prev,
        profile,
        loading: false
      }));
    } catch (error) {
      console.error('Failed to refresh profile:', error);
      setAuthState(prev => ({ 
        ...prev, 
        loading: false,
        error: error instanceof Error ? error.message : 'Failed to refresh profile'
      }));
    }
  };

  return {
    ...authState,
    signIn,
    signOut,
    createUser,
    updateProfile,
    hasFeatureAccess,
    hasRole,
    refreshProfile,
    isAuthenticated: !!authState.user,
    isAdmin: authState.profile?.roles.includes('admin') || false
  };
};
