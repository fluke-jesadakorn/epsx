/**
 * Migration Utilities for Design System
 * 
 * This file provides utilities to help migrate from the old CSS-based
 * approach to the new CVA-based design system.
 * 
 * Benefits:
 * - Smooth migration path from old to new system
 * - Backward compatibility during transition
 * - Clear mapping between old and new approaches
 * - Type-safe migration helpers
 */

import React from 'react';
import { 
  buttonVariants, 
  cardVariants, 
  badgeVariants,
  gradientTextVariants,
  type ButtonVariants,
  type CardVariants,
  type BadgeVariants,
  type GradientTextVariants,
} from './components';

// ============================================================================
// LEGACY CLASS MAPPINGS
// ============================================================================

/**
 * Maps old CSS classes to new variant configurations
 */
export const legacyClassMappings = {
  // Button mappings
  buttons: {
    'btn-pancake-primary': { variant: 'primary' as const, size: 'md' as const },
    'btn-pancake-secondary': { variant: 'secondary' as const, size: 'md' as const },
    'btn-pancake-outline': { variant: 'outline' as const, size: 'md' as const },
    'btn-default': { variant: 'primary' as const, size: 'md' as const },
    'btn-destructive': { variant: 'destructive' as const, size: 'md' as const },
    'btn-secondary': { variant: 'secondary' as const, size: 'md' as const },
  },
  
  // Card mappings
  cards: {
    'card-pancake': { variant: 'pancake' as const, padding: 'md' as const },
    'card-pancake-enhanced': { variant: 'pancake' as const, padding: 'md' as const, glow: true },
    'card-default': { variant: 'default' as const, padding: 'md' as const },
    'glassmorphism': { variant: 'glass' as const, padding: 'md' as const },
  },
  
  // Text mappings
  text: {
    'pancake-gradient-text': { gradient: 'primary' as const, animation: 'normal' as const },
    'pancake-gradient-secondary-text': { gradient: 'secondary' as const, animation: 'normal' as const },
    'pancake-gradient-accent-text': { gradient: 'success' as const, animation: 'normal' as const },
    'text-pancake-gradient': { gradient: 'primary' as const, animation: 'normal' as const },
  },
  
  // Badge mappings
  badges: {
    'badge-primary': { variant: 'primary' as const, size: 'md' as const },
    'badge-secondary': { variant: 'secondary' as const, size: 'md' as const },
    'badge-success': { variant: 'success' as const, size: 'md' as const },
    'badge-warning': { variant: 'warning' as const, size: 'md' as const },
    'badge-destructive': { variant: 'destructive' as const, size: 'md' as const },
  },
} as const;

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Converts old button class names to new variant configurations
 */
export function migrateButtonClass(
  oldClassName: keyof typeof legacyClassMappings.buttons
): string {
  const config = legacyClassMappings.buttons[oldClassName];
  return buttonVariants(config);
}

/**
 * Converts old card class names to new variant configurations
 */
export function migrateCardClass(
  oldClassName: keyof typeof legacyClassMappings.cards
): string {
  const config = legacyClassMappings.cards[oldClassName];
  return cardVariants(config);
}

/**
 * Converts old text gradient class names to new variant configurations
 */
export function migrateTextClass(
  oldClassName: keyof typeof legacyClassMappings.text
): string {
  const config = legacyClassMappings.text[oldClassName];
  return gradientTextVariants(config);
}

/**
 * Converts old badge class names to new variant configurations
 */
export function migrateBadgeClass(
  oldClassName: keyof typeof legacyClassMappings.badges
): string {
  const config = legacyClassMappings.badges[oldClassName];
  return badgeVariants(config);
}

// ============================================================================
// GENERIC MIGRATION UTILITY
// ============================================================================

/**
 * Generic class migration utility that handles multiple class types
 */
export function migrateClassName(className: string): string {
  // Check if it's a button class
  if (className in legacyClassMappings.buttons) {
    return migrateButtonClass(className as keyof typeof legacyClassMappings.buttons);
  }
  
  // Check if it's a card class
  if (className in legacyClassMappings.cards) {
    return migrateCardClass(className as keyof typeof legacyClassMappings.cards);
  }
  
  // Check if it's a text class
  if (className in legacyClassMappings.text) {
    return migrateTextClass(className as keyof typeof legacyClassMappings.text);
  }
  
  // Check if it's a badge class
  if (className in legacyClassMappings.badges) {
    return migrateBadgeClass(className as keyof typeof legacyClassMappings.badges);
  }
  
  // Return original class if no migration is needed
  console.warn(`No migration available for class: ${className}`);
  return className;
}

// ============================================================================
// COMPONENT MIGRATION UTILITIES
// ============================================================================

/**
 * Creates a migration wrapper for Button components
 */
export function createMigrationButton(
  props: {
    variant?: keyof typeof legacyClassMappings.buttons | ButtonVariants['variant'];
    size?: ButtonVariants['size'];
    className?: string;
    children: React.ReactNode;
  } & React.ButtonHTMLAttributes<HTMLButtonElement>
) {
  const { variant = 'primary', size = 'md', className, children, ...rest } = props;
  
  // Check if variant is a legacy class name
  const variantConfig = typeof variant === 'string' && variant in legacyClassMappings.buttons
    ? legacyClassMappings.buttons[variant as keyof typeof legacyClassMappings.buttons]
    : { variant: variant as ButtonVariants['variant'], size };
  
  const buttonClass = buttonVariants(variantConfig);
  
  return (
    <button 
      className={`${buttonClass} ${className || ''}`}
      {...rest}
    >
      {children}
    </button>
  );
}

