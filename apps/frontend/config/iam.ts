// IAM Configuration for the frontend application

export const IAM_CONFIG = {
  // Default role for new users
  defaultRole: 'user',
  
  // Session configuration
  session: {
    cookieName: '__session',
    maxAge: 60 * 60 * 24 * 7, // 7 days
  },
  
  // Route protection configuration
  routes: {
    // Public routes that don't require authentication
    public: [
      '/',
      '/login',
      '/register',
      '/forgot-password',
      '/reset-password',
      '/api/auth',
      '/api/public',
      '/_next',
      '/favicon.ico',
      '/public',
      '/static',
    ],
    
    // Protected routes with required permissions
    protected: {
      '/dashboard': ['read:own_data'],
      '/admin': ['admin:access'],
      '/admin/users': ['manage:users'],
      '/admin/roles': ['manage:roles'],
      '/premium': ['read:premium_content'],
      '/moderator': ['moderate:content'],
      '/api/admin': ['admin:access'],
      '/api/moderator': ['moderate:content'],
      '/api/users': ['manage:users'],
    },
    
    // Routes that require authentication but no specific permissions
    authenticated: [
      '/profile',
      '/settings',
      '/account',
    ],
  },
  
  // API endpoints
  api: {
    baseUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002',
    endpoints: {
      auth: '/api/auth',
      permissions: '/api/permissions',
      roles: '/api/roles',
      users: '/api/users',
    },
  },
  
  // Firebase configuration
  firebase: {
    apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
    authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
    projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
    storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
    messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
    appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
    measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID,
  },
};

// Permission definitions
export const PERMISSIONS = {
  // User permissions
  READ_OWN_DATA: 'read:own_data',
  WRITE_OWN_DATA: 'write:own_data',
  READ_PUBLIC_CONTENT: 'read:public_content',
  
  // Premium user permissions
  READ_PREMIUM_CONTENT: 'read:premium_content',
  WRITE_PREMIUM_CONTENT: 'write:premium_content',
  
  // Moderator permissions
  MODERATE_CONTENT: 'moderate:content',
  READ_MODERATED: 'read:moderated',
  WRITE_MODERATED: 'write:moderated',
  
  // Admin permissions
  ADMIN_ACCESS: 'admin:access',
  MANAGE_USERS: 'manage:users',
  MANAGE_ROLES: 'manage:roles',
  READ_ALL: 'read:all',
  WRITE_ALL: 'write:all',
  
  // Super admin permissions
  SUPER_ADMIN: '*',
} as const;

// Role definitions
export const ROLES = {
  USER: 'user',
  PREMIUM_USER: 'premium_user',
  MODERATOR: 'moderator',
  ADMIN: 'admin',
  SUPER_ADMIN: 'super_admin',
} as const;

// Error messages
export const IAM_ERROR_MESSAGES = {
  UNAUTHORIZED: 'You are not authorized to access this resource',
  FORBIDDEN: 'You do not have permission to access this resource',
  SESSION_EXPIRED: 'Your session has expired. Please log in again',
  INVALID_ROLE: 'Invalid role specified',
  PERMISSION_DENIED: 'Permission denied',
} as const;

// Cache configuration
export const CACHE_CONFIG = {
  permissions: {
    ttl: 5 * 60 * 1000, // 5 minutes
    key: 'user_permissions',
  },
  roles: {
    ttl: 10 * 60 * 1000, // 10 minutes
    key: 'user_roles',
  },
} as const;
