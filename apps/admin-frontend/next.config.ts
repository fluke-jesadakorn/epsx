import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  reactStrictMode: true,
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
  poweredByHeader: false,
  generateEtags: true,
  output: 'standalone',
};

export default nextConfig;
