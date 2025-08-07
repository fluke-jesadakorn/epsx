"use client";

import * as React from "react";

import { designTokens } from "../tokens/design-tokens";
import { themeConfig  } from "../tokens/theme-config";

import type {ThemeVariant} from "../tokens/theme-config";

// Theme context type
type ThemeContextType = {
  theme: ThemeVariant;
  setTheme: (theme: ThemeVariant) => void;
  isDarkMode: boolean;
  toggleDarkMode: () => void;
  tokens: typeof designTokens;
  config: typeof themeConfig;
};

// Create theme context
const ThemeContext = React.createContext<ThemeContextType | undefined>(undefined);

// Theme provider props
type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: ThemeVariant;
  storageKey?: string;
  enableSystem?: boolean;
  disableTransitions?: boolean;
};

// Theme provider component
export function ThemeProvider({
  children,
  defaultTheme = "default",
  storageKey = "epsx-theme",
  enableSystem = true,
  disableTransitions = false,
  ...props
}: ThemeProviderProps): React.JSX.Element {
  const [theme, setThemeState] = React.useState<ThemeVariant>(defaultTheme);
  const [isDarkMode, setIsDarkMode] = React.useState(false);

  // Initialize theme on mount
  React.useEffect(() => {
    const storedTheme = localStorage.getItem(storageKey) as ThemeVariant;
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "default" as ThemeVariant
      : "default";
    
    const initialTheme = storedTheme || (enableSystem ? systemTheme : defaultTheme);
    setThemeState(initialTheme);
    
    // Set initial dark mode state
    const darkMode = localStorage.getItem(`${storageKey}-dark`) === "true" ||
      (enableSystem && window.matchMedia("(prefers-color-scheme: dark)").matches);
    setIsDarkMode(darkMode);
    
    // Apply theme to document
    applyTheme(initialTheme, darkMode);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [defaultTheme, enableSystem, storageKey]);

  // Apply theme to document
  const applyTheme = React.useCallback((newTheme: ThemeVariant, darkMode: boolean) => {
    const root = window.document.documentElement;
    
    // Remove existing theme classes
    root.classList.remove("light", "dark");
    Object.keys(themeConfig.variants).forEach(t => {
      root.classList.remove(`theme-${t}`);
    });
    
    // Add new theme classes
    root.classList.add(darkMode ? "dark" : "light");
    root.classList.add(`theme-${newTheme}`);
    
    // Set data attributes
    root.setAttribute("data-theme", newTheme);
    root.setAttribute("data-mode", darkMode ? "dark" : "light");
    
    // Apply CSS custom properties
    const themeVariant = themeConfig.variants[newTheme];
    if (themeVariant) {
      const cssVars = themeConfig.utils.generateCSSVars(themeVariant);
      Object.entries(cssVars).forEach(([key, value]) => {
        root.style.setProperty(key, value);
      });
    }
    
    // Handle transitions
    if (disableTransitions) {
      root.style.setProperty("--transition-duration", "0ms");
      setTimeout(() => {
        root.style.removeProperty("--transition-duration");
      }, 0);
    }
  }, [disableTransitions]);

  // Set theme function
  const setTheme = React.useCallback((newTheme: ThemeVariant) => {
    setThemeState(newTheme);
    localStorage.setItem(storageKey, newTheme);
    applyTheme(newTheme, isDarkMode);
  }, [storageKey, isDarkMode, applyTheme]);

  // Toggle dark mode function
  const toggleDarkMode = React.useCallback(() => {
    const newDarkMode = !isDarkMode;
    setIsDarkMode(newDarkMode);
    localStorage.setItem(`${storageKey}-dark`, newDarkMode.toString());
    applyTheme(theme, newDarkMode);
  }, [isDarkMode, theme, storageKey, applyTheme]);

  // Listen for system theme changes
  React.useEffect(() => {
    if (!enableSystem) return;
    
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = (): void => {
      const systemDarkMode = mediaQuery.matches;
      // Only update if user hasn't manually set a preference
      if (localStorage.getItem(`${storageKey}-dark`) === null) {
        setIsDarkMode(systemDarkMode);
        applyTheme(theme, systemDarkMode);
      }
    };
    
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [theme, enableSystem, storageKey, applyTheme]);

  const value = React.useMemo(
    () => ({
      theme,
      setTheme,
      isDarkMode,
      toggleDarkMode,
      tokens: designTokens,
      config: themeConfig,
    }),
    [theme, setTheme, isDarkMode, toggleDarkMode]
  );

  return (
    <ThemeContext.Provider {...props} value={value}>
      {children}
    </ThemeContext.Provider>
  );
}

// Theme hook
export const useTheme = (): ThemeContextType => {
  const context = React.useContext(ThemeContext);

  if (context === undefined) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }

  return context;
};

// Higher-order component for theme-aware components
export function withTheme<T extends Record<string, unknown>>(
  Component: React.ComponentType<T & { theme?: ThemeContextType }>
): React.ForwardRefExoticComponent<React.PropsWithoutRef<T> & React.RefAttributes<unknown>> {
  return React.forwardRef<unknown, T>((props, ref) => {
    const theme = useTheme();
    return React.createElement(Component, { ...props, ref, theme } as T & { theme: ThemeContextType; ref: React.Ref<unknown> });
  });
}

// Theme variant selector component
type ThemeVariantSelectorProps = {
  className?: string;
  variants?: ThemeVariant[];
};

export function ThemeVariantSelector({ 
  className, 
  variants = ["default", "pancake", "trading"] 
}: ThemeVariantSelectorProps): React.JSX.Element {
  const { theme, setTheme } = useTheme();

  return (
    <div className={`flex gap-2 ${className || ""}`}>
      {variants.map((variant) => (
        <button
          key={variant}
          onClick={() => setTheme(variant)}
          className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${
            theme === variant
              ? "bg-primary text-primary-foreground"
              : "bg-secondary text-secondary-foreground hover:bg-secondary/80"
          }`}
        >
          {variant.charAt(0).toUpperCase() + variant.slice(1)}
        </button>
      ))}
    </div>
  );
}

// Dark mode toggle component
type DarkModeToggleProps = {
  className?: string;
  showLabel?: boolean;
};

export function DarkModeToggle({ className, showLabel = false }: DarkModeToggleProps): React.JSX.Element {
  const { isDarkMode, toggleDarkMode } = useTheme();

  return (
    <button
      onClick={toggleDarkMode}
      className={`inline-flex items-center gap-2 p-2 rounded-md transition-colors hover:bg-accent hover:text-accent-foreground ${className || ""}`}
      aria-label="Toggle dark mode"
    >
      {isDarkMode ? (
        <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
      ) : (
        <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      )}
      {showLabel && (
        <span className="text-sm font-medium">
          {isDarkMode ? "Light" : "Dark"}
        </span>
      )}
    </button>
  );
}