/**
 * Creates a migration wrapper for Card components
 */
export function createMigrationCard(
  props: {
    variant?: keyof typeof legacyClassMappings.cards | CardVariants['variant'];
    padding?: CardVariants['padding'];
    className?: string;
    children: React.ReactNode;
  } & React.HTMLAttributes<HTMLDivElement>
) {
  const { variant = 'default', padding = 'md', className, children, ...rest } = props;
  
  // Check if variant is a legacy class name
  const variantConfig = typeof variant === 'string' && variant in legacyClassMappings.cards
    ? legacyClassMappings.cards[variant as keyof typeof legacyClassMappings.cards]
    : { variant: variant as CardVariants['variant'], padding };
  
  const cardClass = cardVariants(variantConfig);
  
  return (
    <div 
      className={`${cardClass} ${className || ''}`}
      {...rest}
    >
      {children}
    </div>
  );
}

// ============================================================================
// MIGRATION VALIDATION
// ============================================================================

/**
 * Validates if a component is using legacy classes
 */
export function validateLegacyUsage(className: string): {
  hasLegacyClasses: boolean;
  legacyClasses: string[];
  suggestions: string[];
} {
  const classes = className.split(' ');
  const legacyClasses: string[] = [];
  const suggestions: string[] = [];
  
  const allLegacyClasses = [
    ...Object.keys(legacyClassMappings.buttons),
    ...Object.keys(legacyClassMappings.cards),
    ...Object.keys(legacyClassMappings.text),
    ...Object.keys(legacyClassMappings.badges),
  ];
  
  classes.forEach(cls => {
    if (allLegacyClasses.includes(cls)) {
      legacyClasses.push(cls);
      suggestions.push(`Use ${migrateClassName(cls)} instead of ${cls}`);
    }
  });
  
  return {
    hasLegacyClasses: legacyClasses.length > 0,
    legacyClasses,
    suggestions,
  };
}

// ============================================================================
// DEVELOPMENT HELPERS
// ============================================================================

/**
 * Development-only helper that warns about legacy class usage
 */
export function warnLegacyUsage(className: string, componentName?: string): void {
  if (process.env.NODE_ENV === 'development') {
    const validation = validateLegacyUsage(className);
    
    if (validation.hasLegacyClasses) {
      console.warn(
        `[Design System Migration] Legacy classes detected in ${componentName || 'component'}:`,
        validation.legacyClasses,
        '\nSuggestions:',
        validation.suggestions
      );
    }
  }
}

/**
 * HOC that adds migration warnings to components
 */
export function withMigrationWarnings<T extends { className?: string }>(
  Component: React.ComponentType<T>,
  componentName: string
) {
  return function MigrationAwareComponent(props: T) {
    if (props.className) {
      warnLegacyUsage(props.className, componentName);
    }
    
    return <Component {...props} />;
  };
}

// ============================================================================
// MIGRATION REPORT
// ============================================================================

/**
 * Generates a migration report for a codebase
 */
export interface MigrationReport {
  totalFiles: number;
  filesWithLegacyClasses: number;
  legacyClassUsage: Record<string, number>;
  migrationSuggestions: string[];
}

/**
 * Analyzes code and generates migration suggestions
 */
export function generateMigrationReport(codeFiles: { path: string; content: string }[]): MigrationReport {
  const report: MigrationReport = {
    totalFiles: codeFiles.length,
    filesWithLegacyClasses: 0,
    legacyClassUsage: {},
    migrationSuggestions: [],
  };
  
  const allLegacyClasses = [
    ...Object.keys(legacyClassMappings.buttons),
    ...Object.keys(legacyClassMappings.cards),
    ...Object.keys(legacyClassMappings.text),
    ...Object.keys(legacyClassMappings.badges),
  ];
  
  codeFiles.forEach(file => {
    let hasLegacyClasses = false;
    
    allLegacyClasses.forEach(legacyClass => {
      const regex = new RegExp(`["'\`]([^"'\`]*\\b${legacyClass}\\b[^"'\`]*)["'\`]`, 'g');
      const matches = file.content.match(regex);
      
      if (matches) {
        hasLegacyClasses = true;
        report.legacyClassUsage[legacyClass] = (report.legacyClassUsage[legacyClass] || 0) + matches.length;
        
        const suggestion = `Replace ${legacyClass} with ${migrateClassName(legacyClass)} in ${file.path}`;
        report.migrationSuggestions.push(suggestion);
      }
    });
    
    if (hasLegacyClasses) {
      report.filesWithLegacyClasses++;
    }
  });
  
  return report;
}

// ============================================================================
// EXPORTS
// ============================================================================

export {
  migrateButtonClass,
  migrateCardClass,
  migrateTextClass,
  migrateBadgeClass,
  migrateClassName,
  createMigrationButton,
  createMigrationCard,
  validateLegacyUsage,
  warnLegacyUsage,
  withMigrationWarnings,
  generateMigrationReport,
};