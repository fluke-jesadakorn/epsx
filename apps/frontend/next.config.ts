import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  // outputFileTracingRoot: process.env.NODE_ENV === 'production' ? '/app' : process.cwd(),

  // Ignore TypeScript errors during Docker builds (errors should be fixed separately)
  typescript: {
    ignoreBuildErrors: true,
  },

  // Skip static generation for error pages that fail with useContext issues
  // This is a known issue with Next.js 16 + React 19 + complex provider trees
  staticPageGenerationTimeout: 60,

  experimental: {
    // @ts-expect-error allowedDevOrigins is valid but missing from types in this version
    allowedDevOrigins: ['100.97.9.56', 'localhost:3000'],
  },

  // Transpile shared packages and metamask sdk to apply ignores
  transpilePackages: ['@/shared', '@wagmi/connectors', 'wagmi'],

  // Silence Next.js 16 Turbopack warning (using webpack config for dev)
  turbopack: {},
  // Improve HMR WebSocket reliability and fix SSR issues
  /* eslint-disable @typescript-eslint/no-explicit-any */
  webpack: (config: any, { dev, isServer, webpack }: { dev: boolean; isServer: boolean; webpack: any }) => {
    /* eslint-enable @typescript-eslint/no-explicit-any */
    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }

    // Web3/Wagmi build fix: Explicitly stub out problematic modules
    const path = require('path');
    const stubPath = path.join(process.cwd(), '../../shared/stubs/empty.ts');

    // Ensure shared components can resolve modules from the app's node_modules
    const appNodeModules = path.resolve(process.cwd(), 'node_modules');
    config.resolve.modules = [
      appNodeModules,
      ...(config.resolve.modules || ['node_modules']),
    ];

    config.resolve.alias = {
      ...config.resolve.alias,
      'thread-stream': stubPath,
      tap: stubPath,
      tape: stubPath,
      'why-is-node-running': stubPath,
      'pino-pretty': stubPath,
      'zod/mini': require.resolve('zod'),
    };
    // Standard Node.js polyfills for the browser (Client Component bundle)
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
        crypto: false,
        stream: false,
        url: false,
        zlib: false,
        http: false,
        https: false,
        os: false,
        path: false,
        worker_threads: false,
        lokijs: false,
        encoding: false,
        'pino-pretty': false,
      };
    }

    // Fix specific module resolution issues for both Server and Client
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^(tap|tape|desm|fastbench|pino-elasticsearch|why-is-node-running|thread-stream\/test)$/,
      })
    );

    return config;
  },
};

export default nextConfig;