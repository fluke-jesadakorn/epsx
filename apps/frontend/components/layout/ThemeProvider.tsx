"use client";

import * as React from "react";
import {
  ThemeProvider as NextThemesProvider,
  useTheme,
} from "next-themes";
import { useThemeStore } from "@/lib/store/theme";

function ThemeWrapper({ children }: { children: React.ReactNode }) {
  const { resolvedTheme } = useTheme();
  const { setDarkMode } = useThemeStore();

  React.useEffect(() => {
    if (resolvedTheme) {
      setDarkMode(resolvedTheme === "dark");
    }
  }, [resolvedTheme, setDarkMode]);

  return children;
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  // Initialize theme on mount
  React.useEffect(() => {
    useThemeStore.getState().initializeTheme();
  }, []);

  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      <ThemeWrapper>{children}</ThemeWrapper>
    </NextThemesProvider>
  );
}
