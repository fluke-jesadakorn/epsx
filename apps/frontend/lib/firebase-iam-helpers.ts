import { auth, db } from '@/lib/firebase-iam';
import { doc, getDoc } from 'firebase/firestore';
import { templateEvaluationService, UserTemplateContext } from '@/lib/template-evaluation';
import { PackageTier } from '@epsx/types';

export async function getUserPermissions(uid: string) {
  try {
    const userDoc = await getDoc(doc(db, 'users', uid));
    if (userDoc.exists()) {
      const userData = userDoc.data();
      
      // Build user context for template evaluation
      const context: UserTemplateContext = {
        userId: uid,
        packageTier: userData.packageTier || PackageTier.FREE,
        staticPermissions: userData.permissions || [],
        roles: userData.roles || [],
      };
      
      // Evaluate templates and get effective permissions
      const effectivePermissions = await templateEvaluationService.evaluateUserPermissions(context);
      
      return effectivePermissions.permissions;
    }
    return [];
  } catch (error) {
    console.error('Error getting user permissions:', error);
    return [];
  }
}

export async function getUserRoles(uid: string) {
  try {
    const userDoc = await getDoc(doc(db, 'users', uid));
    if (userDoc.exists()) {
      const userData = userDoc.data();
      return userData.roles || [];
    }
    return [];
  } catch (error) {
    console.error('Error getting user roles:', error);
    return [];
  }
}

export async function hasPermission(uid: string, permission: string) {
  try {
    const userDoc = await getDoc(doc(db, 'users', uid));
    if (userDoc.exists()) {
      const userData = userDoc.data();
      
      // Build user context for template evaluation
      const context: UserTemplateContext = {
        userId: uid,
        packageTier: userData.packageTier || PackageTier.FREE,
        staticPermissions: userData.permissions || [],
        roles: userData.roles || [],
      };
      
      // Use template-aware permission checking
      return await templateEvaluationService.hasPermission(context, permission);
    }
    return false;
  } catch (error) {
    console.error('Error checking permission:', error);
    return false;
  }
}

export async function hasRole(uid: string, role: string) {
  const roles = await getUserRoles(uid);
  return roles.includes(role);
}
