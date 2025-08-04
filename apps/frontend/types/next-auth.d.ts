import { DefaultSession, DefaultUser } from 'next-auth';
import { JWT, DefaultJWT } from 'next-auth/jwt';

declare module 'next-auth' {
  interface Session {
    user: {
      id: string;
      email: string;
      role: string;
      permissions: string[];
      subscription_tier: string;
    } & DefaultSession['user'];
    accessToken: string;
    expires_at: string;
  }

  interface User extends DefaultUser {
    role: string;
    permissions: string[];
    subscription_tier: string;
    token: string;
    expires_at: string;
  }
}

declare module 'next-auth/jwt' {
  interface JWT extends DefaultJWT {
    id: string;
    role: string;
    permissions: string[];
    subscription_tier: string;
    accessToken: string;
    expires_at: string;
  }
}