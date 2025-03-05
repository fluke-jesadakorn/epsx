import * as admin from 'firebase-admin';
import { Injectable } from '@nestjs/common';

@Injectable()
export class FirebaseAdminService {
  private app: admin.app.App;
  public auth: admin.auth.Auth;

  constructor() {
    if (!process.env.FIREBASE_PROJECT_ID || !process.env.FIREBASE_PRIVATE_KEY || !process.env.FIREBASE_CLIENT_EMAIL) {
      throw new Error('Missing Firebase configuration environment variables');
    }

    this.app = admin.initializeApp({
      credential: admin.credential.cert({
        projectId: process.env.FIREBASE_PROJECT_ID,
        privateKey: process.env.FIREBASE_PRIVATE_KEY.replace(/\\n/g, '\n'),
        clientEmail: process.env.FIREBASE_CLIENT_EMAIL,
      }),
      storageBucket: process.env.FIREBASE_STORAGE_BUCKET
    });
    this.auth = this.app.auth();
  }

  async verifyIdToken(token: string): Promise<admin.auth.DecodedIdToken> {
    return this.auth.verifyIdToken(token);
  }

  async setCustomUserClaims(uid: string, claims: Record<string, any>): Promise<void> {
    return this.auth.setCustomUserClaims(uid, claims);
  }

  async getUser(uid: string): Promise<admin.auth.UserRecord> {
    return this.auth.getUser(uid);
  }

  async getUserByEmail(email: string): Promise<admin.auth.UserRecord> {
    return this.auth.getUserByEmail(email);
  }
}

// Single export is enough since the class is already decorated with @Injectable()
