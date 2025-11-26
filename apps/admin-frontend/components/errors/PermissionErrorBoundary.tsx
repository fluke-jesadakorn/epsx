'use client'

/**
 * Permission Error Boundary (Admin Frontend)
 * Catches permission errors and shows appropriate UI
 * Displays backend error messages for admin upgrade prompts
 */

import { Component, type ReactNode } from 'react'

import { PermissionDeniedModal } from './PermissionDeniedModal'

import {
  onPermissionError,
  isPermissionError,
  type BackendPermissionError,
  type PermissionErrorEvent
} from '@/lib/errors/permission-error-handler'

interface Props {
  children: ReactNode
  fallback?: ReactNode | ((error: BackendPermissionError) => ReactNode)
  onError?: (error: BackendPermissionError) => void
}

interface State {
  permissionError: BackendPermissionError | null
  hasError: boolean
}

/**
 *
 */
export class PermissionErrorBoundary extends Component<Props, State> {
  private unsubscribe?: () => void

  /**
   *
   * @param props
   */
  constructor(props: Props) {
    super(props)
    this.state = {
      permissionError: null,
      hasError: false
    }
  }

  /**
   *
   */
  componentDidMount() {
    // Listen for global permission errors
    this.unsubscribe = onPermissionError(this.handlePermissionError)
  }

  /**
   *
   */
  componentWillUnmount() {
    if (this.unsubscribe) {
      this.unsubscribe()
    }
  }

  /**
   *
   * @param error
   */
  static getDerivedStateFromError(error: unknown): State | null {
    // Handle React errors that are permission-related
    if (isPermissionError(error)) {
      return {
        hasError: true,
        permissionError: error.permissionError
      }
    }
    // Not a permission error, let it bubble up
    return null
  }

  handlePermissionError = (event: PermissionErrorEvent) => {
    this.setState({
      hasError: true,
      permissionError: event.error
    })

    // Call optional error handler
    if (this.props.onError) {
      this.props.onError(event.error)
    }
  }

  handleCloseModal = () => {
    this.setState({
      hasError: false,
      permissionError: null
    })
  }

  /**
   *
   */
  render() {
    const { children, fallback } = this.props
    const { hasError, permissionError } = this.state

    if (hasError && permissionError) {
      // Custom fallback provided
      if (fallback) {
        if (typeof fallback === 'function') {
          return fallback(permissionError)
        }
        return fallback
      }

      // Default modal UI
      return (
        <>
          {children}
          <PermissionDeniedModal
            error={permissionError}
            open={true}
            onClose={this.handleCloseModal}
          />
        </>
      )
    }

    return children
  }
}
