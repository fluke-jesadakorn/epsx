interface AdminLoginReq {
  token: string;
}

interface AdminUsrRes {
  user: {
    id: string;
    email: string;
    roles: string[];
    isAdmin: boolean;
  };
}

export const adminAuthApi = {
  login: async (req: AdminLoginReq): Promise<AdminUsrRes> => {
    const res = await fetch('/api/admin/authentication/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    
    if (!res.ok) throw new Error('Admin login failed');
    return res.json();
  },

  logout: async (): Promise<void> => {
    const res = await fetch('/api/admin/authentication/logout', { method: 'POST' });
    if (!res.ok) throw new Error('Admin logout failed');
  },

  me: async (): Promise<AdminUsrRes> => {
    const res = await fetch('/api/admin/authentication/profile');
    if (!res.ok) throw new Error('Admin auth check failed');
    return res.json();
  },
};