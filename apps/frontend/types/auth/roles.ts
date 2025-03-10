export enum UserRole {
  GUEST = "guest",
  REGISTERED_USER = "registered_user",
  PREMIUM_USER = "premium_user",
  TOKEN_HOLDER = "token_holder",
  ADMINISTRATOR = "administrator"
}

// Access levels for each role
export const ROLE_ACCESS_LEVELS: Record<UserRole, number> = {
  [UserRole.ADMINISTRATOR]: 4,
  [UserRole.TOKEN_HOLDER]: 3,
  [UserRole.PREMIUM_USER]: 2,
  [UserRole.REGISTERED_USER]: 1,
  [UserRole.GUEST]: 0
};

// Helper function to check if a role has required access level
export function hasRequiredAccessLevel(currentRole: UserRole, requiredRole: UserRole): boolean {
  return ROLE_ACCESS_LEVELS[currentRole] >= ROLE_ACCESS_LEVELS[requiredRole];
}
