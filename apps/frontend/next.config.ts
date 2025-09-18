import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  outputFileTracingRoot: process.env.NODE_ENV === 'production' ? '/app' : process.cwd(),
};

export default nextConfig;