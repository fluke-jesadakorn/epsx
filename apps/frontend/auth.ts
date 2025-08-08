import NextAuth from 'next-auth';
import type { AuthOptions } from 'next-auth';
import Credentials from 'next-auth/providers/credentials';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export const authOptions: AuthOptions = {
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
          // Updated for new backend session authentication format
          const response = await fetch(`${BACKEND_URL}/api/v1/auth/login`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              type: 'credentials',
              email: credentials.email,
              password: credentials.password
            }),
          });

          if (!response.ok) {
            console.error('Frontend login failed:', response.status, response.statusText);
            return null;
          }

          const data = await response.json();
          
          // Backend returns { user_id, email, role, permissions, subscription_tier, expires_at, access_token }
          if (data.user_id && data.access_token) {
            return {
              id: data.user_id,
              email: data.email,
              role: data.role || 'user',
              permissions: data.permissions || [],
              subscription_tier: data.subscription_tier || 'free',
              package_tier: data.subscription_tier || 'free',
              session_id: data.access_token,  // Store access token as session ID for API calls
              expires_at: data.expires_at,
            };
          }
          
          console.error('Invalid login response format:', data);
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
        token.package_tier = user.package_tier;
        token.session_id = user.session_id; // Store session ID for backend calls
        token.expires_at = user.expires_at;
        token.lastRefresh = Date.now();
      } else if (token?.session_id) {
        // Refresh user role from backend if token exists and it's been more than 5 seconds since last refresh
        const now = Date.now();
        const lastRefresh = token.lastRefresh || 0;
        const shouldRefresh = now - lastRefresh > 5 * 1000; // Refresh every 5 seconds for testing
        
        if (shouldRefresh) {
          try {
            const response = await fetch(`${BACKEND_URL}/api/v1/auth/me`, {
              headers: {
                'Authorization': `Bearer ${token.session_id}`,
              },
            });
            
            if (response.ok) {
              const userData = await response.json();
              if (userData.user) {
                token.role = userData.user.roles?.[0] || userData.user.role || token.role;
                token.permissions = userData.user.permissions || token.permissions;
                token.subscription_tier = userData.user.subscription_tier || token.subscription_tier;
                token.package_tier = userData.user.package_tier || token.package_tier;
                token.lastRefresh = now;
              }
            }
          } catch (error) {
            console.error('Failed to refresh user role:', error);
          }
        }
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
        session.user.package_tier = token.package_tier as string;
        session.session_id = token.session_id as string; // Pass session ID to session
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
    maxAge: 24 * 60 * 60,
  },
  secret: process.env.NEXTAUTH_SECRET,
};

export const { auth, signIn, signOut } = NextAuth(authOptions);