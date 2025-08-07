import NextAuth, { AuthOptions } from 'next-auth';
import Credentials from 'next-auth/providers/credentials';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

const authOptions: AuthOptions = {
  providers: [
    Credentials({
      name: 'credentials',
      credentials: {
        email: { label: 'Email', type: 'email' },
        password: { label: 'Password', type: 'password' }
      },
      async authorize(credentials: any) {
        if (!credentials?.email || !credentials?.password) {
          return null;
        }

        try {
          const response = await fetch(`${BACKEND_URL}/api/v1/auth/login`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              email: credentials.email,
              password: credentials.password,
              app_type: 'admin',
              type: 'credentials'
            }),
          });

          if (!response.ok) {
            return null;
          }

          const data = await response.json();
          
          if (data.user_id && data.access_token) {
            return {
              id: data.user_id,
              email: data.email,
              role: data.role,
              permissions: data.permissions,
              subscription_tier: data.subscription_tier,
              session_id: data.session_id || data.user_id, // Use session_id or fallback to user_id
              token: data.access_token,
              expires_at: data.expires_at,
            };
          }
          
          return null;
        } catch (error) {
          console.error('Auth error:', error);
          return null;
        }
      }
    })
  ],
  callbacks: {
    async jwt({ token, user }: any) {
      if (user) {
        token.id = user.id;
        token.email = user.email;
        token.role = user.role;
        token.permissions = user.permissions;
        token.subscription_tier = user.subscription_tier;
        token.session_id = user.session_id;
        token.accessToken = user.token;
        token.expires_at = user.expires_at;
      }
      return token;
    },
    async session({ session, token }: any) {
      if (token) {
        session.user.id = token.id as string;
        session.user.email = token.email as string;
        session.user.role = token.role as string;
        session.user.permissions = token.permissions as string[];
        session.user.subscription_tier = token.subscription_tier as string;
        session.session_id = token.session_id as string;
        session.accessToken = token.accessToken as string;
        session.expires_at = token.expires_at as string;
      }
      return session;
    },
    async signIn({ user }) {
      const isAdmin = user.role === 'admin' || 
                     user.role === 'system_administrator' || 
                     user.role === 'super_admin';
      
      if (!isAdmin) {
        return false;
      }
      
      return true;
    }
  },
  pages: {
    signIn: '/login',
    error: '/login',
  },
  session: {
    strategy: 'jwt',
    maxAge: 24 * 60 * 60,
  },
  secret: process.env.NEXTAUTH_SECRET,
};

const handler = NextAuth(authOptions);

export { handler as GET, handler as POST };