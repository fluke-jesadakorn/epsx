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
  output: 'standalone',
  webpack: (config, { isServer }) => {
    if (!isServer) {
      // Don't bundle firebase-admin and other server-only packages on the client
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
        crypto: false,
        path: false,
        os: false,
        stream: false,
        http: false,
        https: false,
        zlib: false,
        url: false,
        util: false,
        buffer: false,
        events: false,
        child_process: false,
      };
    }
    return config;
  },
};

export default nextConfig;
