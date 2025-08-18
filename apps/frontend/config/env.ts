// Environment configuration for frontend
// NOTE: Shared variables are loaded via Next.js config from /.env.shared

export const env = {
  NODE_ENV: process.env.NODE_ENV || 'production',
  SITE_URL: process.env.SITE_URL || 'https://epsx.com', 
  BACKEND_URL: process.env.BACKEND_URL || 'https://api-4nrslhaei-info-epsxs-projects.vercel.app',
  NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL || 'https://api-4nrslhaei-info-epsxs-projects.vercel.app',
  NEXTAUTH_URL: process.env.NEXTAUTH_URL || 'https://frontend-info-epsxs-projects.vercel.app',
  NEXTAUTH_SECRET: process.env.NEXTAUTH_SECRET || 'your-shared-jwt-secret-32-chars-minimum-placeholder',
} as const;