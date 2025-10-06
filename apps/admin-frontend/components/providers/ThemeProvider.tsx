'use client';

import { ThemeProvider as NextThemesProvider } from 'next-themes';

// Define local type instead of importing from internal path
interface ThemeProviderProps {
  attribute?: string | undefined;
  defaultTheme?: string | undefined;
  enableSystem?: boolean | undefined;
  disableTransitionOnChange?: boolean | undefined;
  children?: React.ReactNode;
}

export interface GlobalThemeProviderProps extends ThemeProviderProps {
  children: React.ReactNode;
}

/**
 *
 * @param root0
 * @param root0.children
 */
export function GlobalThemeProvider({ children, ...props }: GlobalThemeProviderProps) {
  return (
    <NextThemesProvider
      {...({
        attribute: "class",
        defaultTheme: "system",
        enableSystem: true,
        disableTransitionOnChange: true,
        ...props
      } as any)}
    >
      {children}
    </NextThemesProvider>
  );
}

export { GlobalThemeProvider as ThemeProvider };
export default GlobalThemeProvider;