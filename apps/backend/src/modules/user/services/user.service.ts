import { Injectable, BadRequestException } from "@nestjs/common";
import { FirebaseAdmin } from "@epsx/shared/services/firebase-admin";
import { UserRole } from "@epsx/shared/types/roles.enum";
import { UserRecord } from "firebase-admin/auth";

export interface User {
  uid: string;
  email?: string;
  role: UserRole;
  tokenBalance: number;
}

@Injectable()
export class UserService {
  constructor(private readonly firebaseAdmin: FirebaseAdmin) {}

  async listUsers(maxResults?: number): Promise<User[]> {
    try {
      // Since FirebaseAdmin doesn't expose listUsers directly, we'll get users one by one up to maxResults
      const users: User[] = [];
      let uid = '';
      
      while (users.length < (maxResults || 1000)) {
        const user = await this.firebaseAdmin.getUser(uid);
        if (!user) break;
        
        users.push({
          uid: user.uid,
          email: user.email,
          role: (user.customClaims || {}).role || UserRole.GUEST,
          tokenBalance: (user.customClaims || {}).tokenBalance || 0
        });
        
        uid = user.uid;
      }
      
      return users;
    } catch (error: any) {
      throw new BadRequestException(`Failed to list users: ${error.message}`);
    }
  }

  async assignUserRole(uid: string, role: UserRole): Promise<void> {
    try {
      await this.firebaseAdmin.setCustomUserClaims(uid, { role });
    } catch (error: any) {
      throw new BadRequestException(`Failed to assign role: ${error.message}`);
    }
  }

  async getUserRole(uid: string): Promise<{ role: UserRole }> {
    try {
      const user = await this.firebaseAdmin.getUser(uid);
      const customClaims = user.customClaims || {};
      
      return {
        role: customClaims.role || UserRole.GUEST
      };
    } catch (error: any) {
      throw new BadRequestException(`Failed to get user role: ${error.message}`);
    }
  }
}
