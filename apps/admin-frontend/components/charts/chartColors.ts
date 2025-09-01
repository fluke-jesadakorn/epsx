/**
 * PancakeSwap + Windows Phone Chart Color Palette
 * Consistent colors for all chart components
 */

export const PancakeSwapColors = {
  // Primary PancakeSwap colors
  primary: '#FFC107',        // PancakeSwap yellow
  primaryDark: '#FF8F00',    // Darker yellow
  primaryLight: '#FFD54F',   // Lighter yellow
  
  // Secondary colors matching PancakeSwap theme
  secondary: '#FF6B35',      // Orange
  accent: '#4CAF50',         // Success green
  warning: '#FF9800',        // Warning orange
  error: '#F44336',          // Error red
  info: '#2196F3',          // Info blue
  
  // Gradient colors for Windows Phone live tiles
  gradients: {
    pancake: ['#FFC107', '#FF8F00'],
    success: ['#4CAF50', '#388E3C'],
    warning: ['#FF9800', '#F57C00'],
    error: ['#F44336', '#D32F2F'],
    info: ['#2196F3', '#1976D2'],
    analytics: ['#9C27B0', '#7B1FA2'],
    premium: ['#FF5722', '#E64A19'],
  },
  
  // Chart-specific color palettes
  chartPalette: [
    '#FFC107',  // Primary yellow
    '#FF6B35',  // Orange
    '#4CAF50',  // Green
    '#2196F3',  // Blue
    '#9C27B0',  // Purple
    '#FF5722',  // Deep orange
    '#607D8B',  // Blue grey
    '#795548',  // Brown
  ],
  
  // Windows Phone accent colors
  wpAccents: [
    '#FFC107',  // Yellow
    '#FF6B35',  // Orange
    '#4CAF50',  // Green
    '#2196F3',  // Blue
    '#9C27B0',  // Purple
    '#FF5722',  // Red
  ],
  
  // Background colors for dark mode compatibility
  backgrounds: {
    light: '#FFFFFF',
    dark: '#1A1A1A',
    cardLight: '#F5F5F5',
    cardDark: '#2D2D2D',
  },
  
  // Grid and axis colors
  grid: {
    light: '#E0E0E0',
    dark: '#404040',
  },
  
  text: {
    light: '#333333',
    dark: '#FFFFFF',
    muted: '#757575',
  }
} as const

export type PancakeSwapColorKey = keyof typeof PancakeSwapColors.chartPalette

/**
 * Get color from palette by index
 */
export function getChartColor(index: number): string {
  return PancakeSwapColors.chartPalette[index % PancakeSwapColors.chartPalette.length]
}

/**
 * Get gradient colors for Windows Phone tiles
 */
export function getGradientColors(type: keyof typeof PancakeSwapColors.gradients): [string, string] {
  return PancakeSwapColors.gradients[type]
}

/**
 * Get theme-aware colors
 */
export function getThemeColors(isDark: boolean = false) {
  return {
    background: isDark ? PancakeSwapColors.backgrounds.dark : PancakeSwapColors.backgrounds.light,
    card: isDark ? PancakeSwapColors.backgrounds.cardDark : PancakeSwapColors.backgrounds.cardLight,
    grid: isDark ? PancakeSwapColors.grid.dark : PancakeSwapColors.grid.light,
    text: isDark ? PancakeSwapColors.text.dark : PancakeSwapColors.text.light,
    textMuted: PancakeSwapColors.text.muted,
  }
}