'use client'

/**
 * Permission Error Boundary (Admin Frontend)
 * Wrapper using shared component with admin-specific modal
 */

import { type ReactNode } from 'react'

import {
  SharedPermissionErrorBoundary,
  type BackendPermissionError
} from '@/shared/components/errors/PermissionErrorBoundary'
import { PermissionDeniedModal } from './PermissionDeniedModal'

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
