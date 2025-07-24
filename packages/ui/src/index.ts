// Server-Safe Components (can be used in Server Components)
export * from "./components/badge";
export * from "./components/card";
export * from "./components/input";
export * from "./components/label";

// Client Components (require 'use client')
export * from "./components/button";
export * from "./components/dialog";
export * from "./components/dropdown-menu";
export * from "./components/form";
export * from "./components/select";
export * from "./components/tabs";

// Design System
export * from "./tokens/design-tokens";
export * from "./tokens/theme-config";
export * from "./tokens/tailwind-config";

// Client-Only Providers (require 'use client')
export * from "./providers/theme-provider";

// Utilities
export { cn as utilsCn } from "./lib/utils";
export { 
  responsiveUtils as respUtils,
  breakpoints,
  generateResponsiveClasses,
  getResponsiveValue,
  createMediaQuery,
  useMediaQuery,
  useBreakpoint,
  containerClasses,
  gridClasses,
  flexClasses,
  typographyClasses,
  spacingClasses,
  componentClasses,
  mobileUtils,
  cn,
  createResponsiveClass,
  getThemeResponsiveClass
} from "./utils/responsive";

export type { ResponsiveUtils as RespUtilsType } from "./utils/responsive";
