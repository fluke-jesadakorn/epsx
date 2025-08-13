import type { NextAuthConfig } from "next-auth"

/**
 * Modern Auth.js v5 configuration using backend OIDC discovery
 * Automatically configures endpoints via /.well-known/openid-configuration
 */
export const authConfig: NextAuthConfig = {
  providers: [
    {
      id: "epsx-backend",
      name: "EPSX",
      type: "oidc",
      issuer: `${process.env.BACKEND_URL || 'http://localhost:8080'}`,
      wellKnown: `${process.env.BACKEND_URL || 'http://localhost:8080'}/.well-known/openid-configuration`,
      clientId: process.env.OIDC_CLIENT_ID || "epsx-frontend",
      clientSecret: process.env.OIDC_CLIENT_SECRET,
      checks: ["pkce", "state"],
      authorization: {
        params: {
          scope: "openid profile email",
        },
      },
    }
  ],
  callbacks: {
    async jwt({ token, user, account }) {
      // Process backend OIDC tokens
      if (user && account?.provider === "epsx-backend") {
        try {
          // Extract claims from backend-issued tokens
          if (account.access_token) {
            // Backend tokens already contain the necessary claims
            // Copy relevant claims from the OIDC token
            token.admin_modules = (user as any).admin_modules || []
            token.permissions = (user as any).permissions || ['user:read']
            token.package_tier = (user as any).package_tier || 'FREE'
            token.firebase_uid = user.id
            token.role = (user as any).role || 'user'
          }
        } catch (error) {
          console.warn('Failed to process backend OIDC token, using defaults:', error)
          // Set default claims for new users
          token.admin_modules = []
          token.permissions = ['user:read']
          token.package_tier = 'FREE'
          token.role = 'user'
        }
      }
      
      return token
    },
    async session({ session, token }) {
      // Pass JWT claims to session for client-side access
      if (session.user) {
        session.user.admin_modules = token.admin_modules as string[]
        session.user.permissions = token.permissions as string[]
        session.user.package_tier = token.package_tier as string
        session.user.firebase_uid = token.firebase_uid as string
        session.user.role = token.role as string
        
        // Add convenience properties
        session.user.isAdmin = (token.admin_modules as string[])?.length > 0
        session.user.hasPermission = (permission: string) => 
          (token.permissions as string[])?.includes(permission) || false
        session.user.hasAdminModule = (module: string) =>
          (token.admin_modules as string[])?.includes(module) || false
      }
      
      return session
    },
    async signIn({ account }) {
      // Allow sign in for users with backend OIDC
      if (account?.provider === "epsx-backend") {
        // Backend OIDC flow - authentication already validated by backend
        // No additional user creation needed as backend handles it
        return true
      }
      
      return false
    },
    async redirect({ url, baseUrl }) {
      // Handle redirects after sign in
      if (url.startsWith("/")) return `${baseUrl}${url}`
      else if (new URL(url).origin === baseUrl) return url
      return baseUrl
    }
  },
  pages: {
    signIn: '/login',
    error: '/auth/error',
    signOut: '/auth/signout',
  },
  session: {
    strategy: "jwt",
    maxAge: 2 * 60 * 60, // 2 hours
  },
  jwt: {
    maxAge: 2 * 60 * 60, // 2 hours
  },
  cookies: {
    sessionToken: {
      name: "next-auth.session-token",
      options: {
        httpOnly: true,
        sameSite: "lax",
        path: "/",
        secure: process.env.NODE_ENV === "production",
      },
    },
  },
  trustHost: true,
  secret: process.env.NEXTAUTH_SECRET,
} satisfies NextAuthConfig


export default authConfig