import type { NextConfig } from 'next';
import { config } from 'dotenv';
import { resolve } from 'path';

// Load shared variables first, then app-specific
config({ path: resolve(process.cwd(), '../..', '.env.shared') });
config();

import { env } from './config/env';

const nextConfig: NextConfig = {
  output: "standalone",
  reactStrictMode: true,
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  // Force dynamic rendering for container builds
  generateBuildId: () => 'build',
  trailingSlash: false,
  experimental: {
    forceSwcTransforms: true,
    missingSuspenseWithCSRBailout: false,
    optimizePackageImports: [
      '@radix-ui/react-avatar',
      '@radix-ui/react-checkbox', 
      '@radix-ui/react-dialog',
      '@radix-ui/react-label',
      '@radix-ui/react-progress',
      '@radix-ui/react-slot',
      '@radix-ui/react-switch',
      '@radix-ui/react-toast',
      '@radix-ui/react-tooltip',
      'lucide-react',
    ],
  },
  env: {
    SITE_URL: env.SITE_URL,
    BACKEND_URL: env.BACKEND_URL,
    NEXT_PUBLIC_BACKEND_URL: env.NEXT_PUBLIC_BACKEND_URL,
  },
};

export default nextConfig;