import { auth, db } from '@/lib/firebase-iam';
import { doc, getDoc } from 'firebase/firestore';

export async function getUserPermissions(uid: string) {
  try {
    const userDoc = await getDoc(doc(db, 'users', uid));
    if (userDoc.exists()) {
      const userData = userDoc.data();
      return userData.permissions || [];
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
  const permissions = await getUserPermissions(uid);
  return permissions.includes(permission);
}

export async function hasRole(uid: string, role: string) {
  const roles = await getUserRoles(uid);
  return roles.includes(role);
}
