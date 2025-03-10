import { UserRole } from '../../../shared/types/roles.enum';

export interface UserWithRoleDto {
  userId: string;
  email: string;
  role: UserRole;
  features: string[];
  permissions: string[];
  tokenBalance: number;
}

export interface UsersResponseDto {
  users: UserWithRoleDto[];
}

export interface RoleAssignmentResponse {
  success: boolean;
}

export interface RoleAssignmentMetadata {
  roleUpdate: boolean;
  newRole: UserRole;
  adminId: string;
}
