import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  reactStrictMode: true,
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  // No transpile packages needed for standalone app
  experimental: {
    serverActions: {
      allowedOrigins: ['*'],
    },
    optimizePackageImports: ['@radix-ui/react-*', 'lucide-react'],
    useCache: true,
    staleTimes: {
      dynamic: 180, // 3 minutes for admin dynamic content
      static: 1800, // 30 minutes for admin static content
    },
  },
  
  // Admin-specific caching and security headers
  async headers() {
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'https://api-4nrslhaei-info-epsxs-projects.vercel.app';
    
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
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Content-Security-Policy',
            value: `
              default-src 'self';
              script-src 'self' 'unsafe-eval' 'unsafe-inline';
              style-src 'self' 'unsafe-inline';
              img-src 'self' data: blob:;
              font-src 'self';
              connect-src 'self' ${backendUrl} ws: wss:;
              frame-src 'none';
              object-src 'none';
              base-uri 'self';
              form-action 'self';
              frame-ancestors 'none';
            `.replace(/\s+/g, ' ').trim(),
          },
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'X-XSS-Protection',
            value: '1; mode=block',
          },
          {
            key: 'Referrer-Policy',
            value: 'strict-origin-when-cross-origin',
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
