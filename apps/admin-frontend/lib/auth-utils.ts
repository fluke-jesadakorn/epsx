// Simple auth utilities stub for admin frontend
// Admin users have full access via OIDC middleware

export const useAuthUtils = () => {
  return {
    hasPermission: () => Promise.resolve(true),
    canAccessRoute: () => Promise.resolve(true),
    isLoading: false,
    isAuthenticated: true,
  };
};