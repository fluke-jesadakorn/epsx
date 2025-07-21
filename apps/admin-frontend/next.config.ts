import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  reactStrictMode: true,
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  transpilePackages: [
    '@epsx/api-client',
    '@epsx/config',
    '@epsx/admin-frontend',
    '@epsx/shared',
    '@epsx/types',
    '@epsx/utils'
  ],
  experimental: {
    serverActions: {
      allowedOrigins: ['*'],
    },
    optimizePackageImports: ['@radix-ui/react-*'],
  },
  serverExternalPackages: ['firebase-admin'],
  poweredByHeader: false,
  generateEtags: true,
};

export default nextConfig;
