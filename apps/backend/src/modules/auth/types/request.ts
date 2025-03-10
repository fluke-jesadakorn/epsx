import { Request } from 'express';
import { UserRole } from '../../../shared/types/roles.enum';

export interface AuthUser {
  uid: string;
  email: string | undefined;
  role: UserRole;
}

export interface AuthenticatedRequest extends Request {
  user?: AuthUser;
}
