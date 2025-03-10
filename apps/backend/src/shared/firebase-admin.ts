import * as admin from 'firebase-admin';
import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

@Injectable()
export class FirebaseAdminService {
  private app: admin.app.App;
  public auth: admin.auth.Auth;

  constructor(private configService: ConfigService) {
    const projectId = this.configService.get('FIREBASE_PROJECT_ID');
    const privateKey = this.configService.get('FIREBASE_PRIVATE_KEY');
    const clientEmail = this.configService.get('FIREBASE_CLIENT_EMAIL');

    if (!projectId || !privateKey || !clientEmail) {
      throw new Error('Missing Firebase configuration environment variables');
    }

    this.app = admin.initializeApp({
      credential: admin.credential.cert({
        projectId,
        privateKey: privateKey.replace(/\\n/g, '\n'),
        clientEmail,
      }),
      storageBucket: this.configService.get('FIREBASE_STORAGE_BUCKET')
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

  async verifySessionCookie(sessionCookie: string): Promise<admin.auth.DecodedIdToken> {
    return this.auth.verifySessionCookie(sessionCookie);
  }
}

// Single export is enough since the class is already decorated with @Injectable()
