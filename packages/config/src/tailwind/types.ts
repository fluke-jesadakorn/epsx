export interface TailwindConfigOptions {
  /** Content paths for Tailwind to scan for classes */
  content?: string[];
  
  /** Additional colors to merge with base color scheme */
  additionalColors?: {
    [key: string]: any;
    pancake?: {
      [key: string]: string;
    };
  };
  
  /** Enable animation definitions for frontend apps */
  enableAnimations?: boolean;
  
  /** Enable custom screen breakpoints (xs, 3xl) */
  enableCustomScreens?: boolean;
  
  /** Enable custom spacing values (18, 88, 128) */
  enableCustomSpacing?: boolean;
  
  /** Enable custom font family definitions */
  enableCustomFonts?: boolean;
  
  /** Enable background image gradient definitions */
  enableBackgroundImages?: boolean;
  
  /** Additional border radius values to merge with base */
  additionalBorderRadius?: {
    [key: string]: string;
  };
  
  /** Additional custom utilities to add */
  additionalUtilities?: {
    [key: string]: any;
  };
  
  /** Override any config values */
  override?: Partial<import('tailwindcss').Config>;
}