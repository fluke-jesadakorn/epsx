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
    '@epsx/config',
    '@epsx/types',
    '@epsx/ui',
    '@epsx/utils'
  ],
  experimental: {
    serverActions: {
      allowedOrigins: ['*'],
    },
    optimizePackageImports: ['@radix-ui/react-*', 'lucide-react', '@epsx/ui'],
    useCache: true,
    staleTimes: {
      dynamic: 180, // 3 minutes for admin dynamic content
      static: 1800, // 30 minutes for admin static content
    },
  },
  
  // Admin-specific caching headers
  async headers() {
    return [
      {
        source: '/api/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'private, s-maxage=180, stale-while-revalidate=300',
          },
        ],
      },
      {
        source: '/_next/static/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
    ];
  },
  serverExternalPackages: ['firebase-admin'],
  poweredByHeader: false,
  generateEtags: true,
};

export default nextConfig;
