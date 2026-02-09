// Placeholder for removed auth service
type ServiceArgs = unknown[];

const authServiceInstance = {
  login: async (..._args: ServiceArgs) => ({ success: false, error: 'Not implemented' }),
  logout: async (..._args: ServiceArgs) => ({ success: true }),
  getCurrentUser: async (..._args: ServiceArgs) => null,
  refreshToken: async (..._args: ServiceArgs) => ({ success: false }),
  validateSession: async (..._args: ServiceArgs) => false,
};

// Re-export the auth service singleton
export const authService = authServiceInstance;
