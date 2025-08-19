// Environment configuration for admin-frontend
// All variables read directly from process.env - no shared dependencies

import { z } from 'zod';

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'staging', 'production']).default('development'),
  PORT: z.string().transform(Number).default(3001),
  NEXTAUTH_URL: z.string().url(),
  BACKEND_URL: z.string().url(),
  NEXT_PUBLIC_BACKEND_URL: z.string().url(),
  NEXT_PUBLIC_ADMIN_URL: z.string().url(),
  NEXTAUTH_SECRET: z.string().min(1),
  OIDC_CLIENT_ID: z.string().min(1),
  OIDC_CLIENT_SECRET: z.string().min(1),
});

export const env = envSchema.parse(process.env);