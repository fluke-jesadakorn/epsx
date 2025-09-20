import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  outputFileTracingRoot:
    process.env.NODE_ENV === 'production' ? '/app' : process.cwd(),
  experimental: {
    // Fix WebSocket connection issues in Next.js 15
    webpackBuildWorker: true,
  },
  // Improve HMR WebSocket reliability
  webpack: (config, { dev, isServer }) => {
    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }
    return config;
  },
};

export default nextConfig;
