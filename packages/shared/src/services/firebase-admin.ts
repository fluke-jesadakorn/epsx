import { Injectable } from '@nestjs/common';
import * as admin from 'firebase-admin';

@Injectable()
export class FirebaseAdmin {
  private app: admin.app.App;

  constructor() {
    if (!admin.apps.length) {
      this.app = admin.initializeApp({
        credential: admin.credential.applicationDefault()
      });
    } else {
      this.app = admin.apps[0]!;
    }
  }

  async verifyIdToken(token: string): Promise<admin.auth.DecodedIdToken> {
    return this.app.auth().verifyIdToken(token);
  }

  async getUser(uid: string): Promise<admin.auth.UserRecord> {
    return this.app.auth().getUser(uid);
  }

  async setCustomUserClaims(uid: string, claims: { [key: string]: any }): Promise<void> {
    return this.app.auth().setCustomUserClaims(uid, claims);
  }

  async getUserByEmail(email: string): Promise<admin.auth.UserRecord> {
    return this.app.auth().getUserByEmail(email);
  }
}
