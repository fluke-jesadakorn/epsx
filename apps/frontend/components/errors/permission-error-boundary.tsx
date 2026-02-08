'use client'

/**
 * Permission Error Boundary (Frontend)
 * Wrapper using shared component with frontend-specific modal
 */

import { type ReactNode } from 'react'

import {
  SharedPermissionErrorBoundary,
  type BackendPermissionError
} from '@/shared/components/errors/Permissionerror-boundary'
import { PermissionDeniedModal } from './permission-denied-modal'

interface Props {
  children: ReactNode
  fallback?: ReactNode | ((error: BackendPermissionError) => ReactNode)
  onError?: (error: BackendPermissionError) => void
}

export class PermissionErrorBoundary extends SharedPermissionErrorBoundary {
  constructor(props: Props) {
    super({ ...props, ModalComponent: PermissionDeniedModal })
  }
}
