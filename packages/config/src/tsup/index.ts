import { defineConfig, type Options } from 'tsup';

export interface TsupConfigOptions {
  /** Entry points - can be string, array, or object */
  entry?: Options['entry'];
  /** External dependencies to exclude from bundle */
  external?: string[];
  /** Add "use client" banner for React client components */
  useClient?: boolean;
  /** Path to tsconfig file */
  tsconfig?: string;
  /** Additional config to merge/override defaults */
  override?: Partial<Options>;
}

/**
 * Creates a standardized tsup configuration for EPSX packages
 */
export function createTsupConfig(options: TsupConfigOptions = {}): ReturnType<typeof defineConfig> {
  const {
    entry = ['src/index.ts'],
    external = [],
    useClient = false,
    tsconfig,
    override = {},
  } = options;

  const config: Options = {
    entry,
    format: ['cjs', 'esm'],
    dts: true,
    splitting: false,
    sourcemap: true,
    clean: true,
    treeshake: true,
    minify: false,
    external,
    ...(tsconfig && { tsconfig }),
    ...(useClient && {
      esbuildOptions(options: any) {
        options.banner = {
          js: '"use client";',
        };
      },
    }),
    ...override,
  };

  return defineConfig(config);
}

// Preset configurations for common package types
export const tsupPresets = {
  /** Basic library package */
  library: (options: TsupConfigOptions = {}) => createTsupConfig(options),
  
  /** React UI components package */
  reactUI: (options: TsupConfigOptions = {}) => createTsupConfig({
    external: [
      'react',
      'react-dom',
      'react-hook-form',
      '@radix-ui/react-label',
      '@radix-ui/react-slot',
      ...(options.external || []),
    ],
    useClient: true,
    ...options,
  }),
  
  /** Next.js package with server/client split */
  nextjs: (options: TsupConfigOptions = {}) => createTsupConfig({
    entry: {
      index: 'src/index.ts',
      server: 'src/server/index.ts',
      client: 'src/client/index.ts',
      middleware: 'src/middleware/index.ts',
      ...(typeof options.entry === 'object' ? options.entry : {}),
    },
    external: [
      'react',
      'next',
      ...(options.external || []),
    ],
    ...options,
  }),
  
  /** Multi-entry package with submodules */
  multiEntry: (entries: Record<string, string>, options: TsupConfigOptions = {}) => createTsupConfig({
    entry: entries,
    ...options,
  }),
};

// Default export for simple usage
export default createTsupConfig;