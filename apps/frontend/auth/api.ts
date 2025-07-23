interface LoginReq {
  token: string;
}

interface UsrRes {
  user: {
    id: string;
    email: string;
    roles: string[];
  };
}

export const authApi = {
  login: async (req: LoginReq): Promise<UsrRes> => {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
    
    if (!res.ok) throw new Error('Login failed');
    return res.json();
  },

  logout: async (): Promise<void> => {
    const res = await fetch('/api/auth/logout', { method: 'POST' });
    if (!res.ok) throw new Error('Logout failed');
  },

  me: async (): Promise<UsrRes> => {
    const res = await fetch('/api/auth/me');
    if (!res.ok) throw new Error('Auth check failed');
    return res.json();
  },
};