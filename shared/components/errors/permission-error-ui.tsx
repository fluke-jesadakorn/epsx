// ============================================================================
// UNIFIED PERMISSION ERROR UI SYSTEM
// Consolidated error UI components for all EPSX applications
// ============================================================================

'use client'

// Import types from unified response handler
import {
  isInsufficientTierError,
  isPermissionDeniedError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '../../utils/response-handler'

import { GenericErrorUI } from './partials/generic-error-ui'
import { InsufficientTierUI } from './partials/insufficient-tier-ui'
import { PermissionDeniedUI } from './partials/permission-denied-ui'
import { PermissionExpiredUI } from './partials/permission-expired-ui'
import { RateLimitExceededUI } from './partials/rate-limit-exceeded-ui'
import type { IconComponentProps, PermissionErrorUIProps, UIComponentProps } from './partials/types'

// Export types for consumers
export type { IconComponentProps, UIComponentProps } from './partials/types'

// ============================================================================
// MAIN PERMISSION ERROR UI COMPONENT
// ============================================================================

export function UnifiedPermissionErrorUI({
  error,
  onRetry,
  onUpgrade,
  onLogin,
  onSupport,
  showRetry = true,
  showSupport = true,
  variant = 'card',
  className = '',
  platform = 'frontend',
  components,
  icons
}: PermissionErrorUIProps) {
  // Route to specific error UI based on error type
  if (isPermissionDeniedError(error)) {
    return (
      <PermissionDeniedUI
        error={error}
        onRetry={onRetry}
        onLogin={onLogin}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
        platform={platform}
        components={components}
        icons={icons}
      />
    )
  }

  if (isInsufficientTierError(error)) {
    return (
      <InsufficientTierUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
        platform={platform}
        components={components}
        icons={icons}
      />
    )
  }

  if (isPermissionExpiredError(error)) {
    return (
      <PermissionExpiredUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
        platform={platform}
        components={components}
        icons={icons}
      />
    )
  }

  if (isRateLimitExceededError(error)) {
    return (
      <RateLimitExceededUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
        platform={platform}
        components={components}
        icons={icons}
      />
    )
  }

  // Generic error fallback
  return (
    <GenericErrorUI
      error={error}
      onRetry={onRetry}
      onSupport={onSupport}
      showRetry={showRetry}
      showSupport={showSupport}
      variant={variant}
      className={className}
      platform={platform}
      components={components}
      icons={icons}
    />
  )
}

// ============================================================================
// CONVENIENCE WRAPPER COMPONENTS
// ============================================================================

/**
 * Frontend-specific wrapper with sensible defaults
 */
export function FrontendPermissionErrorUI(
  props: Omit<PermissionErrorUIProps, 'platform' | 'components' | 'icons'>
) {
  // The consuming frontend app must provide these components and icons
  // This is a placeholder - the actual implementation would import from the app's UI lib
  const mockComponents: UIComponentProps = {}
  const mockIcons: IconComponentProps = {}

  return (
    <UnifiedPermissionErrorUI
      {...props}
      platform="frontend"
      components={mockComponents}
      icons={mockIcons}
    />
  )
}

/**
 * Admin-specific wrapper with sensible defaults
 */
export function AdminPermissionErrorUI(
  props: Omit<PermissionErrorUIProps, 'platform' | 'components' | 'icons'>
) {
  // The consuming admin app must provide these components and icons
  // This is a placeholder - the actual implementation would import from the app's UI lib
  const mockComponents: UIComponentProps = {}
  const mockIcons: IconComponentProps = {}

  return (
    <UnifiedPermissionErrorUI
      {...props}
      platform="admin"
      components={mockComponents}
      icons={mockIcons}
    />
  )
}

export default UnifiedPermissionErrorUI