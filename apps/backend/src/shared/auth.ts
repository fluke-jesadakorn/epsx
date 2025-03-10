import { Injectable } from '@nestjs/common';
import { FirebaseAdminService } from './firebase-admin';

@Injectable()
export class AuthService {
  constructor(private readonly firebaseAdmin: FirebaseAdminService) {}

  async verifyToken(token: string) {
    return await this.firebaseAdmin.verifyIdToken(token);
  }
  
  async getUser(userId: string) {
    return await this.firebaseAdmin.getUser(userId);
  }

  async setCustomUserClaims(userId: string, claims: Record<string, any>) {
    return await this.firebaseAdmin.setCustomUserClaims(userId, claims);
  }

  async getUserByEmail(email: string) {
    return await this.firebaseAdmin.getUserByEmail(email);
  }
}
