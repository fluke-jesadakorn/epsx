import type { NextConfig } from 'next';

import { env } from './config/env';

const nextConfig: NextConfig = {
  output: 'standalone',
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
  
  // Mobile performance optimizations
  images: {
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920, 2048, 3840],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
    minimumCacheTTL: 60 * 60 * 24 * 7, // 7 days
    dangerouslyAllowSVG: false,
    contentSecurityPolicy: "default-src 'self'; script-src 'none'; sandbox;",
  },
  
  // Caching optimizations
  headers: async () => [
    {
      source: '/:path*',
      headers: [
        {
          key: 'X-Content-Type-Options',
          value: 'nosniff',
        },
        {
          key: 'X-Frame-Options',
          value: 'DENY',
        },
        {
          key: 'X-XSS-Protection',
          value: '1; mode=block',
        },
      ],
    },
    {
      source: '/static/:path*',
      headers: [
        {
          key: 'Cache-Control',
          value: 'public, max-age=31536000, immutable',
        },
      ],
    },
  ],
  
  experimental: {
    forceSwcTransforms: true,
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
      'react',
      'react-dom',
    ],
    // Container-specific optimizations
    optimizeCss: true,
    scrollRestoration: true,
    gzipSize: true,
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
  
  // Compression and optimization
  compress: true,
  poweredByHeader: false,
  
  env: {
    SITE_URL: env.SITE_URL,
    BACKEND_URL: env.BACKEND_URL,
    NEXT_PUBLIC_BACKEND_URL: env.NEXT_PUBLIC_BACKEND_URL,
  },
};

export default nextConfig;
