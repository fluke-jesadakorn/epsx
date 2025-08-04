import type { NextAuthConfig } from 'next-auth';
import type { Provider } from 'next-auth/providers';
import Credentials from 'next-auth/providers/credentials';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

const providers: Provider[] = [
  Credentials({
    name: 'credentials',
    credentials: {
      email: { label: 'Email', type: 'email' },
      password: { label: 'Password', type: 'password' }
    },
    async authorize(credentials) {
      if (!credentials?.email || !credentials?.password) {
        return null;
      }

      try {
        // Call backend login API
        const response = await fetch(`${BACKEND_URL}/api/v1/auth/login`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: credentials.email,
            password: credentials.password,
            app_type: 'frontend'
          }),
        });

        if (!response.ok) {
          return null;
        }

        const data = await response.json();
        
        if (data.success && data.token) {
          // Return user object that will be stored in the JWT
          return {
            id: data.user.user_id,
            email: data.user.email,
            role: data.user.role,
            permissions: data.user.permissions,
            subscription_tier: data.user.subscription_tier,
            token: data.token,
            expires_at: data.user.expires_at,
          };
        }
        
        return null;
      } catch (error) {
        console.error('Auth error:', error);
        return null;
      }
    }
  })
];

const authConfig: NextAuthConfig = {
  providers,
  callbacks: {
    async jwt({ token, user }) {
      // Store user info in JWT token
      if (user) {
        token.id = user.id;
        token.email = user.email;
        token.role = user.role;
        token.permissions = user.permissions;
        token.subscription_tier = user.subscription_tier;
        token.accessToken = user.token;
        token.expires_at = user.expires_at;
      }
      return token;
    },
    async session({ session, token }) {
      // Send properties to the client
      if (token) {
        session.user.id = token.id as string;
        session.user.email = token.email as string;
        session.user.role = token.role as string;
        session.user.permissions = token.permissions as string[];
        session.user.subscription_tier = token.subscription_tier as string;
        session.accessToken = token.accessToken as string;
        session.expires_at = token.expires_at as string;
      }
      return session;
    }
  },
  pages: {
    signIn: '/login',
    error: '/login',
  },
  session: {
    strategy: 'jwt',
    maxAge: 24 * 60 * 60, // 24 hours
  },
  secret: process.env.NEXTAUTH_SECRET,
};

export default authConfig;