import { FirebaseAdminService } from './firebase-admin';

export const auth = {
  verifyToken: async (token: string) => {
    const firebaseAdmin = new FirebaseAdminService();
    return await firebaseAdmin.verifyIdToken(token);
  },
  
  getUser: async (userId: string) => {
    const firebaseAdmin = new FirebaseAdminService();
    return await firebaseAdmin.getUser(userId);
  },

  setCustomUserClaims: async (userId: string, claims: Record<string, any>) => {
    const firebaseAdmin = new FirebaseAdminService();
    return await firebaseAdmin.setCustomUserClaims(userId, claims);
  },

  getUserByEmail: async (email: string) => {
    const firebaseAdmin = new FirebaseAdminService();
    return await firebaseAdmin.getUserByEmail(email);
  }
};
