import type { AuthOptions } from 'next-auth';
import Credentials from 'next-auth/providers/credentials';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

const providers = [
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
            type: 'admin',
            email: credentials.email,
            password: credentials.password
          }),
        });

        if (!response.ok) {
          // Enhanced error handling
          const errorText = await response.text();
          console.error('Authentication failed:', {
            status: response.status,
            statusText: response.statusText,
            error: errorText
          });
          return null;
        }

        const data = await response.json();
        
        // Backend returns flat response, not nested under user/success
        if (data.user_id && data.access_token) {
          // Return user object that will be stored in the JWT
          return {
            id: data.user_id,
            email: data.email,
            role: data.role,
            permissions: data.permissions,
            subscription_tier: data.subscription_tier,
            session_id: data.access_token, // Use access_token as session_id
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
];

const authConfig: AuthOptions = {
  providers,
  callbacks: {
    async jwt({ token, user }: any) {
      // Store user info in JWT token
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
      // Send properties to the client
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
    async signIn({ user }: any) {
      // Check if user has admin role - match middleware logic
      const isAdmin = user.role === 'admin' || 
                     user.role === 'system_administrator' || 
                     user.role === 'super_admin' ||
                     user.role === 'moderator';
      
      if (!isAdmin) {
        return false; // Deny sign in for non-admin users
      }
      
      return true;
    },
    async redirect({ url, baseUrl }: any) {
      // Redirect /admin to dashboard (/)
      if (url === `${baseUrl}/admin`) {
        return baseUrl;
      }
      
      // If url starts with baseUrl, it's a relative URL
      if (url.startsWith(baseUrl)) {
        return url;
      }
      
      // If url starts with /, it's a relative path
      if (url.startsWith('/')) {
        // Don't redirect to /admin, use dashboard instead
        if (url === '/admin') {
          return baseUrl;
        }
        return `${baseUrl}${url}`;
      }
      
      // Default to dashboard
      return baseUrl;
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