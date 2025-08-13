import NextAuth from "next-auth"
import { authConfig } from "@epsx/auth-shared"

/**
 * Admin Frontend Auth.js v5 configuration
 * Uses the shared auth config from packages/auth-shared with admin-specific customizations
 */
export const { handlers, auth, signIn, signOut } = NextAuth({
  ...authConfig,
  
  // Admin-specific page customizations
  pages: {
    signIn: '/login',
    error: '/login?error=',
    signOut: '/login',
  },
  
  // Admin-specific callback customizations
  callbacks: {
    ...authConfig.callbacks,
    
    async redirect({ url, baseUrl }) {
      // Admin-specific redirect logic
      
      // Always redirect to admin dashboard after sign in
      if (url.includes('/login') || url.includes('/auth/callback')) {
        return `${baseUrl}/`
      }
      
      // Handle other redirects
      if (url.startsWith("/")) return `${baseUrl}${url}`
      else if (new URL(url).origin === baseUrl) return url
      return baseUrl
    },
    
    async session({ session, token }) {
      // Enhanced session with admin-specific methods
      const enhancedSession = await authConfig.callbacks?.session?.({ session, token }) || session
      
      if (enhancedSession?.user) {
        // Add admin-specific convenience methods
        enhancedSession.user.hasAdminModule = (module: string) => 
          (token.admin_modules as string[])?.includes(module) || false
        enhancedSession.user.isSystemAdmin = () => 
          (token.admin_modules as string[])?.includes('system_admin') || false
      }
      
      return enhancedSession
    }
  }
})

export default { handlers, auth, signIn, signOut }