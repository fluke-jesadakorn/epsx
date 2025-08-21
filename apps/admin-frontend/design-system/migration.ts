/**
 * Admin Frontend Migration Utilities
 * 
 * Helper functions and mappings to transition from legacy CSS classes
 * to the new design system. Provides backward compatibility and gradual migration.
 * 
 * Features:
 * - Legacy class to design system mapping
 * - Automatic class conversion utilities
 * - Migration warnings in development
 * - Backward compatibility bridge
 */

import { 
  adminButtonVariants, 
  adminCardVariants, 
  adminBadgeVariants,
  type AdminButtonVariants,
  type AdminCardVariants,
  type AdminBadgeVariants,
} from './components';

// ============================================================================
// LEGACY CLASS MAPPINGS
// ============================================================================

/**
 * Map legacy CSS classes to new design system variants
 */
export const legacyClassMappings = {
  // Legacy button classes
  buttons: {
    'pancake-button': {
      variant: 'primary' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
      glow: 'medium' as AdminButtonVariants['glow'],
    },
    'pancake-button-secondary': {
      variant: 'secondary' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
      glow: 'medium' as AdminButtonVariants['glow'],
    },
    'admin-btn-assign': {
      variant: 'success' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'admin-btn-revoke': {
      variant: 'destructive' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'admin-btn-pending': {
      variant: 'warning' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'btn-primary': {
      variant: 'primary' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'btn-secondary': {
      variant: 'secondary' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'btn-success': {
      variant: 'success' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'btn-danger': {
      variant: 'destructive' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
    'btn-warning': {
      variant: 'warning' as AdminButtonVariants['variant'],
      size: 'default' as AdminButtonVariants['size'],
    },
  },
  
  // Legacy card classes
  cards: {
    'pancake-card': {
      variant: 'pancake' as AdminCardVariants['variant'],
      hover: 'both' as AdminCardVariants['hover'],
      padding: 'default' as AdminCardVariants['padding'],
    },
    'pancake-card-hover': {
      variant: 'pancake' as AdminCardVariants['variant'],
      hover: 'both' as AdminCardVariants['hover'],
      padding: 'default' as AdminCardVariants['padding'],
    },
    'admin-card-dashboard': {
      variant: 'default' as AdminCardVariants['variant'],
      hover: 'both' as AdminCardVariants['hover'],
      padding: 'md' as AdminCardVariants['padding'],
    },
    'admin-card-user': {
      variant: 'user' as AdminCardVariants['variant'],
      hover: 'lift' as AdminCardVariants['hover'],
      padding: 'default' as AdminCardVariants['padding'],
    },
    'admin-card-permission': {
      variant: 'permission' as AdminCardVariants['variant'],
      hover: 'glow' as AdminCardVariants['hover'],
      padding: 'default' as AdminCardVariants['padding'],
    },
    'billing-card': {
      variant: 'billing' as AdminCardVariants['variant'],
      hover: 'both' as AdminCardVariants['hover'],
      padding: 'md' as AdminCardVariants['padding'],
    },
    'analytics-chart-container': {
      variant: 'analytics' as AdminCardVariants['variant'],
      hover: 'none' as AdminCardVariants['hover'],
      padding: 'default' as AdminCardVariants['padding'],
    },
  },
  
  // Legacy badge/status classes
  badges: {
    'admin-status-active': {
      variant: 'active' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'admin-status-inactive': {
      variant: 'inactive' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'admin-status-pending': {
      variant: 'pending' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'admin-status-suspended': {
      variant: 'suspended' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'status-active': {
      variant: 'active' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'status-inactive': {
      variant: 'inactive' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'status-pending': {
      variant: 'pending' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'status-suspended': {
      variant: 'suspended' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'permission-granted': {
      variant: 'granted' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'permission-denied': {
      variant: 'denied' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'permission-pending': {
      variant: 'pending' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
    'permission-inherited': {
      variant: 'inherited' as AdminBadgeVariants['variant'],
      size: 'default' as AdminBadgeVariants['size'],
    },
  },
} as const;

/**
 * Common CSS class replacements
 */
export const cssClassReplacements = {
  // Color replacements
  'bg-blue-100': 'bg-primary-100',
  'bg-blue-500': 'bg-primary-500',
  'bg-blue-600': 'bg-primary-600',
  'text-blue-600': 'text-primary-600',
  'text-blue-800': 'text-primary-800',
  'border-blue-200': 'border-primary-200',
  'border-blue-300': 'border-primary-300',
  
  'bg-green-100': 'bg-success-100',
  'bg-green-500': 'bg-success-500',
  'text-green-800': 'text-success-800',
  'border-green-200': 'border-success-200',
  
  'bg-red-100': 'bg-error-100',
  'bg-red-500': 'bg-error-500',
  'text-red-800': 'text-error-800',
  'border-red-200': 'border-error-200',
  
  'bg-amber-100': 'bg-warning-100',
  'bg-amber-500': 'bg-warning-500',
  'text-amber-800': 'text-warning-800',
  'border-amber-200': 'border-warning-200',
  
  'bg-purple-100': 'bg-info-100',
  'bg-purple-500': 'bg-info-500',
  'text-purple-800': 'text-info-800',
  'border-purple-200': 'border-info-200',
  
  // Gray scale replacements
  'bg-gray-100': 'bg-neutral-100',
  'bg-gray-200': 'bg-neutral-200',
  'bg-gray-300': 'bg-neutral-300',
  'bg-gray-500': 'bg-neutral-500',
  'bg-gray-800': 'bg-neutral-800',
  'text-gray-600': 'text-neutral-600',
  'text-gray-700': 'text-neutral-700',
  'text-gray-800': 'text-neutral-800',
  'border-gray-200': 'border-neutral-200',
  'border-gray-300': 'border-neutral-300',
  
  // Layout replacements
  'rounded-lg': 'rounded-xl',
  'rounded-md': 'rounded-lg',
  'shadow-md': 'shadow-lg',
  'shadow-lg': 'shadow-xl',
} as const;

// ============================================================================
// MIGRATION UTILITIES
// ============================================================================

/**
 * Convert legacy button classes to new design system
 */
export function migrateLegacyButton(legacyClass: string): string {
  const mapping = legacyClassMappings.buttons[legacyClass as keyof typeof legacyClassMappings.buttons];
  
  if (mapping) {
    if (process.env.NODE_ENV === 'development') {
      console.warn(`[Migration] Legacy button class '${legacyClass}' should be replaced with design system variants:`, mapping);
    }
    
    return adminButtonVariants(mapping);
  }
  
  return legacyClass;
}

/**
 * Convert legacy card classes to new design system
 */
export function migrateLegacyCard(legacyClass: string): string {
  const mapping = legacyClassMappings.cards[legacyClass as keyof typeof legacyClassMappings.cards];
  
  if (mapping) {
    if (process.env.NODE_ENV === 'development') {
      console.warn(`[Migration] Legacy card class '${legacyClass}' should be replaced with design system variants:`, mapping);
    }
    
    return adminCardVariants(mapping);
  }
  
  return legacyClass;
}

/**
 * Convert legacy badge classes to new design system
 */
export function migrateLegacyBadge(legacyClass: string): string {
  const mapping = legacyClassMappings.badges[legacyClass as keyof typeof legacyClassMappings.badges];
  
  if (mapping) {
    if (process.env.NODE_ENV === 'development') {
      console.warn(`[Migration] Legacy badge class '${legacyClass}' should be replaced with design system variants:`, mapping);
    }
    
    return adminBadgeVariants(mapping);
  }
  
  return legacyClass;
}

/**
 * Replace CSS classes with design system equivalents
 */
export function replaceCSSClasses(className: string): string {
  const classes = className.split(' ');
  const replacedClasses = classes.map(cls => {
    const replacement = cssClassReplacements[cls as keyof typeof cssClassReplacements];
    
    if (replacement && process.env.NODE_ENV === 'development') {
      console.warn(`[Migration] CSS class '${cls}' should be replaced with '${replacement}'`);
    }
    
    return replacement || cls;
  });
  
  return replacedClasses.join(' ');
}

/**
 * Comprehensive class migration function
 */
export function migrateClassName(className: string): string {
  if (!className) return '';
  
  const classes = className.split(' ');
  const migratedClasses = classes.map(cls => {
    // Try button migration first
    if (cls in legacyClassMappings.buttons) {
      return migrateLegacyButton(cls);
    }
    
    // Try card migration
    if (cls in legacyClassMappings.cards) {
      return migrateLegacyCard(cls);
    }
    
    // Try badge migration
    if (cls in legacyClassMappings.badges) {
      return migrateLegacyBadge(cls);
    }
    
    // Try CSS replacements
    const replacement = cssClassReplacements[cls as keyof typeof cssClassReplacements];
    if (replacement) {
      if (process.env.NODE_ENV === 'development') {
        console.warn(`[Migration] CSS class '${cls}' should be replaced with '${replacement}'`);
      }
      return replacement;
    }
    
    return cls;
  });
  
  return migratedClasses.join(' ');
}

// ============================================================================
// MIGRATION ANALYSIS
// ============================================================================

/**
 * Analyze a codebase for migration opportunities
 */
export function analyzeMigrationOpportunities(codeString: string): {
  legacyButtons: string[];
  legacyCards: string[];
  legacyBadges: string[];
  cssReplacements: string[];
  totalFindings: number;
} {
  const findings = {
    legacyButtons: [] as string[],
    legacyCards: [] as string[],
    legacyBadges: [] as string[],
    cssReplacements: [] as string[],
    totalFindings: 0,
  };
  
  // Find legacy button classes
  Object.keys(legacyClassMappings.buttons).forEach(buttonClass => {
    const regex = new RegExp(`\\b${buttonClass}\\b`, 'g');
    const matches = codeString.match(regex);
    if (matches) {
      findings.legacyButtons.push(...matches);
    }
  });
  
  // Find legacy card classes
  Object.keys(legacyClassMappings.cards).forEach(cardClass => {
    const regex = new RegExp(`\\b${cardClass}\\b`, 'g');
    const matches = codeString.match(regex);
    if (matches) {
      findings.legacyCards.push(...matches);
    }
  });
  
  // Find legacy badge classes
  Object.keys(legacyClassMappings.badges).forEach(badgeClass => {
    const regex = new RegExp(`\\b${badgeClass}\\b`, 'g');
    const matches = codeString.match(regex);
    if (matches) {
      findings.legacyBadges.push(...matches);
    }
  });
  
  // Find CSS replacement opportunities
  Object.keys(cssClassReplacements).forEach(cssClass => {
    const regex = new RegExp(`\\b${cssClass}\\b`, 'g');
    const matches = codeString.match(regex);
    if (matches) {
      findings.cssReplacements.push(...matches);
    }
  });
  
  findings.totalFindings = 
    findings.legacyButtons.length +
    findings.legacyCards.length +
    findings.legacyBadges.length +
    findings.cssReplacements.length;
  
  return findings;
}

/**
 * Generate migration report for a file or codebase
 */
export function generateMigrationReport(
  filePath: string, 
  codeString: string
): {
  filePath: string;
  findings: ReturnType<typeof analyzeMigrationOpportunities>;
  recommendations: string[];
} {
  const findings = analyzeMigrationOpportunities(codeString);
  const recommendations: string[] = [];
  
  if (findings.legacyButtons.length > 0) {
    recommendations.push(
      `Replace ${findings.legacyButtons.length} legacy button classes with adminButtonVariants()`
    );
  }
  
  if (findings.legacyCards.length > 0) {
    recommendations.push(
      `Replace ${findings.legacyCards.length} legacy card classes with adminCardVariants()`
    );
  }
  
  if (findings.legacyBadges.length > 0) {
    recommendations.push(
      `Replace ${findings.legacyBadges.length} legacy badge classes with adminBadgeVariants()`
    );
  }
  
  if (findings.cssReplacements.length > 0) {
    recommendations.push(
      `Update ${findings.cssReplacements.length} CSS classes to use design system tokens`
    );
  }
  
  if (findings.totalFindings === 0) {
    recommendations.push('✅ No migration opportunities found - this file is already using the design system!');
  }
  
  return {
    filePath,
    findings,
    recommendations,
  };
}

// ============================================================================
// DEVELOPMENT HELPERS
// ============================================================================

/**
 * Development-only class name validator
 */
export function validateClassName(className: string): {
  isValid: boolean;
  warnings: string[];
  suggestions: string[];
} {
  if (process.env.NODE_ENV !== 'development') {
    return { isValid: true, warnings: [], suggestions: [] };
  }
  
  const warnings: string[] = [];
  const suggestions: string[] = [];
  const classes = className.split(' ');
  
  classes.forEach(cls => {
    // Check for legacy classes
    if (cls in legacyClassMappings.buttons) {
      warnings.push(`Legacy button class detected: ${cls}`);
      suggestions.push(`Use adminButtonVariants() instead of ${cls}`);
    }
    
    if (cls in legacyClassMappings.cards) {
      warnings.push(`Legacy card class detected: ${cls}`);
      suggestions.push(`Use adminCardVariants() instead of ${cls}`);
    }
    
    if (cls in legacyClassMappings.badges) {
      warnings.push(`Legacy badge class detected: ${cls}`);
      suggestions.push(`Use adminBadgeVariants() instead of ${cls}`);
    }
    
    // Check for replaceable CSS classes
    if (cls in cssClassReplacements) {
      warnings.push(`Replaceable CSS class detected: ${cls}`);
      suggestions.push(`Use ${cssClassReplacements[cls as keyof typeof cssClassReplacements]} instead of ${cls}`);
    }
  });
  
  return {
    isValid: warnings.length === 0,
    warnings,
    suggestions,
  };
}

/**
 * Higher-order component for automatic class migration
 * Note: This would require React import in actual usage
 */
export function withMigration<T extends { className?: string }>(
  Component: any
): any {
  return function MigratedComponent(props: T) {
    const migratedProps = {
      ...props,
      className: props.className ? migrateClassName(props.className) : undefined,
    };
    
    // Note: React.createElement would need React import in actual component files
    return { Component, props: migratedProps };
  };
}

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type LegacyButtonClass = keyof typeof legacyClassMappings.buttons;
export type LegacyCardClass = keyof typeof legacyClassMappings.cards;
export type LegacyBadgeClass = keyof typeof legacyClassMappings.badges;
export type CSSReplacement = keyof typeof cssClassReplacements;