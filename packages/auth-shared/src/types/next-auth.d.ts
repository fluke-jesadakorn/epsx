import "next-auth"
import "next-auth/jwt"

declare module "next-auth" {
  interface User {
    admin_modules?: string[]
    permissions?: string[]
    package_tier?: string
    firebase_uid?: string
    role?: string
    isAdmin?: boolean
    hasPermission?: (permission: string) => boolean
    hasAdminModule?: (module: string) => boolean
  }

  interface Session {
    user: User & {
      admin_modules: string[]
      permissions: string[]
      package_tier: string
      firebase_uid: string
      role: string
      isAdmin: boolean
      hasPermission: (permission: string) => boolean
      hasAdminModule: (module: string) => boolean
    }
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    admin_modules?: string[]
    permissions?: string[]
    package_tier?: string
    firebase_uid?: string
    role?: string
  }
}