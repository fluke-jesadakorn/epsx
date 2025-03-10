import {
  Injectable,
  BadRequestException,
  NotFoundException,
} from "@nestjs/common";
import { FirebaseAdminService } from "../../../shared/firebase-admin";
import { UserRole } from "../../../shared/types/roles.enum";

@Injectable()
export class UserService {
  constructor(private readonly firebaseAdmin: FirebaseAdminService) {}

  async listUsers(maxResults?: number): Promise<
    Array<{
      userId: string;
      email?: string;
      role: UserRole;
    }>
  > {
    try {
      const listUsersResult = await this.firebaseAdmin.auth.listUsers(
        maxResults || 1000
      );

      return listUsersResult.users.map((user) => {
        const claims = user.customClaims || {};
        let role = UserRole.GUEST;

        // Check administrator first since it's highest privilege
        if (claims.admin) {
          role = UserRole.ADMINISTRATOR;
        } else if (claims.premium) {
          role = UserRole.PREMIUM_USER;
        } else if (claims.basic) {
          role = UserRole.REGISTERED_USER;
        }

        return {
          userId: user.uid,
          email: user.email,
          role,
        };
      });
    } catch (error) {
      throw new BadRequestException("Failed to list users");
    }
  }

  async assignUserRole(userId: string, role: UserRole): Promise<void> {
    try {
      const user = await this.firebaseAdmin.auth.getUser(userId);
      let claims = {
        admin: false,
        premium: false,
        basic: false,
      };

      // Set claims based on role
      switch (role) {
        case UserRole.ADMINISTRATOR:
          claims.admin = true;
          claims.premium = true; // Administrator users get premium access too
          break;
        case UserRole.PREMIUM_USER:
          claims.premium = true;
          break;
        case UserRole.REGISTERED_USER:
          claims.basic = true;
          break;
        // Guest role has no special claims
        default:
          break;
      }

      await this.firebaseAdmin.auth.setCustomUserClaims(userId, claims);
    } catch (error: any) {
      if (error?.code === "auth/user-not-found") {
        throw new NotFoundException("User not found");
      }
      throw new BadRequestException("Failed to assign role");
    }
  }
}
