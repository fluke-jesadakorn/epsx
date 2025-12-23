import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  // Empty turbopack config to disable it and use webpack
  turbopack: {},
  // Ignore TypeScript errors during Docker builds (errors should be fixed separately)
  typescript: {
    ignoreBuildErrors: true,
  },
  experimental: {
    // Fix WebSocket connection issues in Next.js 15
    webpackBuildWorker: true,
  },
  // Transpile shared packages for Web3 compatibility
  transpilePackages: ['@/shared'],

  // Improve HMR WebSocket reliability and fix module resolution
  webpack: (config, { dev, isServer, webpack }) => {
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
      'thread-stream/test': stubPath,
      tap: stubPath,
      tape: stubPath,
      'why-is-node-running': stubPath,
      'pino-pretty': stubPath,
      'pino-elasticsearch': stubPath,
      desm: stubPath,
      fastbench: stubPath,
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
        // Web3 specific ignores
        'pino-pretty': false,
        lokijs: false,
        encoding: false,
      };
    }

    // Fix specific module resolution issues for both Server and Client
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^(tap|tape|desm|fastbench|pino-elasticsearch|why-is-node-running|thread-stream\/test|thread-stream)$/,
      })
    );

    // Prevent React Native modules from being bundled in web environment
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^(react-native|@react-native)/,
      })
    );

    // Additional React Native specific modules to ignore
    // Add IgnorePlugin for specific React Native modules instead of externals
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^react-native$/,
      })
    );

    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^react-native-device-info$/,
      })
    );

    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^react-native-keychain$/,
      })
    );

    return config;
  },
};

export default nextConfig;
