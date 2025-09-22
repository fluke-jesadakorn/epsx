import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  outputFileTracingRoot: process.env.NODE_ENV === 'production' ? '/app' : process.cwd(),
  eslint: {
    ignoreDuringBuilds: true,
  },
  experimental: {
    // Fix WebSocket connection issues in Next.js 15
    webpackBuildWorker: true,
  },
  // Improve HMR WebSocket reliability and fix SSR issues
  webpack: (config, { dev, isServer }) => {
    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }
    
    // Fix browser-only APIs during SSR
    if (isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        'indexeddb-js': false,
        'fake-indexeddb': false,
      };
    }
    
    return config;
  },
};

export default nextConfig;