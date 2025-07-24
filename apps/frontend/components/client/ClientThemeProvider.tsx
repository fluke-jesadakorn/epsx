'use client';

import { ThemeProvider } from 'next-themes';
import { useEffect, useState } from 'react';

interface ClientThemeProviderProps {
  children: React.ReactNode;
  defaultTheme: string;
}

/**
 * Client-side theme provider with hydration optimization
 */
export function ClientThemeProvider({ children, defaultTheme }: ClientThemeProviderProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Prevent hydration mismatch by not rendering theme provider until mounted
  if (!mounted) {
    return <div suppressHydrationWarning>{children}</div>;
  }

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme={defaultTheme}
      enableSystem
      disableTransitionOnChange
    >
      {children}
    </ThemeProvider>
  );
}