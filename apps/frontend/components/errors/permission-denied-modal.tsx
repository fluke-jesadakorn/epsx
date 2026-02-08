'use client'

/**
 * Permission Denied Modal (Frontend)
 * Wrapper using shared component with frontend-specific UI
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
} from '@/shared/components/errors/permission-denied-modal'

interface PermissionDeniedModalProps {
  error: BackendPermissionError
  open: boolean
  onClose: () => void
}

// Map frontend UI components
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

export function PermissionDeniedModal({ error, open, onClose }: PermissionDeniedModalProps): React.JSX.Element {
  return (
    <SharedPermissionDeniedModal
      error={error}
      open={open}
      onClose={onClose}
      platform="frontend"
      components={uiComponents}
      icons={iconComponents}
    />
  )
}
