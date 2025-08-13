import NextAuth from "next-auth"
import { authConfig } from "@epsx/auth-shared"

/**
 * Frontend Auth.js v5 configuration
 * Uses the shared auth config from packages/auth-shared
 */
export const { handlers, auth, signIn, signOut } = NextAuth({
  ...authConfig,
  
  // Frontend-specific customizations
  pages: {
    signIn: '/login',
    error: '/auth/error',
    signOut: '/auth/signout',
  },
  
  // Frontend callback customizations
  callbacks: {
    ...authConfig.callbacks,
    
    async redirect({ url, baseUrl }) {
      // Frontend-specific redirect logic
      
      // Always redirect to dashboard after sign in
      if (url.includes('/login') || url.includes('/auth/callback')) {
        return `${baseUrl}/dashboard`
      }
      
      // Handle other redirects
      if (url.startsWith("/")) return `${baseUrl}${url}`
      else if (new URL(url).origin === baseUrl) return url
      return baseUrl
    }
  }
})

export default { handlers, auth, signIn, signOut }