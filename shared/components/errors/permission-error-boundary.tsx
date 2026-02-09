'use client'

/**
 * SHARED Permission Error Boundary
 * Catches permission errors and shows appropriate UI
 * Uses dependency injection for modal component
 * 
 * Single source of truth for both admin-frontend and frontend apps
 */

import { Component, type ComponentType, type ReactNode } from 'react'

import {
    isPermissionError,
    onPermissionError,
    type BackendPermissionError,
    type PermissionErrorEvent
} from '../../utils/permission-error-handler'

// ============================================================================
// TYPES
// ============================================================================

export interface PermissionDeniedModalProps {
    error: BackendPermissionError
    open: boolean
    onClose: () => void
}

export interface PermissionErrorBoundaryProps {
    children: ReactNode
    fallback?: ReactNode | ((error: BackendPermissionError) => ReactNode)
    onError?: (error: BackendPermissionError) => void
    ModalComponent: ComponentType<PermissionDeniedModalProps>
}

interface State {
    permissionError: BackendPermissionError | null
    hasError: boolean
}

// ============================================================================
// SHARED PERMISSION ERROR BOUNDARY
// ============================================================================

export class SharedPermissionErrorBoundary extends Component<PermissionErrorBoundaryProps, State> {
    private unsubscribe?: () => void

    constructor(props: PermissionErrorBoundaryProps) {
        super(props)
        this.state = {
            permissionError: null,
            hasError: false
        }
    }

    override componentDidMount() {
        // Listen for global permission errors
        this.unsubscribe = onPermissionError(this.handlePermissionError)
    }

    override componentWillUnmount() {
        if (this.unsubscribe) {
            this.unsubscribe()
        }
    }

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

    override render() {
        const { children, fallback, ModalComponent } = this.props
        const { hasError, permissionError } = this.state

        if (hasError && permissionError) {
            // Custom fallback provided
            if (fallback !== undefined && fallback !== null) {
                if (typeof fallback === 'function') {
                    return fallback(permissionError)
                }
                return fallback
            }

            // Default modal UI using injected component
            return (
                <>
                    {children}
                    <ModalComponent
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

// Re-export types for convenience
export type { BackendPermissionError, PermissionErrorEvent }
