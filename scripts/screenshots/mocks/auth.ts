import type { MockHandler } from '../types';
import { MOCK_USER, MOCK_ADMIN, USER_PERMS, ADMIN_PERMS } from '../helpers';

const session = {
  valid: true,
  authenticated: true,
  token: MOCK_USER.access,
  user: {
    ...MOCK_USER,
    created_at: '2024-01-15T10:00:00Z',
    last_login: '2025-02-14T08:30:00Z',
  },
  permissions: USER_PERMS,
  wallet_address: MOCK_USER.wallet_address,
};

const adminSession = {
  valid: true,
  authenticated: true,
  token: MOCK_ADMIN.access,
  user: {
    ...MOCK_ADMIN,
    created_at: '2024-01-01T00:00:00Z',
    last_login: '2025-02-14T09:00:00Z',
  },
  permissions: ADMIN_PERMS,
  wallet_address: MOCK_ADMIN.wallet_address,
};

export const authMocks: MockHandler[] = [
  {
    pattern: '**/api/auth/**',
    handler: (url: URL) => {
      if (url.pathname.includes('admin')) return adminSession;
      return session;
    },
  },
];
