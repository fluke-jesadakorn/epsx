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
              credentials: {
                email: credentials.email,
                password: credentials.password
              }
            }),
          });

          if (!response.ok) {
            console.error('Admin login failed:', response.status, response.statusText);
            return null;
          }

          const data = await response.json();
          
          // Backend returns { user: {...}, session: { session_id, expires_at } }
          if (data.user?.user_id && data.session?.session_id) {
            const user = data.user;
            const session = data.session;
            
            return {
              id: user.user_id,
              email: user.email,
              role: user.roles?.[0] || user.role || 'user',
              permissions: user.permissions || [],
              subscription_tier: user.subscription_tier || 'free',
              package_tier: user.package_tier || user.subscription_tier || 'free',
              session_id: session.session_id,  // Store session ID for API calls
              expires_at: session.expires_at,
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
    },
    async signIn({ user }) {
      // Updated admin role check to match backend system
      const isAdmin = user.role === 'admin' || 
                     user.role === 'system_administrator' || 
                     user.role === 'super_admin' ||
                     user.role === 'moderator'; // Add moderator as admin-capable role
      
      if (!isAdmin) {
        console.warn('User login denied - insufficient permissions:', {
          email: user.email,
          role: user.role,
          required: 'admin, system_administrator, super_admin, or moderator'
        });
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

export const { auth, signIn, signOut } = NextAuth(authOptions);