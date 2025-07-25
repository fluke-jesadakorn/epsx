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
    const res = await fetch('/api/v1/authentication/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
      credentials: 'include',
    });
    
    if (!res.ok) throw new Error('Login failed');
    return res.json();
  },

  logout: async (): Promise<void> => {
    const res = await fetch('/api/v1/authentication/logout', { method: 'POST', credentials: 'include' });
    if (!res.ok) throw new Error('Logout failed');
  },

  me: async (): Promise<UsrRes> => {
    const res = await fetch('/api/v1/authentication/profile', { credentials: 'include' });
    if (!res.ok) throw new Error('Auth check failed');
    return res.json();
  },
};