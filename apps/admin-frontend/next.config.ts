import { config } from 'dotenv';
import type { NextConfig } from 'next';

// Load app-specific environment
config();

const nextConfig: NextConfig = {
  output: 'standalone',
  reactStrictMode: true,
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  
  // Ultra-minimal bundle optimizations
  productionBrowserSourceMaps: false,
  modularizeImports: {
    '@radix-ui/react-icons': {
      transform: '@radix-ui/react-icons/dist/{{member}}.js',
    },
    'lucide-react': {
      transform: 'lucide-react/dist/esm/icons/{{kebabCase member}}',
    },
  },
  
  // Admin-specific optimizations
  experimental: {
    serverActions: {
      allowedOrigins: ['*'],
    },
    optimizePackageImports: [
      '@radix-ui/react-*', 
      'lucide-react',
      'react',
      'react-dom'
    ],
    useCache: true,
    staleTimes: {
      dynamic: 180, // 3 minutes for admin dynamic content
      static: 1800, // 30 minutes for admin static content
    },
    // Container-specific optimizations
    turbotrace: {
      logLevel: 'error',
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
  
  // Ultra-minimal standalone bundle
  outputFileTracingExcludes: {
    '*': [
      './node_modules/@swc/core*/**/*',
      './node_modules/esbuild/**/*',
      './node_modules/webpack/**/*',
      './node_modules/@babel/**/*',
      './node_modules/typescript/**/*',
      './node_modules/eslint/**/*',
      './node_modules/@types/**/*',
    ],
  },
  
  serverExternalPackages: ['firebase-admin'],
  poweredByHeader: false,
  generateEtags: true,
};

export default nextConfig;
