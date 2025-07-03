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
  },
  poweredByHeader: false,
  generateEtags: true,
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.externals = config.externals || [];
      config.externals.push('firebase-admin');
      config.externals.push('google-logging-utils');
      // Externalize Node.js built-in modules for client and edge environments
      config.externals.push(/^node:/);
    }
    return config;
  },
};

export default nextConfig;
