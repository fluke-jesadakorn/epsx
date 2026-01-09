'use client'

/**
 * Permission Denied Modal (Admin Frontend)
 * Wrapper using shared component with admin-specific UI
 */

import { AlertTriangle, Lock, Shield, Zap } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { BackendPermissionError } from '@/lib/errors/permission-error-handler'
import {
  SharedPermissionDeniedModal,
  type IconComponents,
  type UIComponents
} from '@/shared/components/errors/PermissionDeniedModal'

interface PermissionDeniedModalProps {
  error: BackendPermissionError
  open: boolean
  onClose: () => void
}

// Map admin-frontend UI components
const uiComponents: UIComponents = {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
}

const iconComponents: IconComponents = {
  AlertTriangle,
  Lock,
  Shield,
  Zap,
}

/**
 *
 * @param root0
 * @param root0.error
 * @param root0.open
 * @param root0.onClose
 */
export function PermissionDeniedModal({ error, open, onClose }: PermissionDeniedModalProps) {
  return (
    <SharedPermissionDeniedModal
      error={error}
      open={open}
      onClose={onClose}
      platform="admin"
      components={uiComponents}
      icons={iconComponents}
    />
  )
}
