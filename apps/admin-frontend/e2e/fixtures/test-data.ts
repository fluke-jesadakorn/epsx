export const testUsers = {
  admin: {
    email: 'admin@epsx.com',
    password: 'admin123',
    name: 'Admin User',
    role: 'admin-full-004'
  },
  moderator: {
    email: 'moderator@epsx.com',
    password: 'mod123',
    name: 'Moderator User',
    role: 'moderator-standard-003'
  },
  basicUser: {
    email: 'user@epsx.com',
    password: 'user123',
    name: 'Basic User',
    role: 'user-basic-001'
  },
  premiumUser: {
    email: 'premium@epsx.com',
    password: 'premium123',
    name: 'Premium User',
    role: 'user-premium-002'
  }
};

export const testRoles = [
  'user-basic-001',
  'user-premium-002',
  'moderator-standard-003',
  'admin-full-004'
];

export const testPermissions = [
  'user:read',
  'user:write',
  'user:delete',
  'admin:read',
  'admin:write',
  'billing:read',
  'billing:write',
  'analytics:read'
];

export const mockUserData = [
  {
    id: '1',
    email: 'test1@epsx.com',
    name: 'Test User 1',
    role: 'user-basic-001',
    status: 'active'
  },
  {
    id: '2',
    email: 'test2@epsx.com',
    name: 'Test User 2',
    role: 'user-premium-002',
    status: 'active'
  },
  {
    id: '3',
    email: 'test3@epsx.com',
    name: 'Test User 3',
    role: 'moderator-standard-003',
    status: 'inactive'
  }
];

export const mockAnalyticsData = {
  totalUsers: 1234,
  activeUsers: 987,
  premiumUsers: 234,
  monthlyRevenue: 15678.90,
  conversionRate: 12.5
};

export const testErrorMessages = {
  invalidCredentials: 'Invalid credentials',
  unauthorized: 'You are not authorized',
  networkError: 'Network error occurred',
  validationError: 'Please check your input'
};