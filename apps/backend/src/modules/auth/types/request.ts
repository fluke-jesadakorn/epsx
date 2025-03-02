import { Request } from 'express';
import { UserRole } from '../../../shared/guards/role.guard';

export interface AuthUser {
  uid: string;
  email: string | undefined;
  role: UserRole;
}

export interface AuthenticatedRequest extends Request {
  user?: AuthUser;
}
