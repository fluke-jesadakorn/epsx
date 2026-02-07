/**
 * Shared Design System
 *
 * Barrel export file for the complete design system.
 * Provides easy imports and a unified API for both Frontend and Admin-Frontend apps.
 *
 * Usage:
 * ```typescript
 * import { buttonVariants, colors, spacing, cn } from '@shared/design-system';
 * ```
 */

// ============================================================================
// DESIGN TOKENS
// ============================================================================

// ============================================================================
// DEFAULT EXPORT
// ============================================================================

import * as components from './components';
import * as tokens from './tokens';

export {
    // Animation system
    animation,
    // Border radius
    borderRadius,
    // Breakpoints
    breakpoints,
    // Color system
    colors,
    // Semantic color mappings
    semanticColors,
    // Shadow system
    shadows,
    // Spacing system
    spacing,
    // Typography system
    typography,
    // Z-index system
    zIndex,
    // Type exports
    type AnimationDuration,
    type AnimationEasing,
    type BorderRadius,
    type Breakpoint,
    type Color,
    type FontSize,
    type FontWeight,
    type Shadow,
    type Spacing,
    type ZIndex
} from './tokens';

// ============================================================================
// COMPONENT VARIANTS (CVA)
// ============================================================================

export {
    // Badge variants
    badgeVariants,
    // Button variants
    buttonVariants,
    // Card variants
    cardVariants,
    // Utility function
    cn, getActionButtonVariant,
    // Utility functions
    getStatusBadgeVariant,
    // Input variants
    inputVariants,
    // Loading variants
    loadingVariants,
    // Modal variants
    modalVariants,
    // Table variants
    tableVariants, type BadgeVariants, type ButtonVariants, type CardVariants, type InputVariants, type LoadingVariants, type ModalVariants, type TableVariants
} from './components';

// ============================================================================
// DESIGN SYSTEM CONFIGURATION
// ============================================================================

/**
 * Design system version for tracking
 */
export const DESIGN_SYSTEM_VERSION = '2.0.0';

/**
 * Design system metadata
 */
export const designSystemMeta = {
    name: 'EPSX Shared Design System',
    version: DESIGN_SYSTEM_VERSION,
    description: 'Type-safe design system shared between Frontend and Admin-Frontend apps',
    author: 'EPSX Team',
} as const;

/**
 * Default export with all design system utilities
 */
const designSystem = {
    tokens,
    components,
    meta: designSystemMeta,
} as const;

export default designSystem;
