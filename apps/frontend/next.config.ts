import type { NextConfig } from 'next';

const STUB_PATH = '../../shared/stubs/empty.ts';

const nextConfig: NextConfig = {
  // Enable standalone output for Docker deployment
  output: 'standalone',

  typescript: { ignoreBuildErrors: true },

  // Skip static generation for error pages that fail with useContext issues
  // This is a known issue with Next.js 16 + React 19 + complex provider trees
  staticPageGenerationTimeout: 60,

  // Transpile shared packages only
  transpilePackages: ['@/shared'],

  // Tree-shake large icon/component libraries
  optimizePackageImports: ['lucide-react', '@radix-ui/react-icons', 'sonner'],

  // Silence Next.js 16 Turbopack warning (using webpack config for dev)
  turbopack: {
    resolveAlias: {
      'thread-stream': STUB_PATH,
      'pino-pretty': STUB_PATH,
      'pino-elasticsearch': STUB_PATH,
      'tap': STUB_PATH,
      'tape': STUB_PATH,
      'why-is-node-running': STUB_PATH,
      'desm': STUB_PATH,
      'fastbench': STUB_PATH,
      'react-native': STUB_PATH,
      'react-native-device-info': STUB_PATH,
      'react-native-keychain': STUB_PATH,
    },
  },
  // Improve HMR WebSocket reliability and fix SSR issues
  webpack: (config: any, { dev, isServer, webpack }: { dev: boolean; isServer: boolean; webpack: any }) => {
     
    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }

    // Web3/Wagmi build fix: Explicitly stub out problematic modules
    const path = require('node:path');
    const stubPath = path.join(process.cwd(), '../../shared/stubs/empty.ts');

    // Ensure shared components can resolve modules from the app's node_modules
    const appNodeModules = path.resolve(process.cwd(), 'node_modules');
    config.resolve.modules = [
      appNodeModules,
      ...(config.resolve.modules ?? ['node_modules']),
    ];

    config.resolve.alias = {
      ...config.resolve.alias,
      'thread-stream': stubPath,
      tap: stubPath,
      tape: stubPath,
      'why-is-node-running': stubPath,
      'pino-pretty': stubPath,
      'zod/mini': require.resolve('zod'),
      'zod/v4/core': require.resolve('zod'),
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