import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  reactStrictMode: true,
  transpilePackages: [
    '@epsx/api-client',
    '@epsx/config',
    '@epsx/frontend',
    '@epsx/shared',
    '@epsx/types',
    '@epsx/utils',
  ],
  experimental: {
    serverActions: {
      allowedOrigins: ['*'],
    },
    optimizePackageImports: ['@radix-ui/react-*'],
    useCache: true,
  },
  poweredByHeader: false,
  generateEtags: true,
};

export default nextConfig;
