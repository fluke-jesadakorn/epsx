'use client';

import { ThemeProvider as NextThemesProvider } from 'next-themes';
import { type ThemeProviderProps } from 'next-themes/dist/types';

export interface GlobalThemeProviderProps extends ThemeProviderProps {
  children: React.ReactNode;
}

export function GlobalThemeProvider({ children, ...props }: GlobalThemeProviderProps) {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
      {...props}
    >
      {children}
    </NextThemesProvider>
  );
}

export { GlobalThemeProvider as ThemeProvider };
export default GlobalThemeProvider;