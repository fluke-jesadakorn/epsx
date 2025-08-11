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
          console.warn('NextAuth: Missing email or password credentials');
          return null;
        }

        try {
          console.log('NextAuth: Attempting login for:', credentials.email);
          
          const response = await fetch(`${BACKEND_URL}/api/v1/auth/login`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({
              email: credentials.email,
              password: credentials.password,
              app_type: 'frontend',
              type: 'credentials'
            }),
          });

          if (!response.ok) {
            const errorText = await response.text();
            console.error('NextAuth: Backend login failed:', response.status, errorText);
            return null;
          }

          const data = await response.json();
          console.log('NextAuth: Backend login successful for:', data.email);
          
          if (data.user_id && data.access_token) {
            // Validate token expiry
            const expiresAt = new Date(data.expires_at);
            const now = new Date();
            if (expiresAt <= now) {
              console.error('NextAuth: Received expired token from backend');
              return null;
            }
            
            console.log('NextAuth: Creating user session for:', data.user_id);
            return {
              id: data.user_id,
              email: data.email,
              role: data.role,
              permissions: data.permissions,
              subscription_tier: data.subscription_tier,
              token: data.access_token,
              expires_at: data.expires_at,
            };
          }
          
          console.error('NextAuth: Invalid response structure from backend');
          return null;
        } catch (error) {
          console.error('NextAuth: Authorization error:', error);
          return null;
        }
      }
    })
  ],
  callbacks: {
    async jwt({ token, user }: any) {
      if (user) {
        console.log('NextAuth JWT: Processing new user data:', user.email);
        
        // Validate token expiry before storing
        if (user.expires_at) {
          const expiresAt = new Date(user.expires_at);
          const now = new Date();
          if (expiresAt <= now) {
            console.error('NextAuth JWT: Token already expired, rejecting');
            throw new Error('Token expired');
          }
        }
        
        token.id = user.id;
        token.email = user.email;
        token.role = user.role;
        token.permissions = user.permissions;
        token.subscription_tier = user.subscription_tier;
        token.accessToken = user.token;
        token.expires_at = user.expires_at;
        
        console.log('NextAuth JWT: Token updated successfully for:', user.id);
      }
      
      // Check if existing token is expired
      if (token.expires_at) {
        const expiresAt = new Date(token.expires_at);
        const now = new Date();
        if (expiresAt <= now) {
          console.warn('NextAuth JWT: Token expired, clearing session');
          // Return empty token to invalidate session
          return {};
        }
      }
      
      return token;
    },
    async session({ session, token }: any) {
      if (token && token.id) {
        console.log('NextAuth Session: Creating session for user:', token.id);
        
        session.user.id = token.id as string;
        session.user.email = token.email as string;
        session.user.role = token.role as string;
        session.user.permissions = token.permissions as string[];
        session.user.subscription_tier = token.subscription_tier as string;
        session.accessToken = token.accessToken as string;
        session.expires_at = token.expires_at as string;
        
        console.log('NextAuth Session: Session created successfully for:', session.user.email);
      } else {
        console.warn('NextAuth Session: No valid token data, session will be empty');
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
    maxAge: 24 * 60 * 60, // 24 hours to match backend token expiry
  },
  jwt: {
    maxAge: 24 * 60 * 60, // 24 hours to match backend token expiry
  },
  secret: process.env.NEXTAUTH_SECRET,
};

const handler = NextAuth(authOptions);

export { handler as GET, handler as POST };