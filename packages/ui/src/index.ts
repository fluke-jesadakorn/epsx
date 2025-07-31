// ============================================================================
// CONSOLIDATED UI - Single Entry Point for All UI Components & Utils
// ============================================================================

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
export * from "./components/loading";
export * from "./components/select";
export * from "./components/tabs";

// Design System & Theme
export * from "./tokens/design-tokens";
export * from "./tokens/theme-config";
export * from "./tokens/tailwind-config";
export * from "./providers/theme-provider";

// Error Handling
export * from "./error";

// Validation (temporarily disabled for build - needs zod dependency)
// TODO: Re-enable validation exports after adding zod to dependencies
// export * from "./validation";

// Hooks & Utilities (consolidated)
export * from "./hooks";
export { cn as utilsCn } from "./lib/utils";

// ============================================================================
// IMPORT GUIDANCE - Use these imports to minimize dependencies:
// 
// Components:      import { Button, Card, Input } from '@epsx/ui';
// Theme:           import { ThemeProvider, useTheme } from '@epsx/ui';
// Validation:      import { ValidationPresets, FormHelpers } from '@epsx/ui';
// Hooks:           import { useBreakpoint, useMediaQuery } from '@epsx/ui';
// Utils:           import { utilsCn } from '@epsx/ui';
// ============================================================================
