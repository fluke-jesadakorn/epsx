/**
 * ADMIN FRONTEND FORM COMPONENTS
 * Migrated to use shared PancakeForm components with backward compatibility
 * Replaces 232 lines of duplicate form implementation
 */

'use client';

import * as React from 'react';
import {
  PancakeFormAlias as Form,
  PancakeInputAlias as Input,
  PancakeLabelAlias as Label,
  PancakeSelectAlias as Select,
  PancakeCheckboxAlias as Checkbox,
  PancakeTextareaAlias as Textarea,
  PancakeFormFieldAlias as FormField,
  PancakeButton as Button,
  PancakeBadge as Badge,
  type PancakeInputProps,
  type PancakeLabelProps,
  type PancakeButtonProps,
  type PancakeBadgeProps,
  type PancakeFormFieldProps
} from '../../../../shared/components';

// ============================================================================
// LEGACY COMPATIBILITY LAYER
// ============================================================================

// Keep the same interfaces for seamless migration
interface ButtonProps {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  children?: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  type?: 'button' | 'submit' | 'reset';
}

interface InputProps {
  error?: boolean | string;
  helperText?: string;
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  type?: string;
  disabled?: boolean;
  className?: string;
}

interface BadgeProps {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline';
  children?: React.ReactNode;
  className?: string;
}

interface LabelProps {
  required?: boolean;
  children?: React.ReactNode;
  htmlFor?: string;
  className?: string;
}

interface FormFieldProps {
  label: string;
  id: string;
  required?: boolean;
  error?: string;
  helperText?: string;
  children: React.ReactNode;
  className?: string;
}

// Re-export with perfect backward compatibility
export { Form, Input, Label, Button, Select, Checkbox, Textarea, FormField };
export type { ButtonProps, InputProps, LabelProps, FormFieldProps };